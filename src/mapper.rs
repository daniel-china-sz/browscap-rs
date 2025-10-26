use crate::BrowsCapField;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, OnceLock, RwLock};
use ustr::Ustr;

pub static MAPPER: OnceLock<Arc<RwLock<Mapper>>> = OnceLock::new();

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Mapper {
    field_index: HashMap<&'static BrowsCapField, usize>,
}

impl Hash for Mapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut hash = 0u64;
        for (key, value) in &self.field_index {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            key.hash(&mut hasher);
            value.hash(&mut hasher);
            hash ^= hasher.finish();
        }
        state.write_u64(hash);
    }
}

impl Mapper {
    pub fn new(field_index: HashMap<&'static BrowsCapField, usize>) -> Self {
        Mapper { field_index }
    }

    pub fn get_value(&self, values: &Vec<Ustr>, field: &BrowsCapField) -> Option<Ustr> {
        let index = self.field_index.get(field);
        match index {
            None => None,
            Some(i) => Some(values[*i]),
        }
    }

    pub fn position_field(&self, index: usize) -> Option<&BrowsCapField> {
        for (key, value) in self.field_index.iter() {
            if *value == index {
                return Some(*key);
            }
        }
        return None;
    }
}

pub fn init_mapper(fields: &Vec<&'static BrowsCapField>) {
    MAPPER.get_or_init(|| {
        let field_index_map: HashMap<&'static BrowsCapField, usize> = fields
            .iter()
            .enumerate()
            .map(|(index, field)| (*field, index))
            .collect();
        Arc::new(RwLock::new(Mapper::new(field_index_map)))
    });
}
