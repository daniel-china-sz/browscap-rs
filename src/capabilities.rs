use crate::mapper::{MAPPER};
use crate::{
    BROWSER, BROWSER_MAJOR_VERSION, BROWSER_TYPE, BrowsCapField, Capabilities, DEVICE_TYPE,
    PLATFORM, PLATFORM_VERSION, capabilities,
};
use hashbrown::HashSet;
use std::cell::{RefCell};
use std::hash::Hash;
use std::sync::{Arc, OnceLock};

pub const UNKNOWN_BROWSCAP_VALUE: &'static str = "Unknown";

pub static DEFAULT_CAPABILITIES: OnceLock<Capabilities> = OnceLock::new();

#[derive(Debug)]
pub struct CapaCache {
    cache: RefCell<HashSet<Arc<Capabilities>>>,
}

impl CapaCache {
    pub fn new() -> CapaCache {
        CapaCache {
            cache: RefCell::new(HashSet::new()),
        }
    }

    pub fn get_or_insert(&self, capa: Capabilities) -> Arc<Capabilities> {
        self.cache.borrow_mut().get_or_insert(Arc::new(capa)).clone()
    }
}

impl Capabilities {
    fn new(values: Vec<&'static str>) -> Capabilities {
        Capabilities { my_values: values }
    }
    pub fn get_value(&self, field: &BrowsCapField) -> Option<&str> {
        let u_str = MAPPER
            .get()
            .unwrap()
            .read()
            .unwrap()
            .get_value(&self.my_values, field);
        return u_str;
    }

    pub fn get_browser(&self) -> Option<&str> {
        self.get_value(&BROWSER)
    }

    pub fn get_browser_type(&self) -> Option<&str> {
        self.get_value(&BROWSER_TYPE)
    }

    pub fn get_browser_major_version(&self) -> Option<&str> {
        self.get_value(&BROWSER_MAJOR_VERSION)
    }

    pub fn get_platform(&self) -> Option<&str> {
        self.get_value(&PLATFORM)
    }

    pub fn get_platform_version(&self) -> Option<&str> {
        self.get_value(&PLATFORM_VERSION)
    }

    pub fn get_device_type(&self) -> Option<&str> {
        self.get_value(&DEVICE_TYPE)
    }
}

impl Hash for Capabilities {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for value in self.my_values.iter() {
            value.hash(state);
        }
    }
}
impl Eq for Capabilities {}

impl PartialEq for Capabilities {
    fn eq(&self, other: &Self) -> bool {
        self.my_values == other.my_values
    }
}

pub fn init_default_capa<'a>(fields: &Vec<&'static BrowsCapField>) -> &'a Capabilities {
    DEFAULT_CAPABILITIES.get_or_init(|| {
        let mut result: Vec<&'static str> = Vec::new();
        for _i in 0..fields.len() {
            result.push(capabilities::UNKNOWN_BROWSCAP_VALUE);
        }
        Capabilities::new(result)
    })
}

/**
 * 缓存capabilities
 */
pub fn get_capabilities(values: Vec<&'static str>, capa_cache: &CapaCache) -> Arc<Capabilities> {
    let capabilities: Capabilities = Capabilities::new(values);
    capa_cache.get_or_insert(capabilities)
}

pub fn init_wild_card_capa<'a>(fields: &'a Vec<&'static BrowsCapField>) -> Arc<Capabilities> {
    let mut values = init_default_capa(fields).my_values.clone();
    let mapper = MAPPER.get().unwrap().read().unwrap();
    for (index, item) in values.iter_mut().enumerate() {
        let position_field = mapper.position_field(index);
        if let Some(field) = position_field {
            if *field == BROWSER || *field == BROWSER_TYPE {
                *item = "Default Browser";
                continue;
            }
            if field.is_default {
                *item = capabilities::UNKNOWN_BROWSCAP_VALUE;
            }
        }
    }
    Arc::new(Capabilities::new(values))
}
