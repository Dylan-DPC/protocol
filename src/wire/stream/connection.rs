use {PacketKind, Error};
use wire::stream::{Transport, transport};
use wire::middleware;

use std::io::prelude::*;
use std::io::Cursor;

/// A stream-based connection.
// TODO: Allow custom transports.
pub struct Connection<P: PacketKind, S: Read + Write, M: middleware::Pipeline = middleware::pipeline::Default>
{
    pub stream: S,
    pub transport: transport::Simple,
    pub middleware: M,

    pub _a: ::std::marker::PhantomData<P>,
}

impl<P,S,M> Connection<P,S,M>
    where P: PacketKind, S: Read + Write, M: middleware::Pipeline
{
    /// Creates a new connection.
    pub fn new(stream: S, middleware: M) -> Self {
        Connection {
            stream: stream,
            transport: transport::Simple::new(),
            middleware: middleware,
            _a: ::std::marker::PhantomData,
        }
    }

    /// Processes any incoming data in thes stream.
    pub fn process_incoming_data(&mut self) -> Result<(), Error> {
        self.transport.process_data(&mut self.stream)
    }

    /// Attempts to receive a packet.
    pub fn receive_packet(&mut self) -> Result<Option<P>, Error> {
        self.process_incoming_data()?;

        if let Some(raw_packet) = self.transport.receive_raw_packet()? {
            let mut packet_data = Cursor::new(self.middleware.decode_data(raw_packet)?);

            let packet = P::read(&mut packet_data)?;

            Ok(Some(packet))
        } else {
            Ok(None)
        }
    }

    /// Sends a packet.
    pub fn send_packet(&mut self, packet: &P) -> Result<(), Error> {
        let raw_packet = self.middleware.encode_data(packet.bytes()?)?;
        self.transport.send_raw_packet(&mut self.stream, &raw_packet)
    }

    pub fn into_inner(self) -> S { self.stream }
}

#[cfg(test)]
mod test
{
    pub use PacketKind;
    pub use super::Connection;
    pub use wire::middleware;

    pub use std::io::Cursor;

    define_packet!(Ping {
        data: Vec<u8>
    });

    define_packet_kind!(Packet : u8 {
        0x00 => Ping
    });

    describe! connection {
        it "can write and read back data" {
            let ping = Packet::Ping(Ping { data: vec![5, 4, 3, 2, 1]});

            let buffer = Cursor::new(Vec::new());
            let mut connection = Connection::new(buffer, middleware::pipeline::default());

            connection.send_packet(&ping).unwrap();

            // Read the packet back.
            connection.stream.set_position(0);
            let response = connection.receive_packet().unwrap();

            assert_eq!(response.unwrap().bytes().unwrap(), ping.bytes().unwrap());
        }
    }
}

