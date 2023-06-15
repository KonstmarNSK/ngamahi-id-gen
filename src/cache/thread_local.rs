/*
    This cache is intended to be used by a single thread.
    It just holds thread-local map.
 */


use std::cell::RefCell;
use crate::cache;
use crate::cache::cache_map::{self, CacheMap};
use crate::range::Range;

thread_local! {
    static MAP: RefCell<CacheMap> = RefCell::new(CacheMap::new());
}

pub fn new() -> Cache {
    Cache
}

#[derive(Clone)]
pub struct Cache;

impl Cache{
    pub async fn put(&self, key: String, value: Range) {
        MAP.with(|m| cache_map::store_range(key, value, &mut m.borrow_mut()));
    }

    pub async fn get(&self, key: String, range_size: u64) -> (Vec<Range>, u64){
        MAP.with(|m| cache_map::get_range(key, range_size, &mut m.borrow_mut()))
    }

    pub async fn stop(&self) {
        // nop
    }
}