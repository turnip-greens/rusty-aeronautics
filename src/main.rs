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

const PORT : &str  = "127.0.0.1:25565";

enum PeakReturn {
    PERFECT { len : usize, val : u32 },
    MALFORMED
}

fn peak_varint_id(buf : &[u8; 5]) -> PeakReturn {
    if (buf[4] >> 7) == 1 {
        return PeakReturn::MALFORMED;
    }
    let mut len = 0;
    let mut total = 0u32;

    for &x in buf {
        let value = (x & 0x7f) as u32;
        total |= value << (len * 7);
        len += 1;

        if (x >> 7) == 0 {
            return PeakReturn::PERFECT { len, val: total}
        }
    }
    PeakReturn::MALFORMED
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
        let listener = TcpListener::bind(PORT).await?;
        println!("Server listening on {}", PORT);

        loop {
            let (mut socket, addr) = listener.accept().await?;
            println!("New cient connected: {}", addr);

            let mut buf = vec![0u32; 512];
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
                            match peak_varint_id(buf[0..5]) {
                                PeakReturn::PERFECT { len, val } => {

                                }
                                PeakReturn::MALFORMED => {

                                }
                            }
                            
                        }
                    }
                }
            });
        }
    })
}

