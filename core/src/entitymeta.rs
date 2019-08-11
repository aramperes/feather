//! This module implements handling of the entity
//! metadata format. See https://wiki.vg/Entity_metadata
//! for the specification.

use crate::bytebuf::{BufMutAlloc, ByteBuf};
use crate::network::mctypes::{McTypeWrite, MAX_VAR_INT_SIZE};
use crate::world::BlockPosition;
use bytes::{BufMut, TryGetError};
use hashbrown::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub enum MetaEntry {
    Byte(i8),
    VarInt(i32),
    Float(f32),
    String(String),
    Chat(String),
    OptChat(Option<String>),
    Slot, // TODO
    Boolean(bool),
    Rotation(f32, f32, f32),
    Position(BlockPosition),
    OptPosition(Option<BlockPosition>),
    Direction(Direction),
    OptUuid(Option<Uuid>),
    OptBlockId(Option<i32>),
    Nbt,      // TODO
    Particle, // TODO
}

impl MetaEntry {
    pub fn id(&self) -> i32 {
        match self {
            MetaEntry::Byte(_) => 0,
            MetaEntry::VarInt(_) => 1,
            MetaEntry::Float(_) => 2,
            MetaEntry::String(_) => 3,
            MetaEntry::Chat(_) => 4,
            MetaEntry::OptChat(_) => 5,
            MetaEntry::Slot => 6,
            MetaEntry::Boolean(_) => 7,
            MetaEntry::Rotation(_, _, _) => 8,
            MetaEntry::Position(_) => 9,
            MetaEntry::OptPosition(_) => 10,
            MetaEntry::Direction(_) => 11,
            MetaEntry::OptUuid(_) => 12,
            MetaEntry::OptBlockId(_) => 13,
            MetaEntry::Nbt => 14,
            MetaEntry::Particle => 15,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            MetaEntry::Byte(_) => 1,
            MetaEntry::VarInt(_) => MAX_VAR_INT_SIZE,
            MetaEntry::Float(_) => 4,
            MetaEntry::String(s) => MAX_VAR_INT_SIZE + s.as_bytes().len(),
            MetaEntry::Chat(s) => MAX_VAR_INT_SIZE + s.as_bytes().len(),
            MetaEntry::OptChat(o) => {
                if let Some(s) = o.as_ref() {
                    MAX_VAR_INT_SIZE + s.as_bytes().len()
                } else {
                    MAX_VAR_INT_SIZE
                }
            }
            MetaEntry::Slot => MAX_VAR_INT_SIZE + 3,
            MetaEntry::Boolean(_) => 1,
            MetaEntry::Rotation(_, _, _) => 12,
            MetaEntry::Position(_) => 8,
            MetaEntry::OptPosition(_) => 9,
            MetaEntry::Direction(_) => MAX_VAR_INT_SIZE,
            MetaEntry::OptUuid(_) => 17,
            MetaEntry::OptBlockId(_) => MAX_VAR_INT_SIZE,
            MetaEntry::Nbt => unimplemented!(),
            MetaEntry::Particle => unimplemented!(),
        }
    }
}

#[derive(Clone)]
pub struct EntityMetadata {
    values: HashMap<u8, MetaEntry>,
}

impl EntityMetadata {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn with(mut self, values: &[(u8, MetaEntry)]) -> Self {
        for val in values {
            self.values.insert(val.0, val.1.clone());
        }

        self
    }

    pub fn size(&self) -> usize {
        let mut count = 1;
        for (_, entry) in &self.values {
            count += 1 + MAX_VAR_INT_SIZE + entry.size()
        }
        count
    }
}

impl Default for EntityMetadata {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EntityMetaIo {
    fn put_metadata(&mut self, meta: &EntityMetadata);
    fn try_get_metadata(&mut self) -> Result<EntityMetadata, TryGetError>;
}

impl<T: BufMut> EntityMetaIo for T {
    fn put_metadata(&mut self, meta: &EntityMetadata) {
        for (index, entry) in meta.values.iter() {
            self.put_u8(*index);
            self.put_var_int(entry.id());
            write_entry_to_buf(entry, self);
        }

        self.put_u8(0xff); // End of metadata
    }

    fn try_get_metadata(&mut self) -> Result<EntityMetadata, TryGetError> {
        unimplemented!()
    }
}

fn write_entry_to_buf<T: BufMut>(entry: &MetaEntry, buf: &mut T) {
    match entry {
        MetaEntry::Byte(x) => buf.put_i8(*x),
        MetaEntry::VarInt(x) => buf.put_var_int(*x),
        MetaEntry::Float(x) => buf.put_f32(*x),
        MetaEntry::String(x) => buf.put_string(x),
        MetaEntry::Chat(x) => buf.put_string(x),
        MetaEntry::OptChat(ox) => {
            if let Some(x) = ox {
                buf.put_bool(true);
                buf.put_string(x);
            } else {
                buf.put_bool(false);
            }
        }
        MetaEntry::Slot => unimplemented!(),
        MetaEntry::Boolean(x) => buf.put_bool(*x),
        MetaEntry::Rotation(x, y, z) => {
            buf.put_f32(*x);
            buf.put_f32(*y);
            buf.put_f32(*z);
        }
        MetaEntry::Position(x) => buf.put_block_position(x),
        MetaEntry::OptPosition(ox) => {
            if let Some(x) = ox {
                buf.put_bool(true);
                buf.put_block_position(x);
            } else {
                buf.put_bool(false);
            }
        }
        MetaEntry::Direction(x) => buf.put_var_int(x.id()),
        MetaEntry::OptUuid(ox) => {
            if let Some(x) = ox {
                buf.put_bool(true);
                buf.put_uuid(x);
            } else {
                buf.put_bool(false);
            }
        }
        MetaEntry::OptBlockId(ox) => {
            if let Some(x) = ox {
                buf.put_var_int(*x);
            } else {
                buf.put_var_int(0); // No value implies air
            }
        }
        MetaEntry::Nbt => unimplemented!(),
        MetaEntry::Particle => unimplemented!(),
    }
}

#[derive(Clone)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn id(&self) -> i32 {
        match self {
            Direction::Down => 0,
            Direction::Up => 1,
            Direction::North => 2,
            Direction::South => 3,
            Direction::West => 4,
            Direction::East => 5,
        }
    }
}
