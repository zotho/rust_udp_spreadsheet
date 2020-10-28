use std::net::UdpSocket;
use std::time::Duration;

use mysql::serde_json::{to_vec, from_slice};

use crate::error::Error;

type MyConnectorResult<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct SocketClient {
    bind_addr: String,
    connect_addr: String,
    socket: UdpSocket,
}

impl SocketClient {
    pub fn new(bind_addr: &str, connect_addr: &str) -> MyConnectorResult<SocketClient> {
        let socket: UdpSocket = UdpSocket::bind(bind_addr)?;
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;
        // socket.set_broadcast(true)?;

        Ok(SocketClient {
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
        match SocketClient::new(bind_addr, self.connect_addr.as_str()) {
            Ok(new_socket) => {
                self.socket = new_socket.socket;
                self.bind_addr = new_socket.bind_addr;
                self.connect_addr = new_socket.connect_addr;
                Ok(())
            }
            Err(error) => Err(error)
        }
    }

    pub fn set_connect_addr(&mut self, connect_addr: &str) -> MyConnectorResult<()> {
        self.socket.connect(connect_addr.clone())?;
        self.connect_addr = connect_addr.to_owned();
        Ok(())
    }

    pub fn receive_data(&self) -> MyConnectorResult<Vec<Vec<String>>> {
        let mut recv_data: Vec<u8> = Vec::new();
        let mut recv_buff = [0; 4800];

        loop {
            let result = self.socket.recv_from(&mut recv_buff);
            match result {
                Ok((n, addr)) => {
                    println!("{} bytes buffer from {:?}", n, addr);

                    let recieved_bytes = &mut recv_buff[..n];
                    recv_data.extend_from_slice(recieved_bytes);
                    if n != recv_buff.len() {
                        break;
                    }
                }
                Err(error) => return Err(error.into())
            }
        }

        println!("{} bytes response", recv_data.len());
        let deserialized: Vec<Vec<String>> = from_slice(&recv_data).unwrap();
            Ok(deserialized)
        
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