use std::future::Future;

use futures_channel::oneshot;

pub trait Service {
    type Request;
    type Response;

    fn execute_async(
        &self,
        seq_id: usize,
        abort_rx: oneshot::Receiver<()>,
        request: Self::Request,
    ) -> impl Future<Output = (usize, Option<Self::Response>)>;

    fn execute(
        &self,
        seq_id: usize,
        request: Self::Request,
    ) -> (usize, Option<Self::Response>);

    fn is_async_request(request: &Self::Request) -> bool;
}

pub type MultiThreadSpawner = Box<dyn Fn(dyn Future<Output = ()> + Send + 'static) -> ()>;

pub type ServiceConfiguration<T> = (T, Option<MultiThreadSpawner>);