use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Header {
    pub type_flag: TypeFlag,
    pub header_size: u16,
    pub size: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TypeFlag {
    RES_NULL_TYPE = 0x0000,
    RES_STRING_POOL_TYPE = 0x0001,
    RES_TABLE_TYPE = 0x0002,
    RES_TABLE_PACKAGE_TYPE = 0x0200,
    RES_TABLE_TYPE_TYPE = 0x0201,
    RES_TABLE_TYPE_SPEC_TYPE = 0x0202,
    RES_TABLE_LIBRARY_TYPE = 0x0203,
}

impl From<u16> for TypeFlag {
    fn from(bits: u16) -> Self {
        use TypeFlag::*;
        match bits {
            0 => RES_NULL_TYPE,
            1 => RES_STRING_POOL_TYPE,
            2 => RES_TABLE_TYPE,
            0x0200 => RES_TABLE_PACKAGE_TYPE,
            0x0201 => RES_TABLE_TYPE_TYPE,
            0x0202 => RES_TABLE_TYPE_SPEC_TYPE,
            0x0203 => RES_TABLE_LIBRARY_TYPE,
            bits => unreachable!("Unexpected bits: {bits}"),
        }
    }
}

pub struct Arsc {
    pub packages: Vec<Package>,
    pub global_string_pool: StringPool,
}

pub struct Package {
    pub id: u32,
    pub name: String,
    pub type_names: StringPool,
    pub types: Vec<Type>,
    pub key_names: StringPool,
}

pub struct StringPool {
    pub strings: Vec<String>,
    pub flags: u32,
}

impl StringPool {
    pub(crate) const UTF8_FLAG: u32 = 0x00000100;
}

#[derive(Default)]
pub struct Type {
    pub id: usize, // id - 1 is the index to type_names
    pub specs: Vec<Spec>,
    pub configs: Vec<Config>,
}

#[derive(Default)]
pub struct Spec {
    pub flags: u32,
    pub id: usize,
    pub name_index: usize, // index to key_names
}

pub struct Config {
    pub res0: u8,
    pub res1: u16,
    pub entry_count: usize,
    pub id: Vec<u8>,
    pub resources: BTreeMap<usize, ResourceEntry>,
}

pub struct ResourceEntry {
    pub flags: u16,
    pub spec_id: usize, // index to spec
    pub value: ResourceValue,
}

pub enum ResourceValue {
    Bag {
        parent: u32,
        values: Vec<(u32, Value)>,
    },
    Plain(Value),
}

pub struct Value {
    pub size: u16,
    pub zero: u8,
    pub r#type: u8,
    pub data_index: usize, // index in global_string_pool
}

impl Value {
    pub(crate) const TYPE_STRING: u8 = 0x03;
}
