#[path = "support/ws_echo.rs"]
mod ws_echo;

#[cfg(feature = "websocket")]
#[test]
fn local_echo_accepts_ping() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;

    let server = ws_echo::EchoServer::start();
    let url = server.url();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let (mut ws, _) = connect_async(url).await.expect("connect");
        ws.send(Message::Ping(vec![].into())).await.unwrap();
        let msg = ws.next().await.expect("frame").expect("ok");
        assert!(matches!(msg, Message::Pong(_)));
    });
}
