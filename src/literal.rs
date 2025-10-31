use crate::searchable_string::SearchableString;
use dashmap::DashMap;
use hashbrown::HashSet;
use std::fmt::Debug;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, LazyLock};

pub static MY_NR_OF_INSTANCES: AtomicIsize = AtomicIsize::new(0);

pub static LITERAL_CACHE: LazyLock<DashMap<&str, Arc<Literal>>> = LazyLock::new(|| DashMap::new());
pub static STR_POOL: LazyLock<HashSet<&'static str>> = LazyLock::new(|| HashSet::new());

pub fn pool_str(str: &str) -> &'static str {
    match STR_POOL.get(str) {
        Some(i) => i,
        None => Box::leak(str.to_string().into_boxed_str()),
    }
}

pub struct Literal {
    pub(crate) my_string: &'static str,
    pub(crate) my_index: usize,
}
pub fn get_searchable_string(contents: String) -> SearchableString {
    let max_index = MY_NR_OF_INSTANCES.load(Ordering::SeqCst) + 1;
    SearchableString::new(contents, max_index as usize)
}

impl Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "my_string:{{{}}},my_index:{{{}}}",
            self.my_string, self.my_index
        )
    }
}

impl Literal {
    pub fn create_literal(contents: &'static str) -> Literal {
        Literal {
            my_string: contents,
            my_index: MY_NR_OF_INSTANCES.fetch_add(1, Ordering::SeqCst) as usize,
        }
    }

    pub(crate) fn get_first_char(&self) -> char {
        self.my_string.chars().next().unwrap()
    }

    pub(crate) fn get_length(&self) -> usize {
        self.my_string.len()
    }

    //Checks whether the value represents a complete substring from the from index.
    //glob匹配
    //逐个字符比较，忽略?不相等（即匹配全部单个字符）
    pub(crate) fn matches(&self, value: &Vec<char>, from: i32) -> bool {
        let len = self.my_string.len() as i32;
        if len + from > value.len() as i32 || from < 0 {
            return false;
        }

        for (i, ci) in self.my_string.chars().enumerate() {
            let vi = value[i + from as usize];
            if ci != vi && ci != '?' {
                return false;
            }
        }
        true
    }

    pub(crate) fn get_index(&self) -> usize {
        self.my_index
    }

    //Checks whether the my_string contains the value.
    pub fn requires(&self, value: &str) -> bool {
        let len = value.len();
        if len > self.my_string.len() {
            return false;
        }
        self.my_string.contains(value)
    }

    pub fn get_string(&self) -> &str {
        self.my_string
    }
}

pub fn get_literal(value: &str) -> Arc<Literal> {
    match LITERAL_CACHE.get(value) {
        Some(entry) => entry.value().clone(),
        None => {
            let key = pool_str(value);
            let literal = Arc::new(Literal::create_literal(key));
            LITERAL_CACHE.insert(key, literal.clone());
            literal
        }
    }
}

#[cfg(test)]
mod test_literal {
    use super::*;

    #[test]
    fn test_literal_basic() {
        let str = "abcdef";
        let literal = Literal::create_literal(str);
        assert_eq!(str.len(), literal.get_length());
        assert_eq!('a', literal.get_first_char());
        assert_eq!(literal.get_string(), literal.get_string());
        let literal2 = Literal::create_literal("di");
        assert_eq!(literal2.get_index(), 1)
    }

    #[test]
    fn test_literal_matches() {
        let literal = Literal::create_literal("def");
        let search: Vec<char> = "abcdef".chars().collect();
        assert_eq!(literal.matches(&search, 3), true);
        assert_eq!(literal.matches(&search, 0), false);
        assert_eq!(literal.matches(&search, 5), false);
        //assert_eq!(literal.matches(&search, -10), true);
        assert_eq!(literal.matches(&search, 100), false);

        let joker = Literal::create_literal("d?f");
        assert_eq!(joker.matches(&search, 3), true);
        assert_eq!(joker.matches(&search, 0), false);
        assert_eq!(joker.matches(&search, 5), false);
    }

    #[test]
    fn test_literal_requires() {
        let literal = Literal::create_literal("hello");
        assert_eq!(literal.requires("hello"), true);
        assert_eq!(literal.requires("hell"), true);
        assert_eq!(literal.requires("hello world"), false);
        assert_eq!(literal.requires("morning world"), false);
        assert_eq!(literal.requires("helloworld"), false);
    }
}
