use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The DNS resolver to use
    #[arg(long)]
    pub resolver: Option<String>,
}


#[derive(Debug)]
pub struct DNSHeader {
    pub id: u16,
    pub flags: u16,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}
impl DNSHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
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
pub struct DNSQuestion {
    pub name: Vec<u8>,
    pub qtype: u16,
    pub qclass: u16,
}
impl DNSQuestion {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.name.clone();
        bytes.push((self.qtype >> 8) as u8);
        bytes.push(self.qtype as u8);
        bytes.push((self.qclass >> 8) as u8);
        bytes.push(self.qclass as u8);
        bytes
    }
}

#[derive(Debug)]
pub struct DNSAnswer {
    pub name: Vec<u8>,
    pub rtype: u16,
    pub rclass: u16,
    pub ttl: u32,
    pub length: u16,
    pub rdata: u32,
}
impl DNSAnswer {
    pub fn to_bytes(&self) -> Vec<u8> {
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