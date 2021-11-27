use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use futures_task::ArcWake;
use parking_lot::{Condvar, Mutex, RawMutex, RwLock};
use parking_lot::lock_api::MutexGuard;
use crate::GemContext;

pub struct EvaluatorRuntime<T, F: Future<Output = ()>> {
    waker: Arc<RtWaker>,
    fut: Pin<Box<F>>,
    ctx: GemContext<T>
}

struct RtWaker {
    inner_waker: RwLock<Option<Waker>>,
    condvar: Condvar,
    flag: Mutex<()>,
}

impl<T, F: Future<Output = ()>> EvaluatorRuntime<T, F> {
    pub fn new(f: F, ctx: &GemContext<T>) -> Self {
        Self {
            waker: Arc::new(RtWaker {
                inner_waker: Default::default(),
                condvar: Default::default(),
                flag: Default::default(),
            }),
            fut: Box::pin(f),
            ctx: GemContext {
                item: ctx.item.clone()
            }
        }
    }
}

impl<T, F: Future<Output = ()>> Iterator for EvaluatorRuntime<T, F> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let waker = futures_task::waker(self.waker.clone());

        let mut redo = self.waker.flag.lock();
        loop {
            let mut pin = self.fut.as_mut();
            match pin.poll(&mut Context::from_waker(&waker)) {
                Poll::Ready(_) => return None,
                Poll::Pending => {
                    let mut lock = self.ctx.item.lock();
                    match lock.take() {
                        Some(t) => return Some(t),
                        None => (),
                    }
                }
            }
            self.waker.condvar.wait(&mut redo);
        }
    }
}

impl ArcWake for RtWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let guard = arc_self.inner_waker.read();
        if let Some(inner) = &*guard {
            inner.wake_by_ref();
        }
        arc_self.condvar.notify_all();
    }
}