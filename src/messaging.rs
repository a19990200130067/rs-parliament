extern crate zmq;

use std::net::UdpSocket;
use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;
use libc::c_int;
use std::mem;

static mut CTX: Option<zmq::Context> = None;

fn get_zmq_context() -> &'static zmq::Context {
    unsafe {
        if CTX.is_none() {
            let ctx = zmq::Context::new();
            CTX = Some(ctx);
            get_zmq_context()
        } else {
            CTX.as_ref().unwrap()
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Hash, Debug)]
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

pub trait MsgRecver<Message> where
    Message: serde::Serialize + serde::de::DeserializeOwned {
    type Ctx;
    
    fn bind(addr: &Addr) -> Self;
    fn try_recv_str(&mut self) -> Option<Vec<u8>>; // non-blocking
    fn get_io_fds(&self) -> Vec<c_int>;

    fn try_recv(&mut self) -> Option<Message> {
        self.try_recv_str().map_or(None, |msg_buf| {
            serde_json::from_reader(msg_buf.as_slice()).ok()
        })
    }

    fn try_recv_timeout(&mut self, timeout_ms: i64) -> Option<Message>;
}

pub trait MsgSender<Message> where 
    Message: serde::Serialize + serde::de::DeserializeOwned {

    fn connect(addr: &Addr) -> Self;
    fn send_str(&mut self, s: &[u8]) -> Result<(), i32>;
    fn send(&mut self, msg: &Message) -> Result<(), i32> {
        match serde_json::to_string(msg) {
            Ok(s) => {
                self.send_str(s.into_bytes().as_slice())
            },
            Err(_) => Err(-1),
        }
    }
}

pub trait Messager {
    type Message: serde::Serialize + serde::de::DeserializeOwned;
    type Sender: MsgRecver<Self::Message>;
    type Recver: MsgSender<Self::Message>;
}

pub struct UdpRecver<T> {
    sock: UdpSocket,
    msg_type: PhantomData<T>,
}

impl<T> UdpRecver<T> {
}

impl<T> MsgRecver<T> for UdpRecver<T> where
    T: serde::Serialize + serde::de::DeserializeOwned {
    type Ctx = ();

    fn bind(addr: &Addr) -> Self {
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

    fn try_recv_str(&mut self) -> Option<Vec<u8>> {
        let mut v: Vec<u8> = Vec::with_capacity(1000);
        v.resize(1000, 0);
        let recv_result;
        {
            recv_result = self.sock.recv(v.as_mut_slice());
        }
        if recv_result.is_err() {
            None
        } else {
            recv_result.ok().map(|size| {
                v.truncate(size);
                v
            })
        }
    }

    fn get_io_fds(&self) -> Vec<c_int> {
        vec![self.sock.as_raw_fd()]
    }

    fn try_recv_timeout(&mut self, timeout_ms: i64) -> Option<T> {
        self.try_recv()
    }
}

pub struct UdpSender<T> {
    sock: UdpSocket,
    addr: String,
    msg_type: PhantomData<T>,
}

impl<T> UdpSender<T> {
}

impl<T> MsgSender<T> for UdpSender<T> where
    T: serde::Serialize + serde::de::DeserializeOwned {

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
                if size == s.len() { 
                    Ok(())
                } else { 
                    Err(-1) 
                },
            Err(e) => e.raw_os_error().map_or(Err(-1), |eno| Err(eno))
        }
    }
}


pub struct ZmqServer<T> {
    ctx: &'static zmq::Context,
    socket: zmq::Socket,
    msg_type: PhantomData<T>,
}

impl<T> ZmqServer<T> {
    pub fn get_sock(&self) -> &zmq::Socket {
        &self.socket
    }
}

impl<T> MsgRecver<T> for ZmqServer<T> where
    T: serde::Serialize + serde::de::DeserializeOwned {
    type Ctx = zmq::Context;

    fn bind(addr: &Addr) -> Self {
        let ctx = get_zmq_context();//zmq::Context::new();
        let sock = ctx.socket(zmq::ROUTER).expect("failed to create socket");
        let tcp_str = format!("tcp://{}:{}", addr.addr.as_str(), addr.port);
        sock.bind(tcp_str.as_str()).expect("failed to bind");
        ZmqServer {
            ctx: ctx,
            socket: sock,
            msg_type: PhantomData,
        }
    }
    
    fn try_recv_str(&mut self) -> Option<Vec<u8>> {
        self.socket.recv_multipart(0).ok().and_then(|mut msg| {
            if msg.len() == 2 {
                String::from_utf8(msg[1].clone()).ok().map(|s| {
                    println!("{} received: {}", std::process::id(), s);
                });
                Some(msg.swap_remove(1).clone())
            } else {
                None
            }
        })
    }

    fn get_io_fds(&self) -> Vec<c_int> {
        Vec::new()
    }

    fn try_recv_timeout(&mut self, timeout_ms: i64) -> Option<T> {
        return self.try_recv();
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

impl<T> MsgSender<T> for ZmqClient<T> where
    T: serde::Serialize + serde::de::DeserializeOwned {
    fn connect(addr: &Addr) -> Self {
        let ctx = get_zmq_context();//zmq::Context::new();
        let sock = ctx.socket(zmq::DEALER).expect("failed to create socket");
        let tcp_str = format!("tcp://{}:{}", addr.addr.as_str(), addr.port);
        sock.connect(tcp_str.as_str()).expect("failed to connect");
        ZmqClient {
            socket: sock,
            msg_type: PhantomData,
        }
    }

    fn send_str(&mut self, s: &[u8]) -> Result<(), i32> {
        let ret = self.socket.send(s, 0).map_err(|e| e as i32);
        std::str::from_utf8(s).ok().map(|msg| {
            println!("{} sent: {}", std::process::id(), msg);
        });
        ret
    }
}
