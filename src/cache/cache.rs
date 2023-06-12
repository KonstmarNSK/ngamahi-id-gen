use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use crate::cache::client::CacheClient;
use crate::cache::Msg;
use crate::Range;


type CacheMap = HashMap<String, Vec<Range>>;


pub struct Cache {
    values: CacheMap,
    channel: Receiver<Msg>
}


impl Cache {
    pub fn new() -> CacheClient {
        let (s, r) = mpsc::channel::<Msg>();

        let client = CacheClient{ channel: s };
        let mut cache = Cache { values: Default::default(), channel: r };

        thread::spawn(move || {
            let mut c = cache;

            for msg in c.channel {
                match msg {
                    Msg::GetFromCache(g) => {
                        println!("Getting value 2");

                        let result = get_range(g.key, g.range_size, &mut c.values);

                        println!("Got: {:?}", result);

                        g.result.signal(result);
                    },

                    Msg::PutToCache(p) => {
                        println!("Putting value 2");

                        store_range(p.key,  p.value, &mut c.values);

                        println!("Now cache is {:?}", &c.values);
                        p.result.signal();
                    },

                    Msg::Stop => break
                }
            }
        });

        return client;
    }
}




fn store_range(seq_name: String, range: Range, map: &mut CacheMap) -> () {
    let mut default = Vec::<Range>::new();
    let mut ranges = map.get_mut(&seq_name);

    if let Some(ranges) = ranges {
        ranges.push(range);
    } else {
        default.push(range);
        map.insert(seq_name, default);
    }
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
        println!("Must split range {:?}. Size is {}", &r, size);
        println!("The range size is {}", get_range_size(&r));

        if get_range_size(&r) <= size {
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