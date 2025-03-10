// This module and its submodules is based on:
// https://github.com/python/cpython/blob/a3990df6121880e8c67824a101bb1316de232898/Lib/pickle.py#L306

use std::io::{Cursor, Read};
use std::mem;
use std::ops::{Deref, DerefMut};

use anyhow::{Context, Result, bail};
use dumpster::sync::Gc;
use itertools::Itertools;

use crate::pickle::value::{BinStr, Dict, List, Number, NumberCache, Value};

mod dispatch;
mod op;
mod value;

const HIGHEST_PROTOCOL: u8 = 5;

pub fn unpickle<R: Read>(reader: &mut R) -> Result<()> {
    Unpickler::new(reader).load()?;

    Ok(())
}

struct Unframer<R> {
    reader: R,
    current_frame: Option<Cursor<Vec<u8>>>,
}

impl<R> Unframer<R>
where
    R: Read,
{
    fn new(reader: R) -> Self {
        Self {
            reader,
            current_frame: None,
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        if let Some(current_frame) = &mut self.current_frame {
            current_frame
                .read_exact(buf)
                .context("pickle exhausted before end of frame")
        } else {
            self.reader
                .read_exact(buf)
                .context("pickle exhausted before end of stream")
        }
    }

    fn read_vec(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; len];

        self.read_exact(&mut buf)?;

        Ok(buf)
    }

    fn read_byte(&mut self) -> Result<u8> {
        let buf = &mut [0];

        self.read_exact(buf)?;

        Ok(buf[0])
    }

    fn read_i32(&mut self) -> Result<i32> {
        let mut buf = [0; 4];

        self.read_exact(&mut buf)?;

        let value = i32::from_le_bytes(buf);

        Ok(value)
    }

    fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0; 8];

        self.read_exact(&mut buf)?;

        let value = u64::from_le_bytes(buf);

        Ok(value)
    }

    fn current_frame_is_finished(&self) -> bool {
        let Some(current_frame) = &self.current_frame else {
            return true;
        };

        // Casting to u64 should be fine.
        // If the buffer size were to exceed a u64,
        // then `Cursor::position()` and would run into issues as well.
        let frame_len = current_frame.get_ref().len() as u64;

        current_frame.position() == frame_len
    }

    fn load_frame(&mut self, frame_size: u64) -> Result<()> {
        if !self.current_frame_is_finished() {
            bail!("beginning of a new frame before end of current frame")
        }

        let frame_size = usize::try_from(frame_size).context("frame size exceeds pointer width")?;
        let frame = self.read_vec(frame_size)?;

        self.current_frame = Some(Cursor::new(frame));

        Ok(())
    }
}

struct Unpickler<R> {
    unframer: Unframer<R>,
    proto: u8,
    stack: Gc<List>,
    meta_stack: Vec<Gc<List>>,
    memo: Gc<Dict>,
    number_cache: NumberCache,
}

impl<R> Unpickler<R>
where
    R: Read,
{
    fn new(reader: R) -> Self {
        Self {
            unframer: Unframer::new(reader),
            proto: 0,
            stack: List::new(),
            meta_stack: Vec::new(),
            // TODO: memo probably needs to be an IndexMap
            memo: Dict::new(),
            number_cache: NumberCache::new(),
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop_mark(&mut self) -> Result<Gc<List>> {
        let stack = self.pop_meta()?;
        let stack = mem::replace(&mut self.stack, stack);

        Ok(stack)
    }

    fn pop_meta(&mut self) -> Result<Gc<List>> {
        self.meta_stack
            .pop()
            .context("tried to pop meta with empty meta stack")
    }

    pub fn last(&self) -> Result<Value> {
        let value = self
            .stack
            .last()
            .context("tried to get value from empty stack")?;

        Ok(value)
    }

    pub fn load(mut self) -> Result<()> {
        loop {
            let op = self.read_byte()?;

            self.dispatch(op)?;
        }
    }

    pub fn load_mark(&mut self) -> Result<()> {
        let stack = mem::replace(&mut self.stack, List::new());

        self.meta_stack.push(stack);

        Ok(())
    }

    pub fn load_binint(&mut self) -> Result<()> {
        let value = self.read_i32()?;
        let value = self.number_cache.get_i32(value);

        self.stack.push(value);

        Ok(())
    }

    pub fn load_binint1(&mut self) -> Result<()> {
        let value = self.read_byte()?;
        let value = self.number_cache.get_u8(value);

        self.stack.push(value);

        Ok(())
    }

    pub fn load_empty_dict(&mut self) -> Result<()> {
        let value = Value::empty_dict();

        self.stack.push(value);

        Ok(())
    }

    pub fn load_appends(&mut self) -> Result<()> {
        let items = self.pop_mark()?;
        let list_obj = self.last()?;

        list_obj.extend(items)?;

        Ok(())
    }

    pub fn load_empty_list(&mut self) -> Result<()> {
        self.stack.push(Value::empty_list());

        Ok(())
    }

    pub fn load_setitems(&mut self) -> Result<()> {
        let items = self.pop_mark()?;
        let dict = self
            .last()?
            .as_dict()
            .context("tried to `setitems` on non-dict")?;

        for (key, value) in items.iter().tuples() {
            dict.insert(key, value).context("load_setitems")?;
        }

        Ok(())
    }

    pub fn load_proto(&mut self) -> Result<()> {
        let proto = self.read_byte()?;

        if proto > HIGHEST_PROTOCOL {
            bail!("unsupported pickle protocol: {proto}")
        }

        self.proto = proto;

        Ok(())
    }

    pub fn load_long1(&mut self) -> Result<()> {
        let len = self.read_byte()?;
        let len = usize::from(len);
        let bytes = self.read_vec(len)?;
        let n = Number::from_signed_bytes_le(&bytes);
        let n = Value::Number(Gc::new(n));

        self.push(n);

        Ok(())
    }

    pub fn load_short_binunicode(&mut self) -> Result<()> {
        let len = self.read_byte()?;
        let len = usize::from(len);
        let value = self.read_vec(len)?;
        let value = BinStr(value);
        let value = Gc::new(value);
        let value = Value::BinStr(value);

        self.stack.push(value);

        Ok(())
    }

    pub fn load_memoize(&mut self) -> Result<()> {
        let key = self.memo.len();
        let key = self.number_cache.get_usize(key);
        let value = self.last().context("load_memoize")?;

        self.memo.insert(key, value)
    }

    pub fn load_frame(&mut self) -> Result<()> {
        let frame_size = self.read_u64()?;

        self.unframer.load_frame(frame_size)
    }
}

impl<R> Deref for Unpickler<R>
where
    R: Read,
{
    type Target = Unframer<R>;

    fn deref(&self) -> &Self::Target {
        &self.unframer
    }
}

impl<R> DerefMut for Unpickler<R>
where
    R: Read,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unframer
    }
}
