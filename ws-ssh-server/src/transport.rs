use bytes::BytesMut;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub struct WsTransport {
    ws: WebSocketStream<TcpStream>,
    read_buf: BytesMut,
}

impl WsTransport {
    pub fn new(ws: WebSocketStream<TcpStream>) -> Self {
        Self {
            ws,
            read_buf: BytesMut::new(),
        }
    }
}

impl AsyncRead for WsTransport {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if !self.read_buf.is_empty() {
            let len = std::cmp::min(buf.remaining(), self.read_buf.len());
            buf.put_slice(&self.read_buf[..len]);
            let _ = self.read_buf.split_to(len);
            return Poll::Ready(Ok(()));
        }

        loop {
            match Stream::poll_next(Pin::new(&mut self.ws), cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e)))
                }
                Poll::Ready(Some(Ok(msg))) => match msg {
                    Message::Binary(data) => {
                        let len = std::cmp::min(buf.remaining(), data.len());
                        buf.put_slice(&data[..len]);
                        if len < data.len() {
                            self.read_buf.extend_from_slice(&data[len..]);
                        }
                        return Poll::Ready(Ok(()));
                    }
                    Message::Ping(payload) => {
                        let pong = Message::Pong(payload);
                        let _ = Sink::start_send(Pin::new(&mut self.ws), pong);
                        continue;
                    }
                    Message::Close(_) => return Poll::Ready(Ok(())),
                    _ => continue,
                },
            }
        }
    }
}

impl AsyncWrite for WsTransport {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match Sink::poll_ready(Pin::new(&mut self.ws), cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(e)) => {
                return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e)))
            }
            Poll::Ready(Ok(())) => {}
        }

        let msg = Message::Binary(buf.to_vec().into());
        match Sink::start_send(Pin::new(&mut self.ws), msg) {
            Ok(()) => Poll::Ready(Ok(buf.len())),
            Err(e) => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e))),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Sink::poll_flush(Pin::new(&mut self.ws), cx)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Sink::poll_close(Pin::new(&mut self.ws), cx)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
