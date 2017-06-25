//! This module contains the `PcapWriter` struct which is used to write to a pcap file

use std::borrow::Cow;
use std::io::Write;

use byteorder::{BigEndian, LittleEndian};

use packet::{Packet, PacketHeader};
use pcap_header::{Datalink, PcapHeader};
use errors::*;

/// This struct wraps another writer and enables it to write a Pcap formated stream.
///
/// # Exemple
///
/// ```no_run
/// use std::fs::File;
/// use pcap_rs::{PcapReader, PcapWriter};
///
///
/// let file_in = File::open("test.pcap").expect("Error opening file");
/// let pcap_reader = PcapReader::new(file_in).unwrap();
///
/// let file_out = File::create("out.pcap").expect("Error creating file out");
/// let mut pcap_writer = PcapWriter::new(file_out).expect("Error writing file");
///
/// // Read test.pcap
/// for pcap in pcap_reader {
///
///     //Write each packet of test.pcap in out.pcap
///     pcap_writer.write_packet(&pcap);
/// }
/// ```
#[derive(Debug)]
pub struct PcapWriter<T: Write> {
    pub header: PcapHeader,
    writer: T,
}


impl<T: Write> PcapWriter<T> {
    /// Create a new `PcapWriter` from an existing writer.
    ///
    /// It Automatically writes this default global pcap header to the file:
    ///
    /// ```ignore
    /// PcapHeader {
    ///
    ///     magic_number : 0xa1b2c3d4,
    ///     version_major : 2,
    ///     version_minor : 4,
    ///     ts_correction : 0,
    ///     ts_accuracy : 0,
    ///     snaplen : 65535,
    ///     datalink : Datalink::Ethernet
    /// };
    /// ```
    ///
    /// # Errors
    ///
    /// Return an error if the writer can't be written to.
    ///
    ///
    /// # Exemple
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use pcap_rs::PcapWriter;
    ///
    /// let file_out = File::create("out.pcap").expect("Error creating file");
    /// let mut pcap_writer = PcapWriter::new(file_out);
    /// ```
    pub fn new(writer: T) -> ResultChain<PcapWriter<T>> {

        let header = PcapHeader {
            magic_number: 0xa1b2c3d4,
            version_major: 2,
            version_minor: 4,
            ts_correction: 0,
            ts_accuracy: 0,
            snaplen: 65535,
            datalink: Datalink::Ethernet,
        };

        PcapWriter::with_header(header, writer)
    }

    /// Create a new `PcapWriter` from an existing writer with a user defined global pcap header.
    ///
    /// Automatically write the global pcap header to the file.
    ///
    /// # Errors
    ///
    /// Return an error if the writer can't be written to.
    ///
    ///
    /// # Exemple
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use pcap_rs::PcapWriter;
    /// use pcap_rs::pcap_header::{PcapHeader, Datalink};
    ///
    /// let file = File::create("out.pcap").expect("Error creating file");
    ///
    /// let header = PcapHeader {
    ///
    ///     magic_number : 0xa1b2c3d4,
    ///     version_major : 2,
    ///     version_minor : 4,
    ///     ts_correction : 0,
    ///     ts_accuracy : 0,
    ///     snaplen : 65535,
    ///     datalink : Datalink::Ethernet
    /// };
    ///
    /// let mut pcap_writer = PcapWriter::with_header(header, file);
    /// ```
    pub fn with_header(header: PcapHeader, mut writer: T) -> ResultChain<PcapWriter<T>> {

        match header.magic_number {

            0xa1b2c3d4 => writer.write_all(&header.to_array::<BigEndian>()?)?,
            0xd4c3b2a1 => writer.write_all(&header.to_array::<LittleEndian>()?)?,
            _ => unreachable!("The magic number should always be valid here")
        }

        Ok(
            PcapWriter {
                header: header,
                writer: writer,
            }
        )
    }

    /// Consumes the `PcapWriter`, returning the wrapped writer.
    ///
    /// # Exemple
    /// ```no_run
    /// use std::fs::File;
    /// use pcap_rs::PcapWriter;
    ///
    /// let file = File::create("out.pcap").expect("Error creating file");
    /// let pcap_writer = PcapWriter::new(file).unwrap();
    ///
    /// let file2 = pcap_writer.into_writer();
    /// ```
    pub fn into_writer(self) -> T {
        self.writer
    }

    /// Gets a reference to the underlying writer.
    ///
    /// # Exemple
    /// ```no_run
    /// use std::fs::File;
    /// use pcap_rs::PcapWriter;
    ///
    /// let file = File::create("out.pcap").expect("Error creating file");
    /// let pcap_writer = PcapWriter::new(file).unwrap();
    ///
    /// let file_ref = pcap_writer.get_ref();
    /// ```
    pub fn get_ref(&self) -> &T {
        &self.writer
    }

    /// Gets a mutable reference to the underlying writer.
    ///
    /// It is inadvisable to directly write to the underlying writer.
    ///
    /// # Exemple
    /// ```no_run
    /// use std::fs::File;
    /// use pcap_rs::PcapWriter;
    ///
    /// let file = File::create("out.pcap").expect("Error creating file");
    /// let mut pcap_writer = PcapWriter::new(file).unwrap();
    ///
    /// let file_mut = pcap_writer.get_mut();
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.writer
    }

    /// Write some raw data, converting it to the pcap file format.
    ///
    /// # Exemple
    /// ```no_run
    /// use std::fs::File;
    /// use pcap_rs::PcapWriter;
    ///
    /// let data = [0u8; 10];
    /// let file = File::create("out.pcap").expect("Error creating file");
    /// let mut pcap_writer = PcapWriter::new(file).unwrap();
    ///
    /// pcap_writer.write(0, 0, &data).unwrap();
    /// ```
    pub fn write(&mut self, ts_sec: u32, ts_usec: u32, data: &[u8]) -> ResultChain<()> {

        let packet = Packet {
            header: PacketHeader {
                ts_sec: ts_sec,
                ts_usec: ts_usec,
                incl_len: data.len() as u32,
                orig_len: data.len() as u32,
            },

            data: Cow::Borrowed(data),
        };

        self.write_packet(&packet)
    }

    /// Write a `Packet`.
    ///
    /// # Exemple
    /// ```no_run
    /// use std::fs::File;
    /// use pcap_rs::{Packet, PcapWriter};
    ///
    /// let data = [0u8; 10];
    /// let packet = Packet::new(0, 0, 10, &data);
    ///
    /// let file = File::create("out.pcap").expect("Error creating file");
    /// let mut pcap_writer = PcapWriter::new(file).unwrap();
    ///
    /// pcap_writer.write_packet(&packet).unwrap();
    /// ```
    pub fn write_packet(&mut self, packet: &Packet) -> ResultChain<()> {

        match self.header.magic_number {

            0xa1b2c3d4 => self.writer.write_all(&packet.header.to_array::<BigEndian>()?)?,
            0xd4c3b2a1 => self.writer.write_all(&packet.header.to_array::<LittleEndian>()?)?,
            _ => unreachable!("The magic number should always be valid here")
        }
        self.writer.write_all(&packet.data)?;

        Ok(())
    }
}