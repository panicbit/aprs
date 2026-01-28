// This module and its submodules is based on:
// https://github.com/python/cpython/blob/a3990df6121880e8c67824a101bb1316de232898/Lib/pickle.py#L306

use std::hash::BuildHasherDefault;
use std::mem;
use std::ops::{Deref, DerefMut};

use aprs_value::{Int, Str, Value};
use eyre::{Context, ContextCompat, Result, anyhow, bail};
use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use itertools::Itertools;
use smallvec::SmallVec;

type Hasher = FxHasher;
type FnvIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<Hasher>>;

pub mod op;

mod dispatch;

pub const HIGHEST_PROTOCOL: u8 = 5;

pub fn unpickle<FindClass>(data: &[u8], find_class: FindClass) -> Result<Value>
where
    FindClass: FnMut(&str, &str) -> Result<Value>,
{
    Unpickler::new(data, find_class).load()
}

struct Unframer<'a> {
    data: &'a [u8],
    current_frame: Option<&'a [u8]>,
}

impl<'a> Unframer<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            current_frame: None,
        }
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.current_frame_is_finished() {
            self.current_frame = None;
        }

        if let Some(current_frame) = &mut self.current_frame {
            let (data, new_current_frame) = current_frame
                .split_at_checked(len)
                .context("pickle exhausted before end of frame")?;

            *current_frame = new_current_frame;

            Ok(data)
        } else {
            let (data, new_data) = self
                .data
                .split_at_checked(len)
                .context("pickle exhausted before end of stream")?;

            self.data = new_data;

            Ok(data)
        }
    }

    fn read_byte(&mut self) -> Result<u8> {
        let data = self.read_exact(1)?;

        Ok(data[0])
    }

    fn read_u16(&mut self) -> Result<u16> {
        let data = self.read_exact(2)?;
        let value = u16::from_le_bytes(data.try_into().unwrap());

        Ok(value)
    }

    fn read_u32(&mut self) -> Result<u32> {
        let data = self.read_exact(4)?;
        let value = u32::from_le_bytes(data.try_into().unwrap());

        Ok(value)
    }

    fn read_i32(&mut self) -> Result<i32> {
        let data = self.read_exact(4)?;
        let value = i32::from_le_bytes(data.try_into().unwrap());

        Ok(value)
    }

    fn read_u64(&mut self) -> Result<u64> {
        let data = self.read_exact(8)?;
        let value = u64::from_le_bytes(data.try_into().unwrap());

        Ok(value)
    }

    fn read_f64(&mut self) -> Result<f64> {
        let data = self.read_exact(8)?;
        let value = f64::from_be_bytes(data.try_into().unwrap());

        Ok(value)
    }

    fn current_frame_is_finished(&self) -> bool {
        let Some(current_frame) = &self.current_frame else {
            return true;
        };

        current_frame.is_empty()
    }

    fn load_frame(&mut self, frame_size: u64) -> Result<()> {
        if !self.current_frame_is_finished() {
            bail!("beginning of a new frame before end of current frame")
        }

        let frame_size = usize::try_from(frame_size).context("frame size exceeds pointer width")?;
        let frame = self.read_exact(frame_size)?;

        self.current_frame = Some(frame);

        Ok(())
    }
}

struct Unpickler<'a, FindClass> {
    unframer: Unframer<'a>,
    proto: u8,
    stack: Vec<Value>,
    meta_stack: Vec<Vec<Value>>,
    memo: FnvIndexMap<Value, Value>,
    find_class: FindClass,
    result: Option<Value>,
}

