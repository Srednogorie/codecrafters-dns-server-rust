mod structs;
use structs::*;

mod utils;
use utils::*;

use std::{net::UdpSocket};
use clap::Parser;
use anyhow::Result;


fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let args = Args::parse();

    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];
    
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((_, source)) => {
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
                        if args.resolver.is_some() {
                            let resolver = args.resolver.as_ref().ok_or(anyhow::anyhow!("Resolver not found"))?;
                            let resolver_answer = get_resolver_answer(resolver, question, &header, &udp_socket)?;
                            response.extend(resolver_answer);
                        } else {
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
                return Err(e.into());
            }
        }
    }
}