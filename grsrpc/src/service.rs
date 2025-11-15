use std::future::Future;

use futures_channel::oneshot;

pub trait Service {
    type Request;
    type Response;

    fn execute(
        &self,
        seq_id: usize,
        abort_rx: oneshot::Receiver<()>,
        request: Self::Request,
    ) -> impl Future<Output = (usize, Option<Self::Response>)>;
}