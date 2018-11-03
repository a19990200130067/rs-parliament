extern crate zmq;

use std::net::UdpSocket;
use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;
use libc::c_int;
use std::mem;

#[derive(Clone, Serialize, Deserialize, Hash)]
pub struct Addr {
    pub addr: String,
    pub port: u16,
}

impl Addr {
    pub fn new(a: &str, p: u16) -> Self {
        Addr {
            addr: a.to_string(),
            port: p,
        }
    }
}

impl PartialEq for Addr {
    fn eq(&self, other: &Addr) -> bool {
        self.addr == other.addr && self.port == other.port
    }
}

impl Eq for Addr {}

pub trait MsgRecver {
    type Message: serde::Serialize + serde::de::DeserializeOwned;
    
    fn try_recv_str(&mut self) -> Option<Vec<u8>>; // non-blocking
    fn get_io_fds(&self) -> Vec<c_int>;

    fn try_recv(&mut self) -> Option<Self::Message> {
        self.try_recv_str().map_or(None, |msg_buf| {
            serde_json::from_reader(msg_buf.as_slice()).ok()
        })
    }
}

pub trait MsgSender {
    type Message: serde::Serialize + serde::de::DeserializeOwned;

    fn connect(addr: &Addr) -> Self;
    fn send_str(&mut self, s: &[u8]) -> Result<(), i32>;
    fn send(&mut self, msg: &Self::Message) -> Result<(), i32> {
        match serde_json::to_string(msg) {
            Ok(s) => {
                self.send_str(s.into_bytes().as_slice())
            },
            Err(_) => Err(-1),
        }
    }
}


pub struct UdpRecver<T> {
    sock: UdpSocket,
    msg_type: PhantomData<T>,
}

impl<T> UdpRecver<T> {
    pub fn bind(addr: Addr) -> Self {
        let sock = UdpSocket::bind(format!("{}:{}", addr.addr, addr.port)).expect("failed to bind");
        sock.set_nonblocking(true).expect("failed to set socket non_blocking");
        unsafe {
            let optval: libc::c_int = 1;
            let ret = libc::setsockopt(sock.as_raw_fd(),
                                       libc::SOL_SOCKET,
                                       libc::SO_REUSEPORT,
                                       &optval as *const _ as *const libc::c_void,
                                       mem::size_of_val(&optval) as libc::socklen_t);
            assert!(ret == 0, "failed to setsockopt");
        }
        UdpRecver {
            sock: sock,
            msg_type: PhantomData,
        }
    }
}

impl<T> MsgRecver for UdpRecver<T> where
    T: serde::Serialize + serde::de::DeserializeOwned {
    type Message = T;
    fn try_recv_str(&mut self) -> Option<Vec<u8>> {
        let mut v: Vec<u8> = Vec::with_capacity(1000);
        v.resize(1000, 0);
        let recv_result;
        {
            recv_result = self.sock.recv_from(v.as_mut_slice());
        }
        recv_result.ok().map(|(size, _addr)| {
            v.truncate(size);
            v
        })
    }

    fn get_io_fds(&self) -> Vec<c_int> {
        vec![self.sock.as_raw_fd()]
    }
}

pub struct UdpSender<T> {
    sock: UdpSocket,
    addr: String,
    msg_type: PhantomData<T>,
}

impl<T> UdpSender<T> {
}

impl<T> MsgSender for UdpSender<T> where
    T: serde::Serialize + serde::de::DeserializeOwned {
    type Message = T;

    fn connect(addr: &Addr) -> Self {
        let sock = UdpSocket::bind("0.0.0.0:0").expect("failed to create socket");
        sock.set_nonblocking(true).expect("failed to set socket non_blocking");
        unsafe {
            let optval: libc::c_int = 1;
            let ret = libc::setsockopt(sock.as_raw_fd(),
                                       libc::SOL_SOCKET,
                                       libc::SO_REUSEPORT,
                                       &optval as *const _ as *const libc::c_void,
                                       mem::size_of_val(&optval) as libc::socklen_t);
            assert!(ret == 0, "failed to setsockopt");
        }
        UdpSender {
            sock: sock,
            addr: format!("{}:{}", addr.addr, addr.port),
            msg_type: PhantomData,
        }
    }

    fn send_str(&mut self, s: &[u8]) -> Result<(), i32> {
        assert!(s.len() < 1000, "message too large: {}", s.len());
        match self.sock.send_to(s, self.addr.clone()) {
            Ok(size) => 
                if size == s.len() { Ok(()) } else { Err(-1) },
            Err(e) => e.raw_os_error().map_or(Err(-1), |eno| Err(eno))
        }
    }
}


pub struct ZmqServer<'a, T> {
    _ctx: &'a zmq::Context,
    socket: zmq::Socket,
    msg_type: PhantomData<T>,
}

impl<'a, T> ZmqServer<'a, T> {
    pub fn bind(ctx: &'a zmq::Context, addr: &Addr) -> Self {
        let sock = ctx.socket(zmq::ROUTER).expect("failed to create socket");
        let tcp_str = format!("tcp://{}:{}", addr.addr.as_str(), addr.port);
        sock.bind(tcp_str.as_str()).expect("failed to bind");
        ZmqServer {
            _ctx: ctx,
            socket: sock,
            msg_type: PhantomData,
        }
    }

    pub fn get_sock(&self) -> &zmq::Socket {
        &self.socket
    }
}

impl<'a, T> ZmqServer<'a, T> where
    T: serde::Serialize + serde::de::DeserializeOwned {
    pub fn try_recv_timeout(&mut self, timeout_ms: i64) -> Option<T> {
        let r;
        {
            r = self.socket.poll(zmq::POLLIN, timeout_ms);
        }
        if r.ok().map_or(false, |_| true) {
            self.try_recv()
        } else {
            None
        }
    }
}

impl<'a, T> MsgRecver for ZmqServer<'a, T> where
    T: serde::Serialize + serde::de::DeserializeOwned {
    type Message = T;
    
    fn try_recv_str(&mut self) -> Option<Vec<u8>> {
        self.socket.recv_multipart(zmq::DONTWAIT).ok().and_then(|mut msg| {
            if msg.len() == 3 {
                Some(msg.swap_remove(2).clone())
            } else {
                None
            }
        })
    }

    fn get_io_fds(&self) -> Vec<c_int> {
        Vec::new()
    }
}

pub struct ZmqClient<T> {
    socket: zmq::Socket,
    msg_type: PhantomData<T>,
}

impl<T> ZmqClient<T> {
    pub fn get_sock(&self) -> &zmq::Socket {
        &self.socket
    }
}

impl<T> MsgSender for ZmqClient<T> where
    T: serde::Serialize + serde::de::DeserializeOwned {
    type Message = T;
    fn connect(addr: &Addr) -> Self {
        let ctx = zmq::Context::new();
        let sock = ctx.socket(zmq::ROUTER).expect("failed to create socket");
        let tcp_str = format!("tcp://{}:{}", addr.addr.as_str(), addr.port);
        sock.connect(tcp_str.as_str()).expect("failed to bind");
        ZmqClient {
            socket: sock,
            msg_type: PhantomData,
        }
    }

    fn send_str(&mut self, s: &[u8]) -> Result<(), i32> {
        self.socket.send(s, zmq::DONTWAIT).map_err(|e| e as i32)
    }
}
