# cliserver_rust

# WIP - Much TODO

A simple interactive command-line server built in Rust using [Tokio][0].  This
was built as a learning exercise for myself to translate my knowledge of
libevent in C and EventMachine in Ruby to Rust.  It is based in concept on my
original [cliserver][1] example code for libevent, using the [Tokio echo server
example][2] to get started.

Clients connect to the server on port 14311, allowing them to run the following
commands:

- **echo** -- Print the command line.
- **help** -- Print a list of commands and their descriptions.
- **info** -- Print connection information.
- **quit** -- Disconnect from the server.
- **kill** -- Shut down the server.


# Compiling cliserver_rust

Compile the server with `cargo build`, run it with `cargo run`.  Connect to the
server using netcat: `nc localhost 14311`.

# Copyright
(C)2017 Mike Bourgeous, licensed under 2-clause BSD

[0]: https://tokio.rs
[1]: https://github.com/mike-bourgeous/cliserver
[2]: https://tokio.rs/docs/getting-started/simple-server/
