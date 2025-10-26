use crate::literal::Literal;
use bitvec::vec::BitVec;
use std::{
    fmt::{Debug, Formatter},
};

const EMPTY: Vec<u32> = Vec::new();
const SINGLE_VALUES: [[u32; 1]; 1024] = get_single_values();
const fn get_single_values() -> [[u32; 1]; 1024] {
    let mut result = [[0; 1]; 1024];
    let mut i = 0;
    while i < 1024 {
        result[i] = [i as u32];
        i += 1;
    }
    result
}

#[derive(Clone, PartialEq, Eq)]
pub struct SearchableString {
    my_str: Vec<char>,
    pub my_indices: Vec<Option<Vec<u32>>>,
    pub my_prefix_cache: Cache,
    pub my_postfix_cache: Cache,
    my_buffer: Vec<u32>,
}

impl Debug for SearchableString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.my_indices)
            .and_then(|_| writeln!(f, "{:?}", self.my_prefix_cache))
            .and_then(|_| writeln!(f, "{:?}", self.my_postfix_cache))
            .and_then(|_| writeln!(f, "{:?}", &self.my_buffer))
    }
}

#[derive(Clone, PartialEq, Eq,Debug)]
pub struct Cache {
    my_values: BitVec,
    my_is_known: BitVec,
}

impl SearchableString {
    pub fn new(string_value: String, max_index: usize) -> SearchableString {
        let my_buffer = vec![0; string_value.len()];
        SearchableString {
            my_str: string_value.chars().collect(),
            my_indices: vec![None; max_index],
            my_postfix_cache: Cache::new(),
            my_prefix_cache: Cache::new(),
            my_buffer,
        }
    }

    pub fn get_size(&self) -> usize {
        self.my_str.len()
    }

    pub fn ends_with(&mut self, literal: &Literal) -> bool {
        let index = literal.get_index();
        let cached = self.my_postfix_cache.get(index);
        if cached.is_some() {
            return cached.unwrap();
        }
        let result = literal.matches(
            &self.my_str,
            self.my_str.len() as i32 - literal.get_length() as i32,
        );
        self.my_postfix_cache.set(index, result);
        result
    }

    pub fn get_indices(&mut self, literal: &Literal) -> &Vec<u32> {
        let index = literal.get_index();
        if self.my_indices[index].is_none() {
            let values = self.find_indices(literal);
            self.my_indices[index]= Some(values);
        }
        self.my_indices[index].as_ref().unwrap()
    }

    pub fn find_indices(&mut self, literal: &Literal) -> Vec<u32> {
        let mut count = 0;
        let s = literal.get_first_char();
        for i in 0..self.my_str.len() {
            // Check the first char for better performance and check the complete string
            if (self.my_str[i] == s || s == '?') && literal.matches(&self.my_str, i as i32) {
                // This index matches
                self.my_buffer[count] = i as u32;
                count += 1;
            }
        }

        // Check whether any match has been found
        if count == 0 {
            return EMPTY;
        }

        // Use an existing array
        if count == 1 && self.my_buffer[0] < SINGLE_VALUES.len() as u32 {
            let index = self.my_buffer[0];
            return SINGLE_VALUES[index as usize].to_vec();
        }

        // Copy the values
        let mut values = vec![0; count];
        for i in 0..count {
            values[i] = self.my_buffer[i];
        }
        values
    }
}

pub fn starts_with(str: &mut SearchableString, literal: &Literal) -> bool {
    let index = literal.get_index();
    let cached = str.my_prefix_cache.get(index);
    if cached.is_some() {
        return cached.unwrap();
    }
    let result = literal.matches(&str.my_str, 0);
    str.my_prefix_cache.set(index, result);
    result
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            my_values: BitVec::new(),
            my_is_known: BitVec::new(),
        }
    }

    pub fn get(self: &Self, index: usize) -> Option<bool> {
        if self.my_values.get(index).map_or(false, |v| *v) {
            return Some(true);
        }
        if self.my_is_known.get(index).map_or(false, |v| *v) {
            return Some(false);
        }
        None
    }

    pub fn set(&mut self, index: usize, flag: bool) {
        bitset_set(&mut self.my_values, index, flag);
        bitset_set(&mut self.my_is_known, index, true);
    }
}

fn bitset_set(bitset: &mut BitVec, index: usize, value: bool) {
    if index >= bitset.len() {
        bitset.resize(index + 1, false);
    }
    bitset.set(index, value);
}

#[cfg(test)]
mod test_searchable_string {
    use super::*;
    use ustr::Ustr;

    #[test]
    fn test_base() {
        let abc = Literal::create_literal(Ustr::from("abc"));
        let ab = Literal::create_literal(Ustr::from("ab"));
        let string_value = "abababc".to_string();
        let mut cache = SearchableString::new(string_value, 0);
        assert_eq!(starts_with(&mut cache, &ab), true);
        assert_eq!(starts_with(&mut cache, &abc), false);
        //test cache
        assert_eq!(starts_with(&mut cache, &abc), false);

        assert_eq!(cache.ends_with(&abc), true);
        assert_eq!(cache.ends_with(&ab), false);
        //test cache
        assert_eq!(cache.ends_with(&ab), false);
    }

    #[test]
    fn test_get_indices() {
        let abc = Literal::create_literal(Ustr::from("abc"));
        let ab = Literal::create_literal(Ustr::from("ab"));
        let any_char = Literal::create_literal(Ustr::from("?ab"));
        let no_match = Literal::create_literal(Ustr::from("aaaaaaaaaaaaaaaaaa"));

        let mut cache = SearchableString::new("abababc".to_string(), no_match.my_index + 1);
        assert_eq!(vec![4; 1], *cache.get_indices(&abc));
        assert_eq!(vec![0, 2, 4], *cache.get_indices(&ab));
        assert_eq!(vec![1, 3], *cache.get_indices(&any_char));
        assert_eq!(Vec::<u32>::new(), *cache.get_indices(&no_match));

        //test cache
        assert!(std::ptr::eq(cache.get_indices(&ab), cache.get_indices(&ab)));
    }

    #[test]
    fn test_get_buffer() {
        let abc = Literal::create_literal(Ustr::from("abc"));
        let ab = Literal::create_literal(Ustr::from("ab"));
        let any_char = Literal::create_literal(Ustr::from("?ab"));
        let no_match = Literal::create_literal(Ustr::from("aaaaaaaaaaaaaaaaaa"));

        let mut cache = SearchableString::new("abababc".to_string(), no_match.my_index + 1);
        println!("{:?}", cache.find_indices(&abc));
        println!("{:?}", cache.find_indices(&ab));
        println!("{:?}", cache.find_indices(&any_char));
        println!("{:?}", cache.find_indices(&no_match));
    }

    #[test]
    fn test_cache() {
        let mut cache = Cache::new();
        assert_eq!(cache.get(0), None);
        cache.set(0, true);
        assert_eq!(cache.get(0), Some(true));

        assert_eq!(cache.get(1), None);
        cache.set(1, false);
        assert_eq!(cache.get(1), Some(false));
    }
}
