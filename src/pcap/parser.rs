//! This module contains the `PcapReader` struct which is used to read from a pcap file

use byteorder::{BigEndian, LittleEndian};

use crate::{
    Endianness,
    errors::*,
    pcap::Packet,
    pcap::PcapHeader
};


/// Helper struct to parse a file
///
/// # Examples
///
/// ```no_run
/// use pcap_file::pcap::PcapParser;
///
/// let pcap = vec![0_u8; 0];
///
/// // Parse all the packets
/// let mut src = &pcap[..];
/// let (pcap_parser, rem) = PcapParser::new(&pcap[..]).unwrap();
/// src = rem;
///
/// while !src.is_empty() {
///
///     let (packet, rem) = pcap_parser.next_packet(src).unwrap();
///     println!("{:?}", packet);
///     src = rem;
/// }
/// ```
#[derive(Debug)]
pub struct PcapParser {
    header: PcapHeader
}

impl PcapParser {

    /// Creates a new `PcapParser`.
    /// Returns the parser and the remainder.
    pub fn new(slice: &[u8]) -> ResultParsing<(PcapParser, &[u8])> {

        let slice = slice;

        let (header, slice) = PcapHeader::from_slice(slice)?;

        let parser = PcapParser {
            header
        };

        Ok((parser, slice))
    }

    /// Returns the next packet and the remainder.
    pub fn next_packet<'a>(&self, slice: &'a[u8]) -> ResultParsing<(Packet<'a>, &'a[u8])> {

        let ts_resolution = self.header.ts_resolution();

        match self.header.endianness() {
            Endianness::Big => Packet::from_slice::<BigEndian>(slice, ts_resolution),
            Endianness::Little => Packet::from_slice::<LittleEndian>(slice, ts_resolution)
        }
    }
}