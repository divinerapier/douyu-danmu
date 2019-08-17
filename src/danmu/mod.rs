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
            // if data.starts_with("type@=loginres".as_bytes()) {
            //     continue;
            // }
            // if data.starts_with("type@=lgpoolsite".as_bytes()) {
            //     // Ok("type@=lgpoolsite/zone@=1/deadsec@=17070/piv@=giftid@AA=2095@AScuserpg@AA=925922@AScownerpg@AA=232302@AScurps@AA=0@ASnuserpg@AA=23062@ASnownerpg@AA=5783@AS@Sgiftid@AA=2096@AScuserpg@AA=2082326@AScownerpg@AA=470965@AScurps@AA=0@ASnuserpg@AA=2636@ASnownerpg@AA=659@AS@Sgiftid@AA=2097@AScuserpg@AA=1099805@AScownerpg@AA=274953@AScurps@AA=0@ASnuserpg@AA=1051@ASnownerpg@AA=262@AS@S/\u{0}")
            //     continue;
            // }
            // if data.starts_with("type@=noble_num_info".as_bytes()) {
            //     continue;
            // }
            // if data.starts_with("type@=shrn".as_bytes()) {
            //     // Ok("type@=shrn/rid@=5981137/srid@=5981137/uid@=243757184/cate_id@=201/ri@=sc@A=0@Sidx@A=20@S/\u{0}")
            //     continue;
            // }
            // if data.starts_with("type@=uenter".as_bytes()) {
            //     // Ok("type@=uenter/rid@=5981137/uid@=207346656/nn@=Christ无痕/level@=42/ic@=avatar_v3@S201901@S1af75c7cbe2c436284996ccb13108e0c/rni@=0/el@=/sahf@=0/wgei@=0/cbid@=29411/\u{0}")
            //     continue;
            // }
            // if data.starts_with("type@=ul_ranklist".as_bytes()) {
            //     // Ok("type@=ul_ranklist/rid@=5981137/ts@=1548814530/list_level@=crk@AA=1@ASuid@AA=234011126@ASlevel@AA=41@ASnn@AA=只摘星星不摘草莓@ASic@AA=avatar_v3@AAS201901@AASbf7ca23d9a0c40ee8e8717c556fae081@ASrg@AA=4@ASpg@AA=1@ASgt@AA=0@ASne@AA=3@ASsahf@AA=0@ASct@AA=2@AS@Scrk@AA=2@ASuid@AA=250612954@ASlevel@AA=33@ASnn@AA=洗月李乘风@ASic@AA=avatar_v3@AAS201812@AAS2e262ceef3c8621cf19e64c74f4385cb@ASrg@AA=1@ASpg@AA=1@ASgt@AA=0@ASne@AA=3@ASsahf@AA=0@ASct@AA=1@AS@Scrk@AA=3@ASuid@AA=103909722@ASlevel@AA=39@ASnn@AA=罓橙汁丶@ASic@AA=avatar_v3@AAS201901@AASdcb4541416cc40ea80a75abb29b599a5@ASrg@AA=4@ASpg@AA=1@ASgt@AA=0@ASne@AA=2@ASsahf@AA=0@ASct@AA=1@AS@Scrk@AA=4@ASuid@AA=257461061@ASlevel@AA=33@ASnn@AA=缘分丶让我们相遇@ASic@AA=avatar_v3@AAS201901@AASee772e1dc7424c1f8562ade45338d522@ASrg@AA=4@ASpg@AA=1@ASgt@AA=0@ASne@AA=1@ASsahf@AA=0@ASct@AA=1@AS@Scrk@AA=5@ASuid@AA=30178092@ASlevel@AA=22@ASnn@AA=怜身眼中人丶@ASic@AA=avatar_v3@AAS201812@AAS2733f77066f142e591a14f8310ccb752@ASrg@AA=4@ASpg@AA=1@ASgt@AA=0@ASne@AA=7@ASsahf@AA=0@ASct@AA=0@AS@Scrk@AA=6@ASuid@AA=32765858@ASlevel@AA=12@ASnn@AA=亲切的乱码弟弟101@ASic@AA=avatar@AASdefault@AAS09@ASrg@AA=1@ASpg@AA=1@ASgt@AA=0@ASne@AA=7@ASsahf@AA=0@ASct@AA=1@AS@Scrk@AA=7@ASuid@AA=204718094@ASlevel@AA=18@ASnn@AA=蔡羽中聪的四姨夫@ASic@AA=avatar_v3@AAS201901@AAS63857de01deb4976abf9d001c2459120@ASrg@AA=1@ASpg@AA=1@AS\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}")
            //     continue;
            // }
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
