/// Header is the ResTable_header.
/// Each chunk in an arsc file has a header
#[derive(Debug)]
pub struct Header {
    pub resource_type: ResourceType,
    pub header_size: u16,
    pub size: u64,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ResourceType {
    Null = 0x0000,
    StringPool = 0x0001,
    Table = 0x0002,
    TablePackage = 0x0200,
    TableType = 0x0201,
    TableTypeSpec = 0x0202,
    TableLibrary = 0x0203,
}

impl From<u16> for ResourceType {
    fn from(bits: u16) -> Self {
        use ResourceType::*;
        match bits {
            0 => Null,
            1 => StringPool,
            2 => Table,
            0x0200 => TablePackage,
            0x0201 => TableType,
            0x0202 => TableTypeSpec,
            0x0203 => TableLibrary,
            bits => unreachable!("Unexpected bits: {bits}"),
        }
    }
}

/// Arsc represents an entire arsc file.
/// Itself is a chunk, with type `RES_TABLE_TYPE`
/// It consists of two parts:
///
/// 1. A global string pool, with type `RES_STRING_POOL_TYPE`
/// 2. A collection of packages, each with type `RES_TABLE_PACKAGE_TYPE`
#[derive(Debug)]
pub struct Arsc {
    pub packages: Vec<Package>,
    pub global_string_pool: StringPool,
}

impl Arsc {
    pub fn get_main_package(&self) -> Option<&Package> {
        if self.packages.len() > 0 {
            Some(&self.packages[0])
        }
        else {
            None
        }
    }

    pub fn get_string(&self, package_name: &String, key: String) -> Option<&String> {
        // find package
        let package = match self.packages.iter().find(|p| p.name.eq(package_name)) {
            Some(p) => p,
            None => return None
        };

        // get index of string table
        let string_table_idx = match package.type_names.strings.iter().position(|s| s.eq("string")) {
            Some(i) => i,
            None => return None
        };

        // get index of string
        let string_idx = match package.key_names.strings.iter().position(|s| s.eq(&key)) {
            Some(i) => i,
            None => return None
        };

        // find resource entry
        let res_entry = match package.types[string_table_idx].configs.iter().find(|c| c.resources.resources.iter().any(|r| r.name_index == string_idx)) {
            Some(config) => {
                match config.resources.resources.iter().find(|r| r.name_index == string_idx) {
                    Some(res) => res,
                    None => return None
                }
            }
            None => return None
        };

        let global_string_index = match &res_entry.value {
            ResourceValue::Plain(val) => val.data_index,
            ResourceValue::Bag{parent: _, values } => values.first().unwrap().1.data_index
        };

        if global_string_index < self.global_string_pool.strings.len() {
            Some(&self.global_string_pool.strings[global_string_index])
        }
        else {
            None
        }
    }
}

/// A chunk with header type `ResTable_package`.
/// It consists of multiple parts:
///
/// 1. package id
/// 2. package name
/// 3. type names string pool
/// 3. key names string pool
#[derive(Debug)]
pub struct Package {
    pub id: u32,
    pub name: String,
    pub type_names: StringPool,
    pub last_public_type: u32,
    pub types: Vec<Type>,
    pub key_names: StringPool,
    pub last_public_key: u32,
}

/// StringPool is a chunk that stores all the strings used in this chunk.
/// It consists of multiple parts:
///
/// 1. string offset array
/// 2. style offset array
/// 3. string content
/// 4. style content
/// 5. flags indicating the encoding (UTF8 or UTF-16) or sorting condition
#[derive(Debug)]
pub struct StringPool {
    pub flags: u32,
    pub strings: Vec<String>,
    pub styles: Vec<Style>,
}

impl StringPool {
    /// The flag indicates whether the strings are encoded with UTF-8
    pub(crate) const UTF8_FLAG: u32 = 0x00000100;

    pub fn use_utf8(&self) -> bool {
        self.flags & Self::UTF8_FLAG != 0
    }
}

/// Style information associated with a string in the string pool
#[derive(Debug)]
pub struct Style {
    pub spans: Vec<StyleSpan>,
}

impl Style {
    pub(crate) const RES_STRING_POOL_SPAN_END: u32 = u32::MAX;
}

#[derive(Debug)]
pub struct StyleSpan {
    /// This is the name of the span -- that is, the name of the XML
    /// tag that defined it.  The special value END (0xFFFFFFFF) indicates
    /// the end of an array of spans.
    pub name: u32,
    /// The start of the range of characters in the string that this span applies to.
    pub start: u32,
    /// The end of the range of characters in the string that this span applies to.
    pub end: u32,
}

/// Type is derived from type name string pool. It is an abstraction
/// from the original arsc file. It contains specs and configs, which
/// can be found in the arsc file
#[derive(Default, Debug)]
pub struct Type {
    /// id - 1 is the index pointing to a type name, that can be found at `type_names[id-1]`
    pub id: usize,
    pub specs: Option<Specs>,
    pub configs: Vec<Config>,
}

impl Type {
    pub fn with_id(id: usize) -> Self {
        Type {
            id,
            ..Default::default()
        }
    }
}

/// Specs is a chunk type with header type `RES_TABLE_TYPE_SPEC_TYPE`
#[derive(Debug)]
pub struct Specs {
    pub type_id: usize,
    pub res0: u8,
    pub res1: u16,
    pub specs: Vec<Spec>,
    pub header_size: u16,
}

#[derive(Default, Debug)]
pub struct Spec {
    pub flags: u32,
    pub id: usize,
}

impl Spec {
    pub fn new(flags: u32, id: usize) -> Self {
        Spec { flags, id }
    }
}

/// Config is a chunk type with header type `RES_TABLE_TYPE_TYPE`
#[derive(Debug)]
pub struct Config {
    pub type_id: usize,
    pub res0: u8,
    pub res1: u16,
    pub id: Vec<u8>,
    pub resources: Resources,
    pub header_size: u16,
}

#[derive(Debug)]
pub struct Resources {
    pub resources: Vec<ResourceEntry>,
    pub missing_entries: usize,
}

impl Resources {
    pub(crate) fn entry_count(&self) -> usize {
        self.missing_entries + self.resources.len()
    }
}

#[derive(Debug)]
pub struct ResourceEntry {
    pub flags: u16,
    pub name_index: usize,
    pub value: ResourceValue,
    /// spec_id points to the specific spec at `specs[spec_id]` that is associated with this resource
    pub spec_id: usize,
}

impl ResourceEntry {
    /// A flag indicating whether the resource is a plain value or a bag of values
    pub(crate) const ENTRY_FLAG_COMPLEX: u16 = 0x0001;

    pub fn is_bag(&self) -> bool {
        self.flags & Self::ENTRY_FLAG_COMPLEX != 0
    }
}

/// Resource values can have two types:
///
/// 1. Plain value
/// 2. Bag
///
/// Bag is a collection of values with a `parent` pointer
#[derive(Debug, Eq, PartialEq)]
pub enum ResourceValue {
    Bag {
        parent: u32,
        values: Vec<(u32, Value)>,
    },
    Plain(Value),
}

///
#[derive(Debug, Eq, PartialEq)]
pub struct Value {
    pub size: u16,
    pub zero: u8,
    pub r#type: u8,
    /// data_index points to `global_string_pool[data_index]` to represent a string
    pub data_index: usize,
}

impl Value {
    const TYPE_STRING: u8 = 0x03;

    /// return true if the type of the Value represents a string
    #[allow(dead_code)]
    pub fn is_string(&self) -> bool {
        self.r#type & Self::TYPE_STRING != 0
    }
}
