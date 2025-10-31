use crate::error::ParseError;
use crate::file_parser::FileParser;
use crate::filter::Filter;
use crate::rule::Rule;
use std::fs::File;
use std::io::BufReader;
use hashbrown::HashSet;

mod brows_cap_field;
mod capabilities;
mod error;
mod mapper;
mod rule;
mod searchable_string;
mod file_parser;
pub mod user_agent_parser;
mod filter;
mod literal;

// 枚举常量定义
pub const IS_MASTER_PARENT: BrowsCapField = BrowsCapField::new("IS_MASTER_PARENT", false, 0);
pub const IS_LITE_MODE: BrowsCapField = BrowsCapField::new("IS_LITE_MODE", false, 1);
pub const PARENT: BrowsCapField = BrowsCapField::new("PARENT", false, 2);
pub const COMMENT: BrowsCapField = BrowsCapField::new("COMMENT", false, 3);
pub const BROWSER: BrowsCapField = BrowsCapField::new("BROWSER", true, 4);
pub const BROWSER_TYPE: BrowsCapField = BrowsCapField::new("BROWSER_TYPE", true, 5);
pub const BROWSER_BITS: BrowsCapField = BrowsCapField::new("BROWSER_BITS", false, 6);
pub const BROWSER_MAKER: BrowsCapField = BrowsCapField::new("BROWSER_MAKER", false, 7);
pub const BROWSER_MODUS: BrowsCapField = BrowsCapField::new("BROWSER_MODUS", false, 8);
pub const BROWSER_VERSION: BrowsCapField = BrowsCapField::new("BROWSER_VERSION", false, 9);
pub const BROWSER_MAJOR_VERSION: BrowsCapField = BrowsCapField::new("BROWSER_MAJOR_VERSION", true, 10);
pub const BROWSER_MINOR_VERSION: BrowsCapField = BrowsCapField::new("BROWSER_MINOR_VERSION", false, 11);
pub const PLATFORM: BrowsCapField = BrowsCapField::new("PLATFORM", true, 12);
pub const PLATFORM_VERSION: BrowsCapField = BrowsCapField::new("PLATFORM_VERSION", true, 13);
pub const PLATFORM_DESCRIPTION: BrowsCapField = BrowsCapField::new("PLATFORM_DESCRIPTION", false, 14);
pub const PLATFORM_BITS: BrowsCapField = BrowsCapField::new("PLATFORM_BITS", false, 15);
pub const PLATFORM_MAKER: BrowsCapField = BrowsCapField::new("PLATFORM_MAKER", false, 16);
pub const IS_ALPHA: BrowsCapField = BrowsCapField::new("IS_ALPHA", false, 17);
pub const IS_BETA: BrowsCapField = BrowsCapField::new("IS_BETA", false, 18);
pub const IS_WIN16: BrowsCapField = BrowsCapField::new("IS_WIN16", false, 19);
pub const IS_WIN32: BrowsCapField = BrowsCapField::new("IS_WIN32", false, 20);
pub const IS_WIN64: BrowsCapField = BrowsCapField::new("IS_WIN64", false, 21);
pub const IS_IFRAMES: BrowsCapField = BrowsCapField::new("IS_IFRAMES", false, 22);
pub const IS_FRAMES: BrowsCapField = BrowsCapField::new("IS_FRAMES", false, 23);
pub const IS_TABLES: BrowsCapField = BrowsCapField::new("IS_TABLES", false, 24);
pub const IS_COOKIES: BrowsCapField = BrowsCapField::new("IS_COOKIES", false, 25);
pub const IS_BACKGROUND_SOUNDS: BrowsCapField = BrowsCapField::new("IS_BACKGROUND_SOUNDS", false, 26);
pub const IS_JAVASCRIPT: BrowsCapField = BrowsCapField::new("IS_JAVASCRIPT", false, 27);
pub const IS_VBSCRIPT: BrowsCapField = BrowsCapField::new("IS_VBSCRIPT", false, 28);
pub const IS_JAVA_APPLETS: BrowsCapField = BrowsCapField::new("IS_JAVA_APPLETS", false, 29);
pub const IS_ACTIVEX_CONTROLS: BrowsCapField = BrowsCapField::new("IS_ACTIVEX_CONTROLS", false, 30);
pub const IS_MOBILE_DEVICE: BrowsCapField = BrowsCapField::new("IS_MOBILE_DEVICE", false, 31);
pub const IS_TABLET: BrowsCapField = BrowsCapField::new("IS_TABLET", false, 32);
pub const IS_SYNDICATION_READER: BrowsCapField = BrowsCapField::new("IS_SYNDICATION_READER", false, 33);
pub const IS_CRAWLER: BrowsCapField = BrowsCapField::new("IS_CRAWLER", false, 34);
pub const IS_FAKE: BrowsCapField = BrowsCapField::new("IS_FAKE", false, 35);
pub const IS_ANONYMIZED: BrowsCapField = BrowsCapField::new("IS_ANONYMIZED", false, 36);
pub const IS_MODIFIED: BrowsCapField = BrowsCapField::new("IS_MODIFIED", false, 37);
pub const CSS_VERSION: BrowsCapField = BrowsCapField::new("CSS_VERSION", false, 38);
pub const AOL_VERSION: BrowsCapField = BrowsCapField::new("AOL_VERSION", false, 39);
pub const DEVICE_NAME: BrowsCapField = BrowsCapField::new("DEVICE_NAME", false, 40);
pub const DEVICE_MAKER: BrowsCapField = BrowsCapField::new("DEVICE_MAKER", false, 41);
pub const DEVICE_TYPE: BrowsCapField = BrowsCapField::new("DEVICE_TYPE", true, 42);
pub const DEVICE_POINTING_METHOD: BrowsCapField = BrowsCapField::new("DEVICE_POINTING_METHOD", false, 43);
pub const DEVICE_CODE_NAME: BrowsCapField = BrowsCapField::new("DEVICE_CODE_NAME", false, 44);
pub const DEVICE_BRAND_NAME: BrowsCapField = BrowsCapField::new("DEVICE_BRAND_NAME", false, 45);
pub const RENDERING_ENGINE_NAME: BrowsCapField = BrowsCapField::new("RENDERING_ENGINE_NAME", false, 46);
pub const RENDERING_ENGINE_VERSION: BrowsCapField = BrowsCapField::new("RENDERING_ENGINE_VERSION", false, 47);
pub const RENDERING_ENGINE_DESCRIPTION: BrowsCapField = BrowsCapField::new("RENDERING_ENGINE_DESCRIPTION", false, 48);
pub const RENDERING_ENGINE_MAKER: BrowsCapField = BrowsCapField::new("RENDERING_ENGINE_MAKER", false, 49);

