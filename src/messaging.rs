use std::net::UdpSocket;
use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;
use libc::c_int;

#[derive(Clone, Copy)]
pub struct Addr<'a> {
    pub addr: &'a str,
    pub port: u16,
}

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
        UdpRecver {
            sock: UdpSocket::bind(format!("{}:{}", addr.addr, addr.port)).expect("failed to bind"),
            msg_type: PhantomData,
        }
    }
}

impl<T> MsgRecver for UdpRecver<T> where
    T: serde::Serialize + serde::de::DeserializeOwned {
    type Message = T;
    fn try_recv_str(&mut self) -> Option<Vec<u8>> {
        let mut v: Vec<u8> = Vec::with_capacity(1000);
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
    pub fn connect(addr: Addr) -> Self {
        UdpSender {
            sock: UdpSocket::bind("0.0.0.0:0").expect("failed to create socket"),
            addr: format!("{}:{}", addr.addr, addr.port),
            msg_type: PhantomData,
        }
    }
}

impl<T> MsgSender for UdpSender<T> where
    T: serde::Serialize + serde::de::DeserializeOwned {
    type Message = T;
    fn send_str(&mut self, s: &[u8]) -> Result<(), i32> {
        assert!(s.len() < 1000, "message too large: {}", s.len());
        match self.sock.send_to(s, self.addr.clone()) {
            Ok(size) => 
                if size == s.len() { Ok(()) } else { Err(-1) },
            Err(e) => e.raw_os_error().map_or(Err(-1), |eno| Err(eno))
        }
    }
}
