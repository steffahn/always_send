# always_send

[![docs.rs]](https://docs.rs/always_send)
[![crates.io]](https://crates.io/crates/always_send)
[![github]](https://github.com/steffahn/always_send)
[![MIT / Apache 2.0 licensed]](#License)

[github]: https://img.shields.io/badge/github-steffahn/always__send-yellowgreen.svg
[crates.io]: https://img.shields.io/crates/v/always_send.svg
[MIT / Apache 2.0 licensed]: https://img.shields.io/crates/l/always_send.svg
[docs.rs]: https://docs.rs/always_send/badge.svg

Wrapper type to check `Send` only on construction, so `rustc` isn’t confused.

For more context, please refer to the documentation of [`always_send::AlwaysSend`][AlwaysSend].

[AlwaysSend]: https://docs.rs/always_send/0.1/always_send/struct.AlwaysSend.html

## License
Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
