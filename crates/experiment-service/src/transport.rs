//! Bounded connection ownership, header deadlines and slow-reader write deadlines.
use axum::Router;
use hyper_util::{
    rt::{TokioIo, TokioTimer},
    service::TowerToHyperService,
};
use std::{
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf},
    net::{TcpListener, TcpStream},
    sync::{Semaphore, oneshot, watch},
    task::JoinSet,
    time::{Instant, Sleep},
};

pub(super) async fn serve(
    listener: TcpListener,
    router: Router,
    mut stop: oneshot::Receiver<()>,
) -> io::Result<()> {
    let slots = std::sync::Arc::new(Semaphore::new(96));
    let (shutdown, _) = watch::channel(false);
    let mut tasks = JoinSet::new();
    loop {
        tokio::select! {
         _=&mut stop=>break,
         result=tasks.join_next(),if !tasks.is_empty()=>{if let Some(Err(error))=result{return Err(io::Error::other(error));}},
         accepted=listener.accept()=>{
          let (mut socket,_)=accepted?;let Ok(permit)=slots.clone().try_acquire_owned()else{let _=tokio::time::timeout(Duration::from_secs(1),socket.write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")).await;continue;};
          let router=router.clone();let mut shutdown=shutdown.subscribe();
          tasks.spawn(async move{let _permit=permit;let io=TokioIo::new(WriteDeadline::new(socket));let mut builder=hyper::server::conn::http1::Builder::new();builder.timer(TokioTimer::new()).header_read_timeout(Duration::from_secs(5)).max_buf_size(8192).keep_alive(false);let connection=builder.serve_connection(io,TowerToHyperService::new(router));tokio::pin!(connection);
           tokio::select!{_=&mut connection=>{},_ = shutdown.changed()=>{connection.as_mut().graceful_shutdown();let _=tokio::time::timeout(Duration::from_secs(4),connection).await;}}
          });
         }
        }
    }
    drop(listener);
    let _ = shutdown.send(true);
    if tokio::time::timeout(Duration::from_secs(4), async {
        while let Some(result) = tasks.join_next().await {
            result.map_err(io::Error::other)?;
        }
        Ok::<_, io::Error>(())
    })
    .await
    .is_err()
    {
        tasks.abort_all();
        while tasks.join_next().await.is_some() {}
    }
    Ok(())
}
struct WriteDeadline {
    stream: TcpStream,
    timer: Pin<Box<Sleep>>,
}
impl WriteDeadline {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            timer: Box::pin(tokio::time::sleep(Duration::from_secs(5))),
        }
    }
    fn reset(&mut self) {
        self.timer
            .as_mut()
            .reset(Instant::now() + Duration::from_secs(5));
    }
    fn stalled(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<usize>> {
        if self.timer.as_mut().poll(cx).is_ready() {
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "HTTP write idle deadline",
            )))
        } else {
            Poll::Pending
        }
    }
}
impl AsyncRead for WriteDeadline {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buffer)
    }
}
impl AsyncWrite for WriteDeadline {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        match Pin::new(&mut self.stream).poll_write(cx, bytes) {
            Poll::Ready(result) => {
                self.reset();
                Poll::Ready(result)
            }
            Poll::Pending => self.stalled(cx),
        }
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}
