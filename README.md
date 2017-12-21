# robinhood

[![Travis CI Status](https://travis-ci.org/sanko/robinhood_rs.svg?branch=master)](https://travis-ci.org/sanko/robinhood_rs)
[![Appveyor CI Status](https://ci.appveyor.com/api/projects/status/1w7jp6qlo6ox2uxr?svg=true)](https://ci.appveyor.com/project/sanko/robinhood-rs)
[![crates.io](https://img.shields.io/crates/v/robinhood.svg)](https://crates.io/crates/robinhood)

Client for Robinhood, the commission-free brokerage, written for Rust.

Please note that this is a very early release and the API will change a lot over the coming days and weeks.

- [Documentation](https://docs.rs/robinhood)
- [Changelog](CHANGELOG.md)

## Requirements

On Linux:

- OpenSSL 1.0.1, 1.0.2, or 1.1.0 with headers (see https://github.com/sfackler/rust-openssl)

On Windows and macOS:

- Nothing.

Robinhood uses reqwest which uses [rust-native-tls](https://github.com/sfackler/rust-native-tls), which will use the operating system TLS framework on Windows and macOS. On Linux, it will use OpenSSL 1.1.

## Installation

```toml
[dependencies]
robinhood = "*"
```

## Example

```rust
extern crate robinhood;

use robinhood::Client;

fn main() {
    let rh = Client::new()
        .build()
        .unwrap();

    let instruments = rh.instruments();
    println!("{:#?}", instruments);
    for instrument in instruments.take(3) {
        println!("Instrument: {:#?}", instrument);
    }
}
```

## License

Licensed under the Artistic License, Version 2.0 ([LICENSE](LICENSE) or https://opensource.org/licenses/Artistic-2.0)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Artistic-2.0 license, shall be licensed as above, without any additional terms or conditions.