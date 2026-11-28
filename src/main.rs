use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Builder;
use tokio::time::timeout;
use std::fmt::Write;

enum State {
    HANDSHAKING,
    PINGING,
    LOGIN,
    CONFIG,
    PLAY
}

const IP : &str  = "127.0.0.1";
const PORT : u16 = 25565

enum PeakReturn {
    PERFECT { len : usize, val : u32 },
    INCOMPLETE,
    MALFORMED
}

fn peek_varint(buf: &[u8]) -> PeekReturn {
    let mut total = 0u32;
    for (i, &byte) in buf.iter().take(5).enumerate() {
        // checks for varint that would cause overflow in u32s
        if i == 4 && (byte & 0xf0) != 0 {
            return PeekReturn::MALFORMED;
        }

        let value = (byte & 0x7f) as u32;
        total |= value << (i * 7);

        if (byte & 0x80) == 0 {
            return PeekReturn::PERFECT {
                len: i + 1,
                val: total as i32,
            };
        }
    }
    PeekReturn::INCOMPLETE
}

fn read_varint(buf: &mut &[u8]) -> Option<i32> {
    match peak_varint(buf) {
        PeakReturn::PERFECT { len, val } => {
            *buf = &buf[len..];
            Some(val as i32)
        } 
        PeakReturn::INCOMPLETE {
            
        }
    } 
}

fn packet_to_hex(vals: Vec<u8>) -> String {
    let mut out = String::with_capacity(vals.len() * 3 + 3);
    write!(&mut out, "0x ").unwrap();
    for x in vals {
        write!(&mut out, "{:02x} ", x).unwrap();
    }
    out
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()?;

    rt.block_on(async {
        let listener = TcpListener::bind((&str, u16)).await?;
        println!("Server listening on {}:{}", IP, PORT);

        loop {
            let (mut socket, addr) = listener.accept().await?;
            println!("New cient connected: {}", addr);

            let mut buf = vec![0u8; 512];
            let mut byte_stack : Vec<u8>;

            let mut state = State::HANDSHAKING;

            tokio::spawn(async move {

                loop {
                    match socket.read(&mut buf).await {
                        Ok(0) => {
                            println!("Client {} disconnected gracefully.", addr);
                            break;
                        }
                        Ok(n) => {
                            println!("Recieved {} bytes from {}: {:?}", n, addr, packet_to_hex(buf[..n].to_vec()));
                            
                            match peak_varint(&byte_stack) {
                                PeakReturn::PERFECT { len : varint_len, val : packet_len } => {
                                    if packet_len <= 0 {
                                        eprintln!("Recieved negative (or zero) packet length prefix {}", packet_len);
                                        return;
                                    }
                                    // check if need more bytes
                                    if (packet_len as usize) + varint_len > byte_stack.len() {
                                        break;
                                    }
                                    // now we can process packet
                                    byte_stack.drain(..varint_len); // remove and discard varint size header
                                    let body: Vec<u8> = byte_stack.drain(..packet_len as usize).collect(); // store the actual packet

                                    match peak_varint(&body) {
                                        PeakReturn::PERFECT { len : id_varint_len, val : packet_id } {
                                            body.drain(..id_varint_len); 
                                            match (&state, packet_id) {
                                                (State::HANDSHAKING, 0) => {
                                                    // handle state change packet

                                                    
                                                }
                                                _ => {
                                                    eprintln!("Recieve unknown packet ID {} for state {:?}", packet_id, &state);
                                                    break;
                                                }
                                            }
                                        }
                                        _ => {
                                            return;
                                        }
                                    }
                                }
                                PeakReturn::INCOMPLETE => {
                                    break;
                                }
                                PeakReturn::MALFORMED => {
                                    return;
                                }
                            }
                            
                        }
                    }
                }
            });
        }
    })
}

