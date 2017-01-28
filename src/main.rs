extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;
use std::str;
use std::collections::BTreeMap;
use tokio_core::io::{Codec, EasyBuf};
use tokio_core::io::{Io, Framed};
use tokio_proto::pipeline::ServerProto;
use tokio_proto::TcpServer;
use tokio_service::Service;
use futures::{future, Future, BoxFuture};

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
                    // (TODO: ignore blank lines)
                    Ok(s) => {
                        // TODO: Find a better way to split the string
                        let s = s.trim();

                        let request = match s.find(' ') {
                            Some(i) => {
                                let (cmd, args) = s.split_at(i);
                                (cmd.trim().to_string(), Some(args.trim().to_string()))
                            },
                            None => (s.to_string(), None)
                        };

                        Ok(Some(request))
                    },

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


trait CliCommand {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn call(&self, args: Option<String>) -> String;
}


#[derive(Default)]
pub struct CliServer<'a> {
    commands: BTreeMap<&'a str, &'a CliCommand>,
}

impl<'a> CliServer<'a> {
    fn add_command(&mut self, cmd: &'a CliCommand) {
        self.commands.insert(cmd.name(), cmd);
    }
}

impl<'a> Service for CliServer<'a> {
    type Request = (String, Option<String>);
    type Response = String;

    type Error = io::Error;

    type Future = BoxFuture<Self::Response, Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match self.commands.get(&req.0[..]) {
            Some(cmd) => {
                println!("Calling command {}", req.0);
                // TODO: have commands return futures?
                future::ok(cmd.call(req.1)).boxed()
            },
            None => {
                println!("No match found for command {}", req.0);
                future::ok(format!("Unknown command: {}", req.0)).boxed()
            }
        }
    }
}

// TODO: Consider registering commands by passing two strings and a closure
struct EchoCommand;

impl CliCommand for EchoCommand {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Prints the command line."
    }

    fn call(&self, args: Option<String>) -> String {
        match args {
            Some(s) => s,
            None => "".to_string()
        }
    }
}

static ECHO: EchoCommand = EchoCommand;

fn main() {
    let addr = "0.0.0.0:14311".parse().unwrap();
    let server = TcpServer::new(CliProto, addr);

    println!("Serving on {}", addr);
    server.serve(|| {
        let mut cli = CliServer::default();
        cli.add_command(&ECHO);
        Ok(cli)
    });
}
