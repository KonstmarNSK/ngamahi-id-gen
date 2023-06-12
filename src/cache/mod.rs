use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::task::{Context, Poll};
use std::thread;

use atomic_waker::AtomicWaker;
use crate::cache::cache_client::CacheClient;
use crate::cache::cache_response::CacheResult;

use crate::Range;

mod cache_response;
mod cache_client;

type CacheMap = HashMap<String, Vec<Range>>;

struct Cache {
    ranges: CacheMap,
    pipe_recv: Receiver<Command>,
}

impl Cache {
    fn new(recv: Receiver<Command>) -> Self {
        Cache {
            ranges: HashMap::new(),
            pipe_recv: recv,
        }
    }
}


pub fn new_cache() -> CacheClient {
    let (rx, tx) = std::sync::mpsc::channel();

    let client = CacheClient { sender: rx };

    thread::spawn(move || {
        let mut cache = Cache::new(tx);

        println!("Created cache thread");

        for cmd in cache.pipe_recv.iter() {
            println!("Got command.");

            match cmd {
                Command::GetRange(mut get_cmd) => {
                    println!("trying to get range from cache");
                    let result =  get_range(get_cmd.seq_name, get_cmd.range_size, &mut cache.ranges);
                    let result = Box::new(result);
                    get_cmd.response.set_result(result);
                },

                Command::PutRange(put_cmd) => {
                    store_range(put_cmd.seq_name, put_cmd.range, &mut cache.ranges)
                },

                Command::Stop => break
            }
        }
    });

    client
}


fn store_range(seq_name: String, range: Range, map: &mut CacheMap) -> () {
    let mut ranges = map.get_mut(&seq_name).unwrap();
    ranges.push(range);
}

fn get_range(seq_name: String, range_size: u64, map: &mut CacheMap) -> (Vec<Range>, u64) {
    let mut ranges = map.get_mut(&seq_name).unwrap();
    let mut result = Vec::with_capacity(2);

    // sum size of already taken ranges
    let mut total = 0_u64;

    loop {
        let needed_size = range_size - total;

        // no need for another range
        if needed_size == 0 {
            return (result, needed_size);
        }

        // not enough ranges in cache
        if ranges.len() == 0 {
            return (result, needed_size);
        }


        let next_range = ranges.swap_remove(0);
        let range_size = get_range_size(&next_range);

        match needed_size.cmp(&range_size) {
            // if a range from cache is bigger than needed, we split it in two smaller ranges,
            // returning one and pushing back to cache another
            Ordering::Less => {
                total += needed_size;

                let (left, right) = split_range(next_range, needed_size).unwrap();
                result.push(left);
                ranges.push(right);
            }

            // if cache contains a range that is smaller than needed, we take it and remember
            // its size for next iteration
            Ordering::Greater => {
                total += get_range_size(&next_range);

                result.push(next_range);
            }

            Ordering::Equal => {
                total += needed_size;

                result.push(next_range)
            }

        }
    }


    fn get_range_size(r: &Range) -> u64 {
        r.end - r.begin
    }

    // left's size is size, right is the rest
    // returns none if size is bigger than one of given Range
    fn split_range(r: Range, size: u64) -> Option<(Range, Range)> {
        if get_range_size(&r) >= size {
            return None;
        }

        let left = Range {
            begin: r.begin,
            end: r.begin + size,
        };

        let right = Range {
            begin: r.begin + size + 1,
            end: r.end
        };

        return Some((left, right));
    }
}


pub(in crate::cache) enum Command {
    GetRange(GetRange),
    PutRange(PutRange),
    Stop,
}


struct GetRange {
    seq_name: String,
    range_size: u64,

    response: CacheResult,
}

struct PutRange {
    seq_name: String,
    range: Range,
}




