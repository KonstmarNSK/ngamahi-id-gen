use std::future::Future;
use crate::Range;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering};
use std::sync::atomic::Ordering::Relaxed;
use std::pin::Pin;
use atomic_waker::AtomicWaker;
use std::cell::RefCell;
use std::collections::HashMap;
use std::task::{Context, Poll};
use std::thread;
use futures::task::ArcWake;


struct CacheResultInner {
    waker: AtomicWaker,
    set: AtomicBool,

    result: AtomicPtr<(Vec<Range>, u64)>
}

#[derive(Clone)]
pub(in crate::cache) struct CacheResult(Arc<CacheResultInner>);

impl CacheResult {
    pub fn new() -> Self {
        let mut result = CacheResult(Arc::new(CacheResultInner {
            result: AtomicPtr::new(Box::into_raw(Box::new((vec![], 0)))),

            waker: AtomicWaker::new(),
            set: AtomicBool::new(false),
        }));

        return result;
    }

    pub fn set_result(&mut self, mut result: Box<(Vec<Range>, u64)>) {
        let mut inner = &mut self.0;

        println!("set result");

        inner.result.swap(Box::into_raw(result), Ordering::Release);

        inner.set.store(true, Ordering::Release);
        inner.waker.wake();

        println!("called wake")
    }
}

impl Future for CacheResult {
    type Output = Box<(Vec<Range>, u64)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Box<(Vec<Range>, u64)>> {
        let inner = &self.0;

        println!("polling");

        // quick check to avoid registration if already done.
        if inner.set.load(Ordering::Acquire) {
            let result = inner.result.load(Ordering::Acquire);
            let result = Box::new((vec![Range{ begin: 0, end: 15 }], 0)); //unsafe { Box::from_raw(result) };

            return Poll::Ready(result);
        }

        inner.waker.register(cx.waker());

        // Need to check condition **after** `register` to avoid a race
        // condition that would result in lost notifications.
        if inner.set.load(Ordering::Acquire) {
            let result = inner.result.load(Ordering::Acquire);
            let result = Box::new((vec![Range{ begin: 0, end: 15 }], 0)); //unsafe { Box::from_raw(result) };

            return Poll::Ready(result);
        } else {
            Poll::Pending
        }
    }
}