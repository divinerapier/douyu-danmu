extern crate byteorder;

use byteorder::ByteOrder;
#[allow(unused_imports)]
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Read;
#[allow(unused_imports)]
use std::io::Write;

fn main() {
    let mut danmu = Danmu::new("openbarrage.douyutv.com:8601");
    let room_id = 5220173;
    danmu.login(room_id);
    danmu.join_group(room_id);
    danmu.keep_alive();
    danmu.run();
}

pub struct Danmu {
    conn: std::net::TcpStream,
}

impl Danmu {
    pub fn new(addr: &str) -> Danmu {
        let conn = std::net::TcpStream::connect(addr).unwrap();
        Danmu { conn }
    }

    pub fn clone(&self) -> Self {
        Danmu {
            conn: self.conn.try_clone().unwrap(),
        }
    }

    pub fn login(&mut self, room_id: i32) {
        let login_message: String = format!("type@=loginreq/roomid@={}/", room_id);
        self.write_message(&login_message);
    }

    pub fn keep_alive(&mut self) {
        let mut clone_danmu = self.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(45));
            let keep_alive_message: &str = "type@=mrkl/";
            clone_danmu.write_message(keep_alive_message);
            match clone_danmu.conn.flush() {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to flush. {}", e);
                }
            }
        });
    }

    pub fn join_group(&mut self, room_id: i32) {
        let join_group_message: String = format!("type@=joingroup/rid@={}/gid@=-9999/", room_id);
        self.write_message(&join_group_message);
    }

    pub fn run(&mut self) {
        loop {
            let data = self.read_payload();
            if data.starts_with("type@=loginres".as_bytes()) {
                continue;
            }
            if data.starts_with("type@=chatmsg".as_bytes()) {
                match self.parse_chatmessage(&data) {
                    (Some(user), Some(txt)) => {
                        let user = std::str::from_utf8(&user);
                        let txt = std::str::from_utf8(&txt);
                        match (user, txt) {
                            (Ok(user), Ok(txt)) => {
                                println!("{} {}: {}", chrono::prelude::Local::now(), user, txt);
                            }
                            (Err(euser), Err(etxt)) => println!(
                                "failed to parse user: {}, failed to parse txt: {}",
                                euser, etxt
                            ),
                            (Err(euser), _) => {
                                println!("failed to parse user: {}", euser);
                            }
                            (_, Err(etxt)) => {
                                println!("failed to parse txt: {}", etxt);
                            }
                        }
                    }
                    (_, _) => {}
                }

                continue;
            }
            if data.starts_with("type@=lgpoolsite".as_bytes()) {}
        }
    }

    fn read_payload(&mut self) -> Vec<u8> {
        let mut header: [u8; 12] = [0; 4 + 4 + 4];
        match self.conn.read(&mut header) {
            Ok(_) => {}
            Err(e) => {
                println!("failed to read from conn. {}", e);
            }
        }
        let length = LittleEndian::read_u32(&header[0..4]);
        LittleEndian::read_u16(&header[8..10]);
        let mut data: Vec<u8> = vec![0; (length - 8) as usize];
        match self.conn.read(data.as_mut_slice()) {
            Ok(_) => {}
            Err(e) => {
                println!("failed to read from conn. {}", e);
            }
        }
        data
    }

    fn write_message(&mut self, message: &str) {
        let length = message.len();
        let mut data: Vec<u8> = Vec::with_capacity(4 + 4 + 4 + length + 1);
        data.resize(4 + 4 + 4 + length + 1, 0);
        LittleEndian::write_u32(&mut data[0..4], (length + 9) as u32);
        LittleEndian::write_u32(&mut data[4..8], (length + 9) as u32);
        LittleEndian::write_u16(&mut data[8..10], 689);
        &mut data[12..12 + length].copy_from_slice(message.as_bytes());
        match self.conn.write(&data) {
            Ok(_) => {}
            Err(e) => {
                println!("failed to write to conn. {}", e);
            }
        }
    }

    fn parse_chatmessage<'a>(&self, message: &'a [u8]) -> (Option<&'a [u8]>, Option<&'a [u8]>) {
        return (
            self.get_message_field(&message, "nn@=".as_bytes()),
            self.get_message_field(&message, "txt@=".as_bytes()),
        );
    }

    fn get_message_field<'a>(&self, message: &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
        match self.find_subsequence(&message, key) {
            Some(start) => match self.find_subsequence(&message[start..], "/".as_bytes()) {
                Some(end) => {
                    return Some(&message[start + key.len()..start + end]);
                }
                None => return Some(&message[start + key.len()..]),
            },
            None => None,
        }
    }

    fn find_subsequence(&self, haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }
}
