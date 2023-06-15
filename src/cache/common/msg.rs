use crate::cache::common::waker::{Flag, GetResult};
use crate::range::Range;

// messages that cache thread receives from channel
pub enum Msg {
    GetFromCache(MsgGet),
    PutToCache(MsgPut),
    Stop,
}

pub struct MsgGet {
    pub key: String,
    pub range_size: u64,

    pub result: GetResult,
}

pub struct MsgPut {
    pub key: String,
    pub value: Range,

    pub result: Flag,
}
