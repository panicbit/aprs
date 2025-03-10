use std::io::Read;

use anyhow::{Result, bail};

use super::{Value, op};

impl<R, FindClass> super::Unpickler<R, FindClass>
where
    R: Read,
    FindClass: FnMut(&str, &str) -> Result<Value>,
{
    pub fn dispatch(&mut self, op: u8) -> Result<()> {
        match op {
            op::MARK => self.load_mark(),
            op::STOP => bail!("unhandled op: STOP"),
            op::POP => bail!("unhandled op: POP"),
            op::POP_MARK => bail!("unhandled op: POP_MARK"),
            op::DUP => bail!("unhandled op: DUP"),
            op::FLOAT => bail!("unhandled op: FLOAT"),
            op::INT => bail!("unhandled op: INT"),
            op::BININT => self.load_binint(),
            op::BININT1 => self.load_binint1(),
            op::LONG => bail!("unhandled op: LONG"),
            op::BININT2 => self.load_binint2(),
            op::NONE => self.load_none(),
            op::PERSID => bail!("unhandled op: PERSID"),
            op::BINPERSID => bail!("unhandled op: BINPERSID"),
            op::REDUCE => self.load_reduce(),
            op::STRING => bail!("unhandled op: STRING"),
            op::BINSTRING => bail!("unhandled op: BINSTRING"),
            op::SHORT_BINSTRING => bail!("unhandled op: SHORT_BINSTRING"),
            op::UNICODE => bail!("unhandled op: UNICODE"),
            op::BINUNICODE => bail!("unhandled op: BINUNICODE"),
            op::APPEND => bail!("unhandled op: APPEND"),
            op::BUILD => bail!("unhandled op: BUILD"),
            op::GLOBAL => bail!("unhandled op: GLOBAL"),
            op::DICT => bail!("unhandled op: DICT"),
            op::EMPTY_DICT => self.load_empty_dict(),
            op::APPENDS => self.load_appends(),
            op::GET => bail!("unhandled op: GET"),
            op::BINGET => self.load_binget(),
            op::INST => bail!("unhandled op: INST"),
            op::LONG_BINGET => bail!("unhandled op: LONG_BINGET"),
            op::LIST => bail!("unhandled op: LIST"),
            op::EMPTY_LIST => self.load_empty_list(),
            op::OBJ => bail!("unhandled op: OBJ"),
            op::PUT => bail!("unhandled op: PUT"),
            op::BINPUT => bail!("unhandled op: BINPUT"),
            op::LONG_BINPUT => bail!("unhandled op: LONG_BINPUT"),
            op::SETITEM => bail!("unhandled op: SETITEM"),
            op::TUPLE => self.load_tuple(),
            op::EMPTY_TUPLE => self.empty_tuple(),
            op::SETITEMS => self.load_setitems(),
            op::BINFLOAT => bail!("unhandled op: BINFLOAT"),
            // Protocol 2
            op::PROTO => self.load_proto(),
            op::NEWOBJ => self.load_newobj(),
            op::EXT1 => bail!("unhandled op: EXT1"),
            op::EXT2 => bail!("unhandled op: EXT2"),
            op::EXT4 => bail!("unhandled op: EXT4"),
            op::TUPLE1 => self.load_tuple1(),
            op::TUPLE2 => self.load_tuple2(),
            op::TUPLE3 => self.load_tuple3(),
            op::NEWTRUE => self.load_newtrue(),
            op::NEWFALSE => self.load_newfalse(),
            op::LONG1 => self.load_long1(),
            op::LONG4 => bail!("unhandled op: LONG4"),
            // Protocol 3 (Python 3.x)
            op::BINBYTES => bail!("unhandled op: BINBYTES"),
            op::SHORT_BINBYTES => bail!("unhandled op: SHORT_BINBYTES"),
            // Protocol 4
            op::SHORT_BINUNICODE => self.load_short_binunicode(),
            op::BINUNICODE8 => bail!("unhandled op: BINUNICODE8"),
            op::BINBYTES8 => bail!("unhandled op: BINBYTES8"),
            op::EMPTY_SET => self.load_empty_set(),
            op::ADDITEMS => bail!("unhandled op: ADDITEMS"),
            op::FROZENSET => bail!("unhandled op: FROZENSET"),
            op::NEWOBJ_EX => bail!("unhandled op: NEWOBJ_EX"),
            op::STACK_GLOBAL => self.load_stack_global(),
            op::MEMOIZE => self.load_memoize(),
            op::FRAME => self.load_frame(),
            // Protocol 5
            op::BYTEARRAY8 => bail!("unhandled op: BYTEARRAY8"),
            op::NEXT_BUFFER => bail!("unhandled op: NEXT_BUFFER"),
            op::READONLY_BUFFER => bail!("unhandled op: READONLY_BUFFER"),
            _ => bail!("unknown op: 0x{op:02x} ('{}')", char::from(op)),
        }
    }
}
