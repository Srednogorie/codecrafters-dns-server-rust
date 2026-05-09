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

#[derive(Debug)]
struct DNSQuestion {
    name: Vec<u8>,
    qtype: u16,
    qclass: u16,
}
impl DNSQuestion {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.name.clone();
        bytes.push(0);
        bytes.push((self.qtype >> 8) as u8);
        bytes.push(self.qtype as u8);
        bytes.push((self.qclass >> 8) as u8);
        bytes.push(self.qclass as u8);
        bytes
    }
}

#[derive(Debug)]
struct DNSAnswer {
    name: Vec<u8>,
    rtype: u16,
    rclass: u16,
    ttl: u32,
    rdata: u32,
}
impl DNSAnswer {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.name.clone();
        bytes.push(0);
        bytes.push((self.rtype >> 8) as u8);
        bytes.push(self.rtype as u8);
        bytes.push((self.rclass >> 8) as u8);
        bytes.push(self.rclass as u8);
        bytes.push((self.ttl >> 24) as u8);
        bytes.push((self.ttl >> 16) as u8);
        bytes.push((self.ttl >> 8) as u8);
        bytes.push(self.ttl as u8);
        bytes.push((self.rdata >> 24) as u8);
        bytes.push((self.rdata >> 16) as u8);
        bytes.push((self.rdata >> 8) as u8);
        bytes.push(self.rdata as u8);
        bytes
    }
}

// struct DNSResponse {
//     header: DNSHeader,
//     question: DNSQuestion,
//     // answers: Vec<DNSAnswer>,
//     // authorities: Vec<DNSAuthority>,
//     // additional: Vec<DNSAdditional>,
// }

fn get_header(buf: &[u8]) -> DNSHeader {
    let header = DNSHeader {
        id: u16::from_be_bytes([buf[0], buf[1]]),
        flags: u16::from_be_bytes([buf[2], buf[3]]),
        qdcount: u16::from_be_bytes([buf[4], buf[5]]),
        ancount: u16::from_be_bytes([buf[6], buf[7]]),
        nscount: u16::from_be_bytes([buf[8], buf[9]]),
        arcount: u16::from_be_bytes([buf[10], buf[11]]),
    };
    header
}

fn get_question(buf: &[u8]) -> (DNSQuestion, usize) {
    if let Some(null_pos) = buf.iter().position(|&b| b == 0) {
        let content = &buf[..null_pos];
        let question = DNSQuestion {
            name: content.to_vec(),
            qtype: u16::from_be_bytes([buf[null_pos + 1], buf[null_pos + 2]]),
            qclass: u16::from_be_bytes([buf[null_pos + 3], buf[null_pos + 4]]),
        };
        (question, null_pos + 5)
    } else {
        let question = DNSQuestion {
            name: buf.to_vec(),
            qtype: 0,
            qclass: 0,
        };
        (question, 0)
    }
}

fn get_answer(buf: &[u8]) -> DNSAnswer {
    
}

fn set_response_bits(response: &mut [u8]) {
    // Set QR bit to 1 (response) and RCODE to 0 (no error)
    response[2] |= 0x80;
    response[3] &= 0xF0;
    // Set QDCOUNT to 1 (one question)
    response[4] &= 0xFF;
    response[5] |= 0x01;
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];
    
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let mut response = vec![];
                // Check if the request is a DNS query
                if buf[2] & 0x80 == 0 {
                    let header = get_header(&buf[..12]).to_bytes();
                    response.extend(&header);
                    let (question, offset) = get_question(&buf[12..size]);
                    response.extend(&question.to_bytes());
                    let answer = get_answer(&buf[12 + offset..size]).to_bytes();
                    response.extend(&answer);

                    set_response_bits(&mut response);

                    udp_socket
                        .send_to(&response, source)
                        .expect("Failed to send response");

                } else {
                    println!("Received a non-DNS query");
                    continue;
                }
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
