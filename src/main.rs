extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;
use std::str;
use tokio_core::io::{Codec, EasyBuf};
use tokio_core::io::{Io, Framed};
use tokio_proto::pipeline::ServerProto;

#[derive(Default)]
pub struct CliCodec {
    /// Offset within the incoming EasyBuf at which newline search last ended
    search_offset: usize,
}

impl Codec for CliCodec {
    /// Input type is a tuple with the command name and an optional argument string
    type In = (String, Option<String>);

    /// Response type is just a string
    type Out = String;

    /// Find a newline, decode to a tuple of command name and argument string
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        // Find newline
        let m = buf
            .as_slice()
            .iter()
            .skip(self.search_offset)
            .position(|&b| b == b'\n');

        match m {
            // Process the line if a newline was found
            Some(idx) => {
                println!("Found newline at position {}, started at {}", idx, self.search_offset); // XXX

                let line = buf.drain_to(self.search_offset + idx);
                buf.drain_to(1);

                self.search_offset = 0;

                match str::from_utf8(line.as_slice()) {
                    // Return the string on successful UTF-8 decode
                    // (TODO: split on the first space)
                    Ok(s) => Ok(Some(
                            ("Received".to_string(), Some(s.trim().to_string()))
                            )),

                    // Return an error if invalid UTF-8 is received
                    Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("invalid UTF-8: {:?}", e))),
                }
            },

            // Save our position if a newline wasn't found
            _ => {
                println!("Haven't received newline yet!  We started at offset {}, now at {}", self.search_offset, buf.len()); // XXX

                self.search_offset = buf.len();

                Ok(None)
            }
        }
    }

    /// Append the String to the outgoing buffer
    fn encode(&mut self, msg: String, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.extend(msg.as_bytes());
        buf.push(b'\n');
        Ok(())
    }
}


pub struct CliProto;

impl<T: Io + 'static> ServerProto<T> for CliProto {
    type Request = (String, Option<String>);
    type Response = String;

    type Transport = Framed<T, CliCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(CliCodec::default()))
    }
}


fn main() {
    // TODO
    println!("Hello, world!");
}
