use crate::structs::{DNSHeader, DNSQuestion};
use std::net::UdpSocket;

pub fn get_header(buf: &[u8]) -> DNSHeader {
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

pub fn read_name(packet: &[u8], mut offset: usize) -> (Vec<u8>, usize) {
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

pub fn get_question(packet: &[u8], offset: usize) -> (DNSQuestion, usize) {
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

pub fn set_response_bits(response: &mut [u8], question_count: u16) {
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

pub fn get_resolver_answer(
    resolver: &str,
    question: &DNSQuestion,
    header: &DNSHeader,
    udp_socket: &UdpSocket,
) -> std::io::Result<Vec<u8>> {
    let mut resolver_request: Vec<u8> = Vec::new();
    let mut resolver_header = header.to_bytes();
    resolver_header[4] = 0x00;
    resolver_header[5] = 0x01;
    resolver_request.extend(&resolver_header);
    resolver_request.extend(&question.to_bytes());

    udp_socket.send_to(&resolver_request, resolver)?;
    let mut res_buf = [0; 512];
    udp_socket.recv_from(&mut res_buf)?;
    let (_, resolver_response_q_consumed) = get_question(&res_buf, 12);
    let resolver_answer = &res_buf[12 + resolver_response_q_consumed..];
    Ok(resolver_answer.to_vec())
}