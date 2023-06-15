use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering};
use std::sync::atomic::Ordering::Relaxed;
use std::task::{Context, Poll};
use atomic_waker::AtomicWaker;
use crate::range::Range;

// A response from cache


struct Inner {
    waker: AtomicWaker,
    set: AtomicBool,
}

#[derive(Clone)]
pub struct Flag(Arc<Inner>);

impl Flag {
    pub fn new() -> Self {
        Flag(Arc::new(Inner {
            waker: AtomicWaker::new(),
            set: AtomicBool::new(false),
        }))
    }

    pub fn signal(&self) {
        self.0.set.store(true, Relaxed);
        self.0.waker.wake();
    }
}

impl Future for Flag {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // quick check to avoid registration if already done.
        if self.0.set.load(Relaxed) {
            return Poll::Ready(());
        }

        self.0.waker.register(cx.waker());

        // Need to check condition **after** `register` to avoid a race
        // condition that would result in lost notifications.
        if self.0.set.load(Relaxed) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}







struct GetResultInner {
    waker: AtomicWaker,
    set: AtomicBool,

    result: AtomicPtr<(Vec<Range>, u64)>,
}

#[derive(Clone)]
pub struct GetResult(Arc<GetResultInner>);

impl GetResult {
    pub fn new() -> Self {
        GetResult(Arc::new(GetResultInner {
            waker: AtomicWaker::new(),
            set: AtomicBool::new(false),
            result: AtomicPtr::new(std::ptr::null_mut())
        }))
    }

    pub fn signal(&self, result: (Vec<Range>, u64)) {
        let result = Box::into_raw(Box::new(result));

        self.0.result.store(result, Ordering::Release);
        self.0.set.store(true, Relaxed);
        self.0.waker.wake();
    }
}

impl Future for GetResult {
    type Output = Box<(Vec<Range>, u64)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // quick check to avoid registration if already done.
        if self.0.set.load(Relaxed) {
            let result = self.0.result.load(Ordering::Acquire);
            let result = unsafe {Box::from_raw(result) };
            return Poll::Ready(result);
        }

        self.0.waker.register(cx.waker());

        // Need to check condition **after** `register` to avoid a race
        // condition that would result in lost notifications.
        if self.0.set.load(Relaxed) {
            let result = self.0.result.load(Ordering::Acquire);
            let result = unsafe {Box::from_raw(result) };
            return Poll::Ready(result);
        } else {
            Poll::Pending
        }
    }
}