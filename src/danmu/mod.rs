use std::io::{Error as IOError, Read, Write};

use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug)]
struct Error {
    message: String,
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Error {
        Error {
            message: format!("{:?}", e),
        }
    }
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
        let mut clone_danmu = self.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let join_group_message: String =
                format!("type@=joingroup/rid@={}/gid@=-9999/", room_id);
            clone_danmu.write_message(&join_group_message);
        });
    }

    pub fn run(&mut self) {
        loop {
            let data = self.read_payload();
            if data.is_err() {
                println!(
                    "{} {:?}",
                    chrono::prelude::Local::now(),
                    data.err().unwrap()
                );
                return;
            }
            let data: Vec<u8> = data.unwrap();
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
            } else {
                continue;
            }
        }
    }

    fn read_payload(&mut self) -> Result<Vec<u8>, Error> {
        let mut header: [u8; 12] = [0; 4 + 4 + 4];
        self.conn.read_exact(&mut header)?;
        let length = LittleEndian::read_u32(&header[0..4]);
        let length2 = LittleEndian::read_u32(&header[4..8]);
        let message_type = LittleEndian::read_u16(&header[8..10]);
        if length != length2 || length <= 0 || message_type != 690 {
            return Err(Error {
                message: format!(
                    "invalid header. length: {} length2: {}, message type: {}",
                    length, length2, message_type
                ),
            });
        }
        let mut data: Vec<u8> = vec![0; (length - 8) as usize];
        self.conn.read_exact(data.as_mut_slice())?;
        return Ok(data);
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
