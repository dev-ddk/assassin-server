# Assassin Server

Server for the Assassin Project backend.

## Compiling and running

To compile the project, you need [Rust](https://www.rust-lang.org/).

First, you must provide a .env file. To do so, you may copy the sample file `.env.sample` and edit it to your liking:

```
$ cp .env.sample .env
```

Afterwards, you may start the project in development mode:

```
$ cd assassin-server
$ cargo run
```

or in release mode:

```
$ cargo build --release
$ ./target/release/assassin_server
```

## Developing

For easier development, you should install `systemfd` and `cargo-watch`:

```
$ cargo install systemfd cargo-watch
```

Then, you can run the server in watch mode. The server will automatically detect changes in the source files and reload itself automatically, while keeping control of the socket.

```
$ systemfd --no-pid -s http::5000 -- cargo watch -x run
```

Or equivalently:

```
$ ./run-dev.sh
```
