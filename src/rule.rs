use crate::capabilities::init_wild_card_capa;
use crate::error::ParseError;
use crate::literal::Literal;
use crate::searchable_string::SearchableString;
use crate::{BrowsCapField, Capabilities, searchable_string};
use regex::Regex;
use std::fmt::Debug;
use std::sync::Arc;
use ustr::Ustr;

pub struct Rule {
    my_prefix: Option<Arc<Literal>>,
    my_suffixes: Option<Vec<Arc<Literal>>>,
    my_postfix: Option<Arc<Literal>>,
    pattern_len: u32,
    my_capabilities: Arc<Capabilities>,
}
impl Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "my_prefix:{{{:?}}},", self.my_prefix)
            .and_then(|_| write!(f, "my_suffixes:{{{:?}}},", self.my_suffixes))
            .and_then(|_| write!(f, "my_postfix:{{{:?}}},", self.my_postfix))
            .and_then(|_| write!(f, "my_size:{{{:?}}},", self.pattern_len))
            .and_then(|_| write!(f, "my_capabilities:{{{:?}}}", self.my_capabilities))
    }
}

impl Rule {
    pub fn new(
        prefix: Option<Arc<Literal>>,
        suffixes: Option<Vec<Arc<Literal>>>,
        postfix: Option<Arc<Literal>>,
        pattern_len: u32,
        capabilities: Arc<Capabilities>,
    ) -> Self {
        Rule {
            my_prefix: prefix,
            my_suffixes: suffixes,
            my_postfix: postfix,
            pattern_len,
            my_capabilities: capabilities,
        }
    }

    pub fn get_size(&self) -> u32 {
        self.pattern_len
    }

    pub fn matches(&self, value: &mut SearchableString) -> bool {
        let start: i32;
        if self.my_prefix.is_none() {
            start = 0;
        } else if searchable_string::starts_with(value, &*self.my_prefix.as_ref().unwrap().clone())
        {
            start = self.my_prefix.as_ref().unwrap().get_length() as i32;
        } else {
            return false;
        }

        let end: i32;
        if self.my_postfix.is_none() {
            end = value.get_size() as i32 - 1;
        } else if value.ends_with(self.my_postfix.as_ref().unwrap()) {
            end =
                value.get_size() as i32 - 1 - self.my_postfix.as_ref().unwrap().get_length() as i32;
        } else {
            return false;
        }
        let x = self.check_wild_cards(value, self.my_suffixes.clone(), start, end);

        x
    }

    fn check_wild_cards(
        &self,
        value: &mut SearchableString,
        suffixes: Option<Vec<Arc<Literal>>>,
        start: i32,
        end: i32,
    ) -> bool {
        match suffixes {
            None => {
                // No wildcards
                return start == end + 1;
            }
            Some(suffix_list) => {
                // One wildcard
                if suffix_list.is_empty() {
                    return start <= end + 1;
                }

                let mut from = start;
                for suffix in suffix_list {
                    let match_pos = Self::check_wild_card(value, &suffix, from);
                    if match_pos == -1i32 {
                        return false;
                    }

                    from = suffix.get_length() as i32 + match_pos;
                    if from > end + 1 {
                        return false;
                    }
                }
                true
            }
        }
    }

    fn check_wild_card(value: &mut SearchableString, suffix: &Literal, start: i32) -> i32 {
        let x = value.get_indices(suffix);
        for i in x {
            let index = *i as i32;
            if index >= start {
                return index;
            }
        }
        -1i32
    }

    pub fn get_pattern(&self) -> String {
        // 精确计算容量
        let capacity = self.my_prefix.as_ref().map_or(0, |p| p.my_string.len())
            + self.my_suffixes.as_ref().map_or(0, |s| {
                if s.is_empty() {
                    0
                } else {
                    // 后缀字符串总长度 + 每个后缀前后的星号
                    s.iter().map(|sub| sub.my_string.len()).sum::<usize>() + s.len() + 1
                }
            })
            + self.my_postfix.as_ref().map_or(0, |p| p.my_string.len());

        let mut result = String::with_capacity(capacity);

        if let Some(prefix) = &self.my_prefix {
            result.push_str(&prefix.my_string);
        }

        if let Some(suffixes) = &self.my_suffixes {
            if !suffixes.is_empty() {
                result.push('*');
                for sub in suffixes {
                    result.push_str(&sub.my_string);
                    result.push('*');
                }
            }
        }

        if let Some(postfix) = &self.my_postfix {
            result.push_str(&postfix.my_string);
        }
        result
    }

