#[allow(unused_imports)]
use std::net::UdpSocket;

#[derive(Debug)]
struct DNSHeader {
    id: u16,
    flags: u16,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}
impl DNSHeader {
    fn to_bytes(&self) -> Vec<u8> {
        vec![
            (self.id >> 8) as u8,
            self.id as u8,
            (self.flags >> 8) as u8,
            self.flags as u8,
            (self.qdcount >> 8) as u8,
            self.qdcount as u8,
            (self.ancount >> 8) as u8,
            self.ancount as u8,
            (self.nscount >> 8) as u8,
            self.nscount as u8,
            (self.arcount >> 8) as u8,
            self.arcount as u8,
        ]
    }
}

struct DNSResponse {
    header: DNSHeader,
    // questions: Vec<DNSQuestion>,
    // answers: Vec<DNSAnswer>,
    // authorities: Vec<DNSAuthority>,
    // additional: Vec<DNSAdditional>,
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // TODO: Uncomment the code below to pass the first stage
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];
    
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                // Check if the request is a DNS query
                if buf[2] & 0x80 == 0 {
                    let header = DNSHeader {
                        id: u16::from_be_bytes([buf[0], buf[1]]),
                        flags: u16::from_be_bytes([buf[2], buf[3]]),
                        qdcount: u16::from_be_bytes([buf[5], buf[6]]),
                        ancount: u16::from_be_bytes([buf[7], buf[8]]),
                        nscount: u16::from_be_bytes([buf[9], buf[10]]),
                        arcount: u16::from_be_bytes([buf[11], buf[12]]),
                    };
                    println!("DNS query: {:?}", header);
                    let response = DNSResponse {
                        header,
                        // questions: vec![],
                        // answers: vec![],
                        // authorities: vec![],
                        // additional: vec![],
                    };
                    let mut response_bytes = response.header.to_bytes();
                    // Set QR bit to 1 (response) and RCODE to 0 (no error)
                    response_bytes[2] |= 0x80;
                    response_bytes[3] &= 0xF0;
                    println!("Response len: {}", response_bytes.len());
                    udp_socket
                        .send_to(&response_bytes, source)
                        .expect("Failed to send response");
                } else {
                    println!("Received a non-DNS query");
                    continue;
                }
                // println!("Received {} bytes from {}", size, source);
                // let response = [];
                // udp_socket
                //     .send_to(&response, source)
                //     .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
