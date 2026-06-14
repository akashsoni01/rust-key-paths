//! Local WebSocket echo server for subscription integration tests.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub struct EchoServer {
    url: &'static str,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl EchoServer {
    pub fn start() -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind echo ws");
        listener.set_nonblocking(true).ok();
        let port = listener.local_addr().expect("addr").port();
        let url: &'static str =
            Box::leak(format!("ws://127.0.0.1:{port}").into_boxed_str());
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_thread = shutdown.clone();

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("echo tokio");
            rt.block_on(async move {
                use futures_util::{SinkExt, StreamExt};
                use tokio::net::TcpListener;
                use tokio_tungstenite::accept_async;
                use tokio_tungstenite::tungstenite::Message;

                let listener = TcpListener::from_std(listener).expect("async listener");
                loop {
                    if shutdown_thread.load(Ordering::Relaxed) {
                        break;
                    }
                    let Ok((stream, _)) =
                        tokio::time::timeout(std::time::Duration::from_millis(50), listener.accept())
                            .await
                            .map_err(|_| ())
                            .and_then(|r| r.map_err(|_| ()))
                    else {
                        continue;
                    };
                    let shutdown_conn = shutdown_thread.clone();
                    tokio::spawn(async move {
                        let Ok(mut ws) = accept_async(stream).await else {
                            return;
                        };
                        while !shutdown_conn.load(Ordering::Relaxed) {
                            match ws.next().await {
                                Some(Ok(Message::Text(text))) => {
                                    let _ = ws.send(Message::Text(text)).await;
                                }
                                Some(Ok(Message::Binary(bin))) => {
                                    let _ = ws.send(Message::Binary(bin)).await;
                                }
                                Some(Ok(Message::Ping(payload))) => {
                                    let _ = ws.send(Message::Pong(payload)).await;
                                }
                                Some(Ok(Message::Close(_))) | None => break,
                                Some(Err(_)) => break,
                                _ => {}
                            }
                        }
                    });
                }
            });
        });

        Self {
            url,
            shutdown,
            handle: Some(handle),
        }
    }

    pub fn url(&self) -> &'static str {
        self.url
    }
}

impl Drop for EchoServer {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