    pub fn get_prefix(&self) -> Option<Arc<Literal>> {
        self.my_prefix.clone()
    }

    pub fn get_capabilities(&self) -> &Capabilities {
        &self.my_capabilities
    }

    pub fn requires(&self, value: Ustr) -> bool {
        // 检查前缀和后缀
        if Self::requires_single(&self.my_prefix, value)
            || Self::requires_single(&self.my_postfix, value)
        {
            return true;
        }

        // 检查后缀列表
        if let Some(suffixes) = &self.my_suffixes {
            for suffix in suffixes {
                if Self::requires_single(&Some(suffix.clone()), value) {
                    return true;
                }
            }
        }

        false
    }

    fn requires_single(literal: &Option<Arc<Literal>>, value: Ustr) -> bool {
        if let Some(lit) = literal {
            lit.requires(value)
        } else {
            false
        }
    }
}

pub fn create_rule(pattern: String, capabilities: Arc<Capabilities>) -> Result<Rule, ParseError> {
    let parts = get_parts(&pattern);
    if parts.is_empty() {
        return Err(ParseError::EmptyPattern);
    }
    if parts.len() == 1 {
        let first = &parts[0];
        if "*" == *first {
            return Err(ParseError::FixedPattern);
        }
        let option = crate::literal::get_literal(first);
        return Ok(Rule::new(
            Some(option),
            None,
            None,
            pattern.len() as u32,
            capabilities,
        ));
    }

    let first = &parts[0];
    let last = &parts[parts.len() - 1];

    // 预先计算需要的值
    let has_prefix = !("*" == *first);
    let has_postfix = !("*" == *last);
    let mut middle_parts: Vec<&str> = parts[if has_prefix { 1 } else { 0 }..if has_postfix {
        parts.len() - 1
    } else {
        parts.len()
    }]
        .to_vec();
    middle_parts.retain(|suffix| !(*suffix == "*"));

    let prefix = if has_prefix {
        Some(crate::literal::get_literal(first))
    } else {
        None
    };

    let postfix = if has_postfix {
        Some(crate::literal::get_literal(last))
    } else {
        None
    };

    let mut suffix_array: Vec<Arc<Literal>> = Vec::new();
    for part in middle_parts {
        suffix_array.push(crate::literal::get_literal(part));
    }

    Ok(Rule::new(
        prefix,
        Some(suffix_array),
        postfix,
        pattern.len() as u32,
        capabilities,
    ))
}

fn get_parts(pattern: &str) -> Vec<&str> {
    let mut parts: Vec<&str> = Vec::new();
    let mut start = 0;
    let chars: Vec<(usize, char)> = pattern.char_indices().collect();

    for (i, ch) in &chars {
        if *ch == '*' {
            if *i > start {
                // 添加前面的文本部分
                let sub_str = &pattern[start..*i];
                parts.push(sub_str);
            }
            // 添加星号部分
            let star_pos = *i;
            parts.push(&pattern[star_pos..star_pos + 1]);
            start = *i + 1;
        }
    }

    // 添加最后的部分
    if start < pattern.len() {
        parts.push(&pattern[start..]);
    }
    parts
}

pub(crate) fn normalize_pattern(pattern: &str) -> String {
    let pattern = pattern.to_lowercase();
    if pattern.contains("**") {
        let re = Regex::new(r"\*+").unwrap();
        re.replace_all(&pattern, "*").to_string()
    } else {
        pattern.to_string()
    }
}

pub fn get_wild_card_rule(fields: &Vec<&'static BrowsCapField>) -> Rule {
    Rule::new(
        None,
        Some(Vec::new()),
        None,
        "*".len() as u32,
        init_wild_card_capa(fields),
    )
}
