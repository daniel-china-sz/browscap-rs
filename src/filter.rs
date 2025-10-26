use crate::literal::Literal;
use crate::rule::Rule;
use crate::searchable_string::{self, SearchableString};
use bitvec::prelude::BitVec;
use std::fmt::Debug;
use std::ops::Not;
use ustr::Ustr;

#[derive(Debug)]
pub enum FilterType {
    Prefix(Literal),
    Contains(Literal),
}

#[derive(Debug)]
pub struct Filter {
    filter_type: FilterType,
    my_mask: BitVec,
}
impl Filter {
    pub(crate) fn new(filter_type: FilterType, my_mask: BitVec) -> Filter {
        let filter = Filter {
            filter_type,
            my_mask,
        };
        filter
    }
}


/*
    用长度为rule个数的bitvec，用每一个位保存当前prefix与所有逐个rule的prefix比较“起始于”的真假结果
    方法结果是返回一个记录prefix filter的bitvec数据，记录着与每个rule的前缀匹配情况
*/
pub(crate) fn create_prefix_masker(rules: &Vec<Rule>, pattern: Ustr) -> BitVec {
    let mut my_mask = BitVec::new();
    my_mask.resize(rules.len(), false);
    for (i, rule) in rules.iter().enumerate() {
        let r = match rule.get_prefix() {
            Some(prefix) => prefix.get_string().starts_with(pattern.as_str()),
            None => false,
        };
        if r {
            my_mask.set(i, true);
        }
    }
    my_mask
}

pub fn create_contains_masker(rules: &Vec<Rule>, pattern: Ustr) -> BitVec {
    let mut my_mask = BitVec::new();
    my_mask.resize(rules.len(), false);
    for (i, rule) in rules.iter().enumerate() {
        let r = rule.requires(pattern);
        if r {
            my_mask.set(i, true);
        }
    }
    my_mask
}

pub(crate) fn filter(
    searchable_string: &mut SearchableString,
    filters: &Vec<Filter>,
    excludes_len: usize,
) -> BitVec {
    let mut bit_vec = BitVec::new();
    bit_vec.resize(excludes_len, false);
    for filter in filters.iter() {
        match &filter.filter_type {
            FilterType::Prefix(literal) => {
                if !searchable_string::starts_with(searchable_string, literal) {
                    bit_vec |= &filter.my_mask;
                }
            }
            FilterType::Contains(literal) => {
                if !(searchable_string.get_indices(literal).len() > 0) {
                    bit_vec |= &filter.my_mask;
                }
            }
        }
    }
    bit_vec.not()
}
