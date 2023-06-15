use crate::range::Range;

mod common;
mod thread_local;
mod cache_map;


pub fn new_common() -> CacheClient {
    CacheClient::Common(common::new())
}

pub fn new_thread_local() -> CacheClient {
    CacheClient::ThreadLocal(thread_local::new())
}


#[derive(Clone)]
pub enum CacheClient{
    Common(common::CacheClient),
    ThreadLocal(thread_local::Cache)
}


impl CacheClient {
    pub async fn put(&self, key: String, value: Range) {
        match self {
            CacheClient::Common(c) => c.put(key, value).await,
            CacheClient::ThreadLocal(tl) => tl.put(key, value).await,
        }
    }

    pub async fn get(&self, key: String, range_size: u64) -> (Vec<Range>, u64) {
        match self {
            CacheClient::Common(c) => c.get(key, range_size).await,
            CacheClient::ThreadLocal(tl) => tl.get(key, range_size).await,
        }
    }

    pub async fn stop(&self) {
        match self {
            CacheClient::Common(c) => c.stop().await,
            CacheClient::ThreadLocal(tl) => tl.stop().await,
        }
    }
}