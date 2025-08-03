# android-version-searcher

**Work in progress**, or at least unfinished.

Statically finds the following properties of Android partitions:
- boot: SDK level, security patch and Magisk
- system: nothing yet! awaiting userspace EROFS implementation

Useful for seeing when over-the-air (OTA) updates have actually been written to
disk on the Android operating system on your phone.

For example, I believe that the "Install" button on OnePlus devices actually
means "Reboot". This isn't documented in Magisk so the first time I tried to
re-apply Magisk on top of the OTA I was left with no Magisk install.

^^ Combined with OnePlus's broken `fastboot fetch`, I ultimately had to
download the OTA again..


## Development

0. Have Linux or MacOS

1. Install [Nix](https://nixos.org/download#download-nix)

2. Run the command `nix develop` in a shell.

   This creates a `bash` subshell with all the dependencies.

3. Run `cargo` commands as you like.

   i.e. `cargo build`, `cargo run`, `cargo clippy`, etc.

## Contributing patches

Please first make sure that you have not introduced any regressions and format the code by running the following commands at the repository root.
```sh
cargo fmt
cargo clippy
cargo test
```

Then make a GitHub [pull request](https://github.com/axelkar/android-version-searcher/pulls).

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
