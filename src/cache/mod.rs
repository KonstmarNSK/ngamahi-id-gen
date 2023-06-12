use std::sync::mpsc::Sender;
use crate::cache::cache::Cache;
use crate::cache::client::CacheClient;
use crate::cache::waker::{Flag, GetResult};
use crate::Range;

pub mod cache;
pub mod client;
mod waker;



pub struct MsgGet{
    key: String,
    range_size: u64,

    result: GetResult,
}

pub struct MsgPut{
    key: String,
    value: Range,

    result: Flag,
}

pub enum Msg {
    GetFromCache(MsgGet),
    PutToCache(MsgPut),
    Stop
}


pub fn new_cache() -> CacheClient {
    Cache::new()
}