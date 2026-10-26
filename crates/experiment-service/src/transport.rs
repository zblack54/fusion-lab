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
    let serve_result = loop {
        tokio::select! {
            _ = &mut stop => break Ok(()),
            result = tasks.join_next(), if !tasks.is_empty() => {
                if let Some(Err(error)) = result {
                    break Err(io::Error::other(error));
                }
            },
            accepted = listener.accept() => {
                let (mut socket, _) = match accepted {
                    Ok(accepted) => accepted,
                    Err(error) => break Err(error),
                };
                let Ok(permit) = slots.clone().try_acquire_owned() else {
                    // Overload replies are best effort; no admitted connection owns work.
                    let _ = tokio::time::timeout(
                        Duration::from_secs(1),
                        socket.write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"),
                    ).await;
                    continue;
                };
                let router = router.clone();
                let mut shutdown = shutdown.subscribe();
                tasks.spawn(async move {
                    let _permit = permit;
                    let io = TokioIo::new(WriteDeadline::new(socket));
                    let mut builder = hyper::server::conn::http1::Builder::new();
                    builder.timer(TokioTimer::new())
                        .header_read_timeout(Duration::from_secs(5))
                        .max_buf_size(8192)
                        .keep_alive(false);
                    let connection = builder.serve_connection(io, TowerToHyperService::new(router));
                    tokio::pin!(connection);
                    tokio::select! {
                        _ = &mut connection => {},
                        _ = shutdown.changed() => {
                            connection.as_mut().graceful_shutdown();
                            // The outer drain owns the shared shutdown deadline and observes aborts.
                            let _ = connection.await;
                        }
                    }
                });
            }
        }
    };
    drop(listener);
    // No receivers means no remaining connection needs the signal.
    let _ = shutdown.send(true);
    let drain_result = drain_connections(&mut tasks, Duration::from_secs(4)).await;
    serve_result.and(drain_result)
}

/// Observe every connection even after one fails; a deadline never abandons a task.
async fn drain_connections(tasks: &mut JoinSet<()>, deadline: Duration) -> io::Result<()> {
    let mut failure = None;
    let timed_out = tokio::time::timeout(deadline, async {
        while let Some(result) = tasks.join_next().await {
            if let Err(error) = result {
                failure.get_or_insert_with(|| io::Error::other(error));
            }
        }
    })
    .await
    .is_err();
    if timed_out {
        tasks.abort_all();
        while let Some(result) = tasks.join_next().await {
            if let Err(error) = result
                && !error.is_cancelled()
            {
                failure.get_or_insert_with(|| io::Error::other(error));
            }
        }
        failure.get_or_insert_with(|| {
            io::Error::new(io::ErrorKind::TimedOut, "connection shutdown deadline")
        });
    }
    match failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
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

#[cfg(test)]
mod tests {
    use super::drain_connections;
    use std::{
        io,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        time::Duration,
    };
    use tokio::task::JoinSet;

    struct Dropped(Arc<AtomicBool>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn shutdown_reports_task_panic_and_observes_remaining_connections() {
        let mut tasks = JoinSet::new();
        let joined = Arc::new(AtomicBool::new(false));
        tasks.spawn(async {
            panic!("injected connection task failure");
        });
        let observed = joined.clone();
        tasks.spawn(async move {
            observed.store(true, Ordering::SeqCst);
        });
        let error = drain_connections(&mut tasks, Duration::from_secs(1))
            .await
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(joined.load(Ordering::SeqCst));
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn shutdown_deadline_aborts_and_observes_pending_connection() {
        let mut tasks = JoinSet::new();
        let dropped = Arc::new(AtomicBool::new(false));
        let guard = Dropped(dropped.clone());
        let (started, ready) = tokio::sync::oneshot::channel();
        tasks.spawn(async move {
            let _guard = guard;
            started.send(()).unwrap();
            std::future::pending::<()>().await;
        });
        ready.await.unwrap();
        let error = drain_connections(&mut tasks, Duration::from_millis(1))
            .await
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(dropped.load(Ordering::SeqCst));
        assert!(tasks.is_empty());
    }
}
