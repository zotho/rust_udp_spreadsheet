use std::net::UdpSocket;
use std::time::Duration;

use mysql::serde_json::{to_vec, from_slice};

use crate::error::Error;

pub type MyConnectorResult<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Connector {
    bind_addr: String,
    connect_addr: String,
    socket: UdpSocket,
}

impl Connector {
    pub fn new(bind_addr: &str, connect_addr: &str) -> MyConnectorResult<Connector> {
        let socket: UdpSocket = UdpSocket::bind(bind_addr)?;
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;

        Ok(Connector {
            bind_addr: bind_addr.to_owned(),
            connect_addr: connect_addr.to_owned(),
            socket
        })
    }

    pub fn bind_addr(&self) -> &str {
        self.bind_addr.as_str()
    }

    pub fn connect_addr(&self) -> &str {
        self.connect_addr.as_str()
    }

    pub fn set_bind_addr(&mut self, bind_addr: &str) -> MyConnectorResult<()> {
        Connector::new(bind_addr, self.connect_addr.as_str())
            .map(|new_socket| {
                self.socket = new_socket.socket;
                self.bind_addr = new_socket.bind_addr;
                self.connect_addr = new_socket.connect_addr;
            })
    }

    pub fn set_connect_addr(&mut self, connect_addr: &str) -> MyConnectorResult<()> {
        self.socket.connect(connect_addr.clone())?;
        self.connect_addr = connect_addr.to_owned();
        Ok(())
    }

    pub fn receive_data(&self) -> MyConnectorResult<Vec<Vec<String>>> {
        // TODO: Handle incoming data larger than the buffer
        let mut recv_buff = [0; 4800];

        match self.socket.recv_from(&mut recv_buff) {
            Ok((n, addr)) => {
                println!("{} bytes buffer from {:?}", n, addr);

                let recieved_bytes = &mut recv_buff[..n];
                
                let deserialized: Vec<Vec<String>> = from_slice(&recieved_bytes).unwrap();
                Ok(deserialized)
            }
            Err(error) => return Err(error.into())
        }
    }

    pub fn send_data(&self, data: &Vec<Vec<String>>) -> MyConnectorResult<usize> {
        let call: Vec<u8> = to_vec(&data.clone()).unwrap();
        match self.socket.send_to(&call, self.connect_addr.clone()) {
            Ok(n_bytes) => {
                if n_bytes != call.len() {
                    Err(Error::new("Sent the wrong number of bytes"))
                }
                else {
                    Ok(n_bytes)
                }
            },
            Err(error) => return Err(error.into()),
        }
    }
}

pub fn make_connector() -> MyConnectorResult<Connector> {
    let mut bind_port = 10001;
    let dest_socket = "127.0.0.1:10000";

    // Try to bind free port
    let mut raw_connector = Err(Error::new("Not initialized"));
    while bind_port < 10010 {
        raw_connector = Connector::new(format!("127.0.0.1:{}", bind_port).as_str(), dest_socket);
        if raw_connector.is_err() {
            bind_port += 1;
        } else {
            break;
        }
    }
    raw_connector
}