use crate::cache::common;
use crate::cache::common::waker::{Flag, GetResult};
use crate::range::Range;

mod cache;
mod client;
mod waker;
mod msg;

pub(in crate::cache) use client::CacheClient;


/*
    This cache is intended to be common for multiple threads:
    on creation it spawns a new thread which owns cache map and interacts with other threads
    via channel and atomic waker.
 */
pub(in crate::cache) fn new() -> CacheClient {
    cache::Cache::new()
}