const DEFAULT_FILE_NAME: &'static str = "browscap_sorted.csv";

pub trait Predicate<T> {
    fn test(&self, value: &T) -> bool;
}

// 为函数指针实现
impl<F, T> Predicate<T> for F
where
    F: Fn(&T) -> bool,
{
    fn test(&self, value: &T) -> bool {
        self(value)
    }
}

#[derive(Eq, Hash, PartialEq,Debug)]
pub struct BrowsCapField {
    name: &'static str,
    is_default: bool,
    ordinal: usize,
}

#[derive(Debug)]
pub struct Capabilities {
    my_values: Vec<&'static str>,
}

#[derive(Debug)]
pub struct UserAgentParser {
    my_rules: Vec<Rule>,
    my_filters: Vec<Filter>,
}


pub fn load_parser_default() -> Result<UserAgentParser, ParseError> {
    load_parser_with_fields(default_fields())
}

pub fn load_parser_with_fields(
    fields: Vec<&'static BrowsCapField>,
) -> Result<UserAgentParser, ParseError> {
    create_parser_by_file(fields, DEFAULT_FILE_NAME)
}

pub fn create_parser_by_file(
    fields: Vec<&'static BrowsCapField>,
    file_name: & str
) -> Result<UserAgentParser, ParseError> {
    let file = File::open(file_name).unwrap();
    let merged_unique_fields = merge_fields(fields);
    let reader = BufReader::new(file);
    let mut file_parser=FileParser::new(merged_unique_fields);
    file_parser.parse(reader);
    let user_agent_parser=file_parser::create_agent_parser(file_parser);
    Ok(user_agent_parser)
}

 fn default_fields() -> Vec<&'static BrowsCapField> {
    BrowsCapField::values()
        .iter()
        .filter(|i| i.is_default())
        .collect()
}

fn merge_fields(fields: Vec<&'static BrowsCapField>) -> HashSet<&'static BrowsCapField> {
    let mut unique_fields: HashSet<&BrowsCapField> = HashSet::new();
    fields.into_iter().for_each(|i| {
        unique_fields.insert(i);
    });
    default_fields().into_iter().for_each(|i| {
        unique_fields.insert(i);
    });
    unique_fields
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_load_fields() {
        let fields = default_fields();
        for field in fields.iter() {
            println!("{}", field.name());

        }
        println!("-------------");
        assert_eq!(fields.len(), 6)
    }

    #[test]
    fn test_merge_fields(){
        let mut my_fields:Vec<&'static BrowsCapField>=Vec::new();
        my_fields.push(&BROWSER);
        my_fields.push(&IS_BETA);
        my_fields.push(&PLATFORM);
        let merge_fields = merge_fields(my_fields);
        for merge_field in merge_fields.iter() {
            println!("{}", merge_field.name())
        }
        println!("-------------");
        assert_eq!(merge_fields.len(), 7)
    }
}
