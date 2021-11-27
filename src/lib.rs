use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use parking_lot::Mutex;

pub mod rt;

pub struct Gem {

}

pub struct GemContext<T> {
    item: Arc<Mutex<Option<T>>>,
}

pub struct Yielding(bool);

impl<T> GemContext<T> {
    pub fn new() -> Self {
        Self {
            item: Arc::new(Default::default())
        }
    }
    
    pub fn r#yield(&self, t: T) -> Yielding {
        *self.item.lock() = Some(t);
        Yielding(true)
    }
}

impl Future for Yielding {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0 {
            true => {
                self.get_mut().0 = false;
                Poll::Pending
            },
            false => Poll::Ready(())
        }
    }
}
