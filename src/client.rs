use bytes::BytesMut;
use futures::Future;
use std::io;
use std::net::SocketAddr;
use tokio_executor::{DefaultExecutor, Executor, SpawnError};
use tokio_tcp::TcpStream;

use protocol::{Connection, ConnectionHandle, EmptyService, OperationError};
use request::Request;
use response::Response;

pub struct Client {
    handle: ConnectionHandle,
}

impl Client {
    pub fn connect(address: SocketAddr) -> impl Future<Item = Client, Error = io::Error> {
        TcpStream::connect(&address).and_then(|tcp_stream| {
            let mut executor = DefaultExecutor::current();
            let (connection, handler, handle) =
                Connection::new::<_, EmptyService>(tcp_stream, None);

            executor.spawn(Box::new(connection)).unwrap();

            if let Some(handler) = handler {
                executor.spawn(Box::new(handler)).unwrap();
            }

            Ok(Client { handle })
        })
    }

    pub fn send_request<R, B>(
        &mut self,
        request: R,
    ) -> impl Future<Item = Response<BytesMut>, Error = OperationError>
    where
        R: Into<Request<B>>,
        B: AsRef<[u8]>,
    {
        self.handle.send_request(request)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bounds() {
        fn check_bounds<T: Send + Send>() {}

        check_bounds::<Client>();
    }
}
