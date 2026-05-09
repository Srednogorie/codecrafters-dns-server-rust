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
    length: u16,
    rdata: u32,
}
impl DNSAnswer {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.name.clone();
        bytes.push((self.rtype >> 8) as u8);
        bytes.push(self.rtype as u8);
        bytes.push((self.rclass >> 8) as u8);
        bytes.push(self.rclass as u8);
        bytes.push((self.ttl >> 24) as u8);
        bytes.push((self.ttl >> 16) as u8);
        bytes.push((self.ttl >> 8) as u8);
        bytes.push(self.ttl as u8);
        bytes.push((self.length >> 8) as u8);
        bytes.push(self.length as u8);
        bytes.push((self.rdata >> 24) as u8);
        bytes.push((self.rdata >> 16) as u8);
        bytes.push((self.rdata >> 8) as u8);
        bytes.push(self.rdata as u8);
        bytes
    }
}

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

fn read_name(packet: &[u8], mut offset: usize) -> (Vec<u8>, usize) {
    let mut name = Vec::new();
    let start_offset = offset;

    loop {
        let byte = packet[offset];

        if byte == 0 {
            // End of name
            offset += 1;
            name.push(0);
            break;
        } else if byte >= 0xC0 {
            // Pointer: 2 bytes, upper 2 bits are 11, remaining 14 bits are offset
            let second_byte = packet[offset + 1];
            let pointer_offset = (((byte as u16) & 0x3F) << 8) | (second_byte as u16);
            offset += 2;
            
            // Recursively read the name from the pointer target
            let (followed_name, _) = read_name(packet, pointer_offset as usize);
            name.extend(followed_name);
            break;
        } else {
            // Normal label: length byte followed by that many bytes
            let length = byte as usize;
            name.push(length as u8);
            name.extend_from_slice(&packet[offset + 1..offset + 1 + length]);
            offset += 1 + length;
        }
    }

    (name, offset - start_offset)
}

fn get_question(packet: &[u8], offset: usize) -> (DNSQuestion, usize) {
    let (name, name_consumed) = read_name(packet, offset);
    let qtype = u16::from_be_bytes([
        packet[offset + name_consumed],
        packet[offset + name_consumed + 1],
    ]);
    let qclass = u16::from_be_bytes([
        packet[offset + name_consumed + 2],
        packet[offset + name_consumed + 3],
    ]);
    let total_consumed = name_consumed + 4;

    (
        DNSQuestion { name, qtype, qclass },
        total_consumed,
    )
}

fn set_response_bits(response: &mut [u8], question_count: u16) {
    // Set QR bit to 1 (response) and RCODE to 4 (not implemented)
    response[2] |= 0x80;
    response[3] = (response[3] & 0xF0) | 0x04;
    // Set QDCOUNT to 1 (one question)
    response[4] &= 0xFF;
    response[5] |= question_count as u8;
    // Set ANCOUNT to 1 (one answer)
    response[6] &= 0xFF;
    response[7] |= question_count as u8;
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];
    
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let mut response: Vec<u8> = Vec::new();
                // Check if the request is a DNS query
                if buf[2] & 0x80 == 0 {
                    let header = get_header(&buf[..12]);
                    let question_count = header.qdcount;

                    let mut questions = Vec::new();
                    let mut offset = 0;
                    for _ in 0..question_count {
                        let (question, consumed) = get_question(&buf, 12 + offset);
                        questions.push(question);
                        offset += consumed;
                    }
                    response.extend(&header.to_bytes());
                    for question in &questions {
                        response.extend(&question.to_bytes());
                    }
                    for question in &questions {
                        let answer = DNSAnswer {
                            name: question.name.clone(),
                            rtype: question.qtype,
                            rclass: question.qclass,
                            ttl: 60,
                            length: 4,
                            rdata: 0x7f000001,
                        };
                        response.extend(&answer.to_bytes());
                    }

                    set_response_bits(&mut response, question_count);

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
