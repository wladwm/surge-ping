#[cfg(target_os = "linux")]
use std::ffi::CStr;
use std::io;
use std::sync::Arc;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::io::unix::AsyncFd;

#[derive(Debug, Clone)]
pub struct AsyncSocket {
    inner: Arc<AsyncFd<Socket>>,
}

impl AsyncSocket {
    #[cfg(target_os = "linux")]
    pub fn new(interface: Option<&CStr>) -> io::Result<AsyncSocket> {
        let socket = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4()))?;
        socket.bind_device(interface)?;
        socket.set_nonblocking(true)?;
        Ok(AsyncSocket {
            inner: Arc::new(AsyncFd::new(socket)?),
        })
    }

    #[cfg(not(target_os = "linux"))]
    pub fn new() -> io::Result<AsyncSocket> {
        let socket = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4()))?;
        socket.set_nonblocking(true)?;
        Ok(AsyncSocket {
            inner: Arc::new(AsyncFd::new(socket)?),
        })
    }

    pub async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.inner.readable().await?;

            match guard.try_io(|inner| inner.get_ref().recv(buf)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    pub async fn send_to(&self, buf: &mut [u8], target: &SockAddr) -> io::Result<usize> {
        loop {
            let mut guard = self.inner.writable().await?;

            match guard.try_io(|inner| inner.get_ref().send_to(buf, target)) {
                Ok(n) => return n,
                Err(_would_block) => continue,
            }
        }
    }
}