impl<'a, FindClass> Unpickler<'a, FindClass>
where
    FindClass: FnMut(&str, &str) -> Result<Value>,
{
    fn new(data: &'a [u8], find_class: FindClass) -> Self {
        Self {
            unframer: Unframer::new(data),
            proto: 0,
            stack: Vec::new(),
            meta_stack: Vec::new(),
            memo: FnvIndexMap::default(),
            find_class,
            result: None,
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    fn pop_mark(&mut self) -> Result<Vec<Value>> {
        let new_stack = self.pop_meta()?;
        let stack = mem::replace(&mut self.stack, new_stack);

        Ok(stack)
    }

    fn pop_meta(&mut self) -> Result<Vec<Value>> {
        self.meta_stack
            .pop()
            .context("tried to pop meta with empty meta stack")
    }

    pub fn last(&self) -> Result<&Value> {
        let value = self
            .stack
            .last()
            .context("tried to get value from empty stack")?;

        Ok(value)
    }

    pub fn load(mut self) -> Result<Value> {
        loop {
            let op = self.read_byte().context("read op")?;

            self.dispatch(op)?;

            if let Some(value) = self.result {
                return Ok(value);
            }
        }
    }

    pub fn load_mark(&mut self) -> Result<()> {
        let stack = mem::take(&mut self.stack);

        self.meta_stack.push(stack);

        Ok(())
    }

    pub fn load_stop(&mut self) -> Result<()> {
        let value = self.pop().context("empty stack")?;

        self.result = Some(value);

        Ok(())
    }

    pub fn load_reduce(&mut self) -> Result<()> {
        let args = self
            .pop()
            .context("tied to load reduce args with empty stack")?;
        let callable = self
            .pop()
            .context("tried to load reduce with too small stack")?;
        let callable = callable
            .as_callable()
            .context("tried to reduce with a non-callable")?;

        let value = callable.call(args)?;

        self.stack.push(value);

        Ok(())
    }

    pub fn load_binint(&mut self) -> Result<()> {
        let value = self.read_i32()?;
        let value = Value::Int(value.into());

        self.stack.push(value);

        Ok(())
    }

    pub fn load_binint1(&mut self) -> Result<()> {
        let value = self.read_byte()?;
        let value = Value::Int(value.into());

        self.stack.push(value);

        Ok(())
    }

    pub fn load_binint2(&mut self) -> Result<()> {
        let value = self.read_u16()?;
        let value = Value::Int(value.into());

        self.stack.push(value);

        Ok(())
    }

    pub fn load_none(&mut self) -> Result<()> {
        self.stack.push(Value::none());

        Ok(())
    }

    pub fn load_binunicode(&mut self) -> Result<()> {
        let len = self.read_u32()?;
        let len = usize::try_from(len)?;
        let value = self.read_exact(len)?;
        let value = str::from_utf8(value)?;
        let value = Value::str(value);

        self.push(value);

        Ok(())
    }

    pub fn load_append(&mut self) -> Result<()> {
        let value = self.pop().context("stack is empty")?;
        let list = self.last().context("stack too small")?;
        let list = list.as_list()?;

        // TODO: use `.append` or `.extend` attributes of `list_obj`
        list.push(value);

        Ok(())
    }

    pub fn load_empty_dict(&mut self) -> Result<()> {
        let value = Value::empty_dict();

        self.stack.push(value);

        Ok(())
    }

    pub fn load_appends(&mut self) -> Result<()> {
        let items = self.pop_mark()?;
        // TODO: cast to list here? are non-list types supported?
        let list_obj = self.last()?;

        // TODO: use `.append` or `.extend` attributes of `list_obj`
        list_obj.extend(items)?;

        Ok(())
    }

    pub fn load_binget(&mut self) -> Result<()> {
        let index = self.read_byte()?;
        let index = Value::Int(index.into());

        let value = self
            .memo
            .get(&index)
            .with_context(|| anyhow!("Memo value not found at index {index:?}"))?
            .clone();

        self.push(value);

        Ok(())
    }

    pub fn load_long_binget(&mut self) -> Result<()> {
        let index = self.read_u32()?;
        let index = Value::Int(index.into());

        let value = self
            .memo
            .get(&index)
            .with_context(|| anyhow!("Memo value not found at index {index:?}"))?
            .clone();

        self.push(value);

        Ok(())
    }

    pub fn load_empty_list(&mut self) -> Result<()> {
        self.stack.push(Value::empty_list());

        Ok(())
    }

    pub fn load_setitem(&mut self) -> Result<()> {
        let value = self.pop().context("empty stack")?;
        let key = self.pop().context("empty stack")?;
        let dict = self.last().context("empty stack")?;
        let dict = dict.as_dict()?;

        dict.write().insert(key, value)?;

        Ok(())
    }

    pub fn load_tuple(&mut self) -> Result<()> {
        let items = self.pop_mark()?;
        let tuple = Value::tuple(SmallVec::from(items));

        self.push(tuple);

        Ok(())
    }

    pub fn load_newobj(&mut self) -> Result<()> {
        let args = self.pop().context("empty stack")?;
        let class = self.pop().context("empty stack")?;
        // TODO: This should call `__new__` on the `class`. Might need a class type.3
        let class = class.as_callable()?;
        let value = class.call(args)?;

        self.push(value);

        Ok(())
    }

    pub fn empty_tuple(&mut self) -> Result<()> {
        self.stack.push(Value::empty_tuple());

        Ok(())
    }

    pub fn load_setitems(&mut self) -> Result<()> {
        let items = self.pop_mark()?;
        let dict = self.last()?;
        let mut dict = dict
            .as_dict()
            .context("tried to `setitems` on non-dict")?
            .write();
        let items = items.into_iter().tuples::<(_, _)>();

        dict.extend(items);

        Ok(())
    }

    pub fn load_binfloat(&mut self) -> Result<()> {
        let value = self.read_f64()?;
        let value = Value::from(value);

        self.push(value);

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

    pub fn load_tuple1(&mut self) -> Result<()> {
        let v1 = self
            .pop()
            .context("tried to construct 1-tuple from empty stack")?;

        let tuple = Value::tuple((v1,));

        self.push(tuple);

        Ok(())
    }

    pub fn load_tuple2(&mut self) -> Result<()> {
        let v2 = self
            .pop()
            .context("tried to construct 2-tuple from empty stack")?;
        let v1 = self
            .pop()
            .context("tried to construct 2-tuple from too small stack")?;

        let tuple = Value::tuple((v1, v2));

        self.push(tuple);

        Ok(())
    }

    pub fn load_tuple3(&mut self) -> Result<()> {
        let v3 = self
            .pop()
            .context("tried to construct 3-tuple from empty stack")?;
        let v2 = self
            .pop()
            .context("tried to construct 3-tuple from too small stack")?;
        let v1 = self
            .pop()
            .context("tried to construct 3-tuple from too small stack")?;

        let tuple = Value::tuple((v1, v2, v3));

        self.push(tuple);

        Ok(())
    }

    pub fn load_newtrue(&mut self) -> Result<()> {
        self.push(Value::True());

        Ok(())
    }

    pub fn load_newfalse(&mut self) -> Result<()> {
        self.push(Value::False());

        Ok(())
    }

    pub fn load_long1(&mut self) -> Result<()> {
        let len = self.read_byte()?;
        let len = usize::from(len);
        let bytes = self.read_exact(len)?;
        let n = Int::from_signed_bytes_le(bytes);
        let n = Value::Int(n);

        self.push(n);

        Ok(())
    }

    pub fn load_short_binunicode(&mut self) -> Result<()> {
        let len = self.read_byte()?;
        let len = usize::from(len);
        let value = self.read_exact(len)?;
        // TODO: this might be too strict, python uses `surrogatepass` error handler
        let value = str::from_utf8(value).context("invalid BinUnicode")?;
        let value = Str::from(value);
        let value = Value::Str(value);

        self.stack.push(value);

        Ok(())
    }

    pub fn load_empty_set(&mut self) -> Result<()> {
        self.push(Value::empty_set());

        Ok(())
    }

    pub fn load_additems(&mut self) -> Result<()> {
        let items = self.pop_mark()?;
        let set_obj = self.stack.last().context("empty stack")?;
        let set_obj = set_obj
            .as_set()
            .context("tried to add items to a non-set value")?;

        // TODO: try to use `.add` method if not a set (e.g. class or dict)
        set_obj.write().extend(items);

        Ok(())
    }

    pub fn load_stack_global(&mut self) -> Result<()> {
        let name = self.pop().context("stack global pop from empty stack")?;
        let name = name.as_str().context("stack global name is not a str")?;
        let module = self
            .pop()
            .context("stack global pop from too small stack")?;
        let module = module
            .as_str()
            .context("stack global module is not a str")?;

        // TODO: ensure name and type are strings
        // TODO: create single string type that also covers "binunicode"
        // TODO: custom global loading

        let value = (self.find_class)(module, name).context("find class failed")?;
        self.push(value);

        Ok(())
    }

    pub fn load_memoize(&mut self) -> Result<()> {
        let key = self.memo.len();
        let key = Value::Int(key.into());
        let value = self.last().context("load_memoize")?;

        self.memo.insert(key, value.clone());

        Ok(())
    }

    pub fn load_frame(&mut self) -> Result<()> {
        let frame_size = self.read_u64()?;

        self.unframer.load_frame(frame_size)
    }
}

impl<'a, FindClass> Deref for Unpickler<'a, FindClass> {
    type Target = Unframer<'a>;

    fn deref(&self) -> &Self::Target {
        &self.unframer
    }
}

impl<'a, FindClass> DerefMut for Unpickler<'a, FindClass> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unframer
    }
}
