use crate::inventory::ItemStack;
use crate::prelude::*;
use crate::world::BlockPosition;
use bytes::{Buf, BufMut};
use feather_items::{Item, ItemExt};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Write};

/// The maximum size, in bytes, of a VarInt.
pub const MAX_VAR_INT_SIZE: usize = 5;

/// An error which occurred when getting a Minecraft
/// type.
#[derive(Debug, Fail)]
pub enum McTypeError {
    #[fail(display = "VarInt size {} bytes too big", _0)]
    VarIntTooBig(usize),
    #[fail(display = "String length {} exceeds maximum length {}", _0, _1)]
    StringTooLong(usize, usize),
    #[fail(display = "{}", _0)]
    NotEnoughBytes(#[fail(cause)] bytes::TryGetError),
    #[fail(display = "{}", _0)]
    Nbt(#[fail(cause)] nbt::Error),
    #[fail(display = "Invalid boolean value {}", _0)]
    InvalidBoolean(u8),
    #[fail(display = "Invalid item ID {}", _0)]
    InvalidItemId(i32),
}

/// Identifies a type to which Minecraft-specific
/// types (`VarInt`, `VarLong`, etc.) can be written.
pub trait McTypeWrite {
    /// Writes a `VarInt` to the object. See wiki.vg for
    /// details on `VarInt`s and related types.
    ///
    /// Returns the number of bytes written to the buffer.
    ///
    /// # Panics
    /// Panics if there buffer is too small.
    fn put_var_int(&mut self, x: i32) -> usize;

    /// Writes a string to the object. This method
    /// will first write the length of the string in bytes
    /// encodes as a `VarInt` and will then write
    /// the UTF-8 bytes of the string.
    fn put_string(&mut self, x: &str);

    fn put_block_position(&mut self, x: &BlockPosition);

    fn put_bool(&mut self, x: bool);

    fn put_uuid(&mut self, x: &Uuid);

    /// Puts an NBT value into this buffer.
    ///
    /// # Note
    /// Unlike the other put_* methods, this
    /// method will expand the buffer as necessary.
    /// As a result, this function is guaranteed
    /// not to panic.
    fn put_nbt<T: Serialize>(&mut self, x: &T);

    fn put_slot(&mut self, slot: &Option<ItemStack>);
}

/// Identifies a type from which Minecraft-specified
/// types can be read.
pub trait McTypeRead {
    /// Reads a `VarInt` from this object, returning
    /// `Some(x)` if successful or `None` if the object
    /// does not contain a valid `VarInt`.
    fn try_get_var_int(&mut self) -> Result<i32, McTypeError>;

    /// Reads a string from the object.
    fn try_get_string(&mut self) -> Result<String, McTypeError>;

    fn try_get_block_position(&mut self) -> Result<BlockPosition, McTypeError>;

    fn try_get_bool(&mut self) -> Result<bool, McTypeError>;

    fn try_get_uuid(&mut self) -> Result<Uuid, McTypeError>;

    fn try_get_nbt<'de, T: Deserialize<'de>>(&mut self) -> Result<T, McTypeError>;

    fn try_get_slot(&mut self) -> Result<Option<ItemStack>, McTypeErrorr>;
}

impl<T: BufMut + Write> McTypeWrite for T {
    fn put_var_int(&mut self, x: i32) -> usize {
        let mut count = 0;
        loop {
            count += 1;
            let mut temp = (x & 0b0111_1111) as u8;
            x >>= 7;
            if x != 0 {
                temp |= 0b1000_0000;
            }
            self.put_u8(temp);
            if x == 0 {
                break;
            }
        }
        count
    }

    fn put_string(&mut self, x: &str) {
        let len = x.len();
        self.put_var_int(len as i32);

        let bytes = x.as_bytes();
        self.put_slice(bytes);
    }

    fn put_block_position(&mut self, x: &BlockPosition) {
        let result: u64 = ((x.x as u64 & 0x03FF_FFFF) << 38)
            | ((x.y as u64 & 0xFFF) << 26)
            | (x.z as u64 & 0x03FF_FFFF);

        self.put_u64(result);
    }

    fn put_bool(&mut self, x: bool) {
        if x {
            self.put_u8(1);
        } else {
            self.put_u8(0);
        }
    }

    fn put_uuid(&mut self, x: &Uuid) {
        self.put_slice(&x.as_bytes()[..]);
    }

    fn put_nbt<T: Serialize>(&mut self, x: &T) {
        nbt::to_writer(self, x, None).unwrap();
    }

    fn put_slot(&mut self, slot: &Option<ItemStack>) {
        self.put_bool(slot.is_some());

        if let Some(slot) = slot.as_ref() {
            self.put_var_int(slot.ty.native_protocol_id());
            slot.put_i8(slot.amount as i8);
            slot.put_u8(0x00); // TODO NBT support - this is TAG_End
        }
    }
}

impl<T: Buf> McTypeRead for T {
    fn try_get_var_int(&mut self) -> Result<i32, McTypeError> {
        let mut num_read = 0;
        let mut result = 0;
        loop {
            let read = self.try_get_u8().map_err(McTypeError::NotEnoughBytes)?;
            let value = read & 0b0111_1111;
            result |= (i32::from(value)) << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                return Err(McTypeError::VarIntTooBig(num_read));
            }
            if read & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(result)
    }

    fn try_get_string(&mut self) -> Result<String, McTypeError> {
        let len = self.try_get_var_int()? as usize;

        if len > 65536 {
            return Err(McTypeError::StringTooLong(len, 65536));
        }

        let mut result = String::with_capacity(len);
        for _ in 0..len {
            let c = self.try_get_u8().map_err(McTypeError::NotEnoughBytes)?;
            result.push(c as char);
        }

        Ok(result)
    }

    fn try_get_block_position(&mut self) -> Result<BlockPosition, McTypeError> {
        let val = self.try_get_i64().map_err(McTypeError::NotEnoughBytes)?;
        let x = val >> 38;
        let y = (val >> 26) & 0xFFF;
        let z = val << 38 >> 38;

        Ok(BlockPosition::new(x as i32, y as i32, z as i32))
    }

    fn try_get_bool(&mut self) -> Result<bool, McTypeError> {
        let val = self.try_get_u8().map_err(McTypeError::NotEnoughBytes)?;
        match val {
            0 => Ok(false),
            1 => Ok(true),
            val => Err(McTypeError::InvalidBoolean(val)),
        }
    }

    fn try_get_uuid(&mut self) -> Result<Uuid, McTypeError> {
        let mut bytes = [0; 16];
        self.try_copy_to_slice(&mut dst)
            .map_err(McTypeError::NotEnoughBytes);
        Ok(Uuid::from(bytes))
    }

    fn try_get_nbt<'de, T: Deserialize<'de>>(&mut self) -> Result<T, McTypeError> {
        let cursor = Cursor::new(self.bytes());
        let nbt = nbt::from_reader(cursor).map_err(McTypeError::Nbt)?;

        self.advance(cursor.position() as usize);

        Ok(nbt)
    }

    fn try_get_slot(&mut self) -> Result<Option<ItemStack>, McTypeError> {
        let exists = self.try_get_bool()?;

        if !exists {
            return Ok(None);
        }

        let id = self.try_get_var_int()?;
        let ty = Item::from_native_protocol_id(id).ok_or(McTypeError::InvalidItemId(id))?;
        let amount = self.try_get_u8().map_err(McTypeError::NotEnoughBytes)?;

        Ok(Some(ItemStack::new(ty, amount)))
    }
}

/// Returns the number of bytes which will be needed
/// to write a given VarInt.
pub fn varint_needed_bytes(x: i32) -> usize {
    if x == 0 {
        return 1;
    }

    // Find highest set bit
    let mut highest_bit = 0;
    for i in (0..32).rev() {
        if x & (1 << i) > 0 {
            highest_bit = i;
        }
    }

    // Divide by 7, rounding up
    (highest_bit + 6) / 7
}
