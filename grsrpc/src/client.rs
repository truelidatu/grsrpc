use std::{
    cell::RefCell,
    collections::HashMap,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use futures_channel::oneshot;
use futures_core::{
    future::{BoxFuture, LocalBoxFuture},
    Future,
};
use futures_util::{
    future::{self, Shared},
    FutureExt,
};

use crate::task_set::TaskSetHandle;

#[doc(hidden)]
pub trait Client {
    type Request;
    type Response;
}

#[doc(hidden)]
pub type CallbackMap<Response> = HashMap<usize, oneshot::Sender<Response>>;

#[doc(hidden)]
pub type Configuration<Request, Response> = (
    Rc<RefCell<CallbackMap<Response>>>,
    futures_channel::mpsc::UnboundedSender<(usize, Request)>,
    futures_channel::mpsc::UnboundedSender<usize>,
    TaskSetHandle<()>, // Rc<dyn Fn(usize, Request) -> Vec<u8>>,
                       // Rc<dyn FnMut(usize)>,
);

#[must_use = "Either await this future or remove the return type from the RPC method"]
pub struct RequestFuture<T: 'static> {
    result: LocalBoxFuture<'static, T>,
    abort: Pin<Box<RequestAbort>>,
}

impl<T> RequestFuture<T> {
    pub fn new(result: impl Future<Output = T> + 'static, abort: Box<dyn Fn()>) -> Self {
        Self {
            result: result.boxed_local(),
            abort: Box::pin(RequestAbort {
                active: true,
                abort: Some(abort),
            }),
        }
    }
}

struct RequestAbort {
    active: bool,
    abort: Option<Box<dyn FnOnce()>>,
}

impl Drop for RequestAbort {
    fn drop(&mut self) {
        if self.active {
            (self.abort.take().unwrap())();
        }
    }
}

impl<T> Future for RequestFuture<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let poll_result = self.as_mut().result.poll_unpin(cx);
        if matches!(poll_result, Poll::Ready(_)) {
            self.as_mut().abort.active = false;
        }
        poll_result
    }
}

pub struct MultiThreadRequestFuture<T: Send + 'static> {
    result: BoxFuture<'static, T>,
    abort: Pin<Box<RequestAbort>>,
}

impl<T> MultiThreadRequestFuture<T>
where
    T: Send + 'static,
{
    pub fn new(result: impl Future<Output = T> + Send + 'static, abort: Box<dyn FnOnce()>) -> Self {
        Self {
            result: result.boxed(),
            abort: Box::pin(RequestAbort {
                active: true,
                abort: Some(abort),
            }),
        }
    }
}

impl<T> Future for MultiThreadRequestFuture<T>
where
    T: Send + 'static,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let poll_result = self.as_mut().result.poll_unpin(cx);
        if matches!(poll_result, Poll::Ready(_)) {
            self.as_mut().abort.active = false;
        }
        poll_result
    }
}
