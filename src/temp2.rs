use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Builder;
use std::fmt::Write;

fn encode_varint(mut val: i32) -> Vec<u8> {
    let mut buf = Vec::new();
    loop {
        let temp = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            buf.push(temp | 0x80);
        } else {
            buf.push(temp);
            break;
        }
    }
    buf
}

fn packet_to_hex(vals: Vec<u8>) -> String {
    let mut out = String::with_capacity(vals.len() * 3 + 3);
    write!(&mut out, "0x ").unwrap();
    for x in vals {
        write!(&mut out, "{:02x} ", x).unwrap();
    }
    out
}

enum State {
    HANDSHAKING,
    PINGING,
    LOGIN,
    CONFIG,
    PLAY
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()?;

    rt.block_on(async {
        let listener = TcpListener::bind("127.0.0.1:25565").await?;
        println!("Server listening on 127.0.0.1:25565");

        loop {
            let (mut socket, addr) = listener.accept().await?;
            println!("New cient connected: {}", addr);

            tokio::spawn(async move {
                let mut buf = vec![0u8; 512];
                let mut state = State::HANDSHAKING;

                loop {
                    match socket.read(&mut buf).await {
                        Ok(0) => {
                            println!("Client {} disconnected gracefully.", addr);
                            break;
                        }
                        Ok(n) => {
                            println!("Recieved {} bytes from {}: {:?}", n, addr, packet_to_hex(buf[..n].to_vec()));
                            match state {
                                State::HANDSHAKING => {
                                    if buf[1] == 0x00 {
                                        // TODO: Replace with a variable changes and checks to remove hardcodedness and allow any sized handshake packets to be recieved and handled properly
                                        if buf[2..=3] == [0xff, 0x05] {
                                            println!("Received handshake packet with protocol 767 (0xff05)");
                                            if buf[4] == 0x09 && (buf[5..=13] == [0x31, 0x32, 0x37, 0x2e, 0x30, 0x2e, 0x30, 0x2e, 0x31] || buf[5..=13] == [0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74] ) {
                                                println!("Received handshake packet with domain localhost");
                                                if buf[14..=15] == [0x63, 0xDD] {
                                                    println!("Received handshake packet with port 25565");
                                                    println!("Matching buffer @ 17: {}", buf[16]);
                                                    state = match buf[16] {
                                                        0x01 => {
                                                            println!("Entering PINGING state for connection {}", addr);
                                                            State::PINGING
                                                        }
                                                        0x02 => {
                                                            println!("Entering LOGIN state for connection {}", addr);
                                                            State::LOGIN
                                                        }
                                                        0x03 => {
                                                            // handle transfer
                                                            println!("Entering LOGIN (transfer) state for connection {}", addr);
                                                            State::LOGIN
                                                        }
                                                        _ => {
                                                            eprintln!("Invalid state, killing connection");
                                                            State::HANDSHAKING
                                                        }
                                                    }
                                                }
                                                else {
                                                    eprintln!("Received handshake packet with wrong port: {}", packet_to_hex(buf[14..=15].to_vec()))
                                                }
                                            }
                                            else
                                            {
                                                eprintln!("Received packet but wrong domain");
                                            }
                                        }
                                        else {
                                            eprintln!("Received packet but wrong protocol: {}", packet_to_hex(buf[2..=3].to_vec()));
                                        }
                                    }
                                }
                                State::PINGING => {
                                    println!("Recieved {} bytes from {}: {:?}", n, addr, packet_to_hex(buf[..n].to_vec()));
                            
                                    // is this a status request?
                                    if n >= 2 && buf[0] == 1 && buf[1] == 0 {
                                        
                                    } // echo back ping packet
                                    else if n >= 9 && buf[1] == 0x01 {
                                        
                                    }
                                }
                                State::LOGIN => {

                                }
                                State::CONFIG => {
                                
                                }
                                State::PLAY => {

                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from socket for {}: {}", addr, e);
                            break;
                        }
                    }
                }
            });
        }
    })
}

/* Server status request
println!("-> Received Status Request! Sending server JSON status...");

let json_response = r#"{
    "version": { "name": "1.21.1", "protocol": 767},
    "players": { "max": 50, "online": 0, "sample": [] },
    "description": { "text": "A HPC Rust Minecraft Server for Create Aeronautics!"}
}"#;

let json_bytes = json_response.as_bytes();
let mut payload = Vec::new();
payload.push(0x00);

let mut len_encoded = encode_varint(json_bytes.len() as i32);
payload.append(&mut len_encoded);
payload.extend_from_slice(json_bytes);

let mut final_packet = encode_varint(payload.len() as i32);
final_packet.append(&mut payload);

state = State::PINGING;

if let Err(e) = socket.write_all(&final_packet).await {
    eprintln!("Failed to write status response: {}", e);
    break;
}
*/

/* Each Ping
println!("-> Received Ping Packet! Echoing back payload...");
if let Err(e) = socket.write_all(&buf[..n]).await {
    eprintln!("Failed to write ping echo: {}", e);
    break;
}
*/