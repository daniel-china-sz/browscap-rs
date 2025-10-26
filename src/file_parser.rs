use crate::capabilities::CapaCache;
use crate::error::ParseError;
use crate::rule::Rule;
use crate::{ BrowsCapField, UserAgentParser, capabilities};
use csv::{ReaderBuilder, StringRecord};
use hashbrown::HashSet;
use std::io;
use std::rc::Rc;
use ustr::Ustr;

pub struct FileParser {
    fields: Vec<&'static BrowsCapField>,
    rules: Vec<Rule>,
    capa_cache: CapaCache,
}

impl FileParser {
    pub fn new(unique_fields: HashSet<&'static BrowsCapField>) -> Self {
        let fields: Vec<&'static BrowsCapField> = unique_fields.into_iter().collect();
        crate::mapper::init_mapper(&fields);
        crate::capabilities::init_default_capa(&fields);
        FileParser {
            fields,
            rules: Vec::new(),
            capa_cache: CapaCache::new(),
        }
    }

    pub fn parse(&mut self, read: impl io::Read) {
        let csv_reader = ReaderBuilder::default().has_headers(true).from_reader(read);
        for record_r in csv_reader.into_records() {
            if let Ok(record) = record_r {
                let rule_result = self.get_rule(record);
                match rule_result {
                    Ok(rule) => {
                        self.rules.push(rule);
                    }
                    Err(_) => {}
                }
            }
        }
        self.rules.push(crate::rule::get_wild_card_rule(&self.fields));
        self.rules.shrink_to_fit();
    }

    pub fn get_rule(&self, record: StringRecord) -> Result<Rule, ParseError> {
        if record.len() <= 47 {
            return Err(ParseError::InvalidRecord);
        }
        if let Some(rule_str_column) = record.get(0) {
            let pattern = crate::rule::normalize_pattern(rule_str_column);
            let values = get_brows_cap_fields(&record, &self.fields);
            let capabilities = crate::capabilities::get_capabilities(values, &self.capa_cache);
            let rule = crate::rule::create_rule(pattern, capabilities);
            Ok(rule?)
        } else {
            Err(ParseError::InvalidRecord)
        }
    }
}

pub fn create_agent_parser(file_parser: FileParser) -> UserAgentParser {
    UserAgentParser::new(file_parser.rules)
}

//合并了get_value方法
fn get_brows_cap_fields<'b>(
    record: &StringRecord,
    fields: &Vec<&'static BrowsCapField>,
) -> Vec<Ustr> {
    let mut values: Vec<Ustr> = Vec::new();
    for field in fields.iter() {
        let value = record.get(field.index());
        if value.is_none() {
            values.push(Ustr::from(capabilities::UNKNOWN_BROWSCAP_VALUE));
            continue;
        }
        let trimmed = Rc::new(value.unwrap().trim().to_string());
        if trimmed.is_empty() {
            values.push(Ustr::from(capabilities::UNKNOWN_BROWSCAP_VALUE));
            continue;
        }
        values.push(Ustr::from(&trimmed));
    }
    values
}
