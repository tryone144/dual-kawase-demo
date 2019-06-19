dual-kawase-demo
================

**Simple live demo of the dual-filter kawase blur algorithm.**

This is just a simple test project as of now. It might kill your cat!

## Build

Just clone this repo and build with `cargo`:

```sh
$ git clone https://github.com/tryone144/dual-kawase-demo && cd dual-kawase-demo
$ cargo build
$ cargo run -- /path/to/image.(png|jpg)
```

You can then modify the number of iterations and the pixel offset width the arrow-keys.
Reset all parameters with `r` and save the blurred image to a file with `s`.
Some (arbitrary) presets are available via the number keys `1` to `9`.
Toggle fullscreen/windowed display with `f`.

---

## License

dual-kawase-demo is licensed under either of

* [Apache License, Version 2.0](./LICENSE-APACHE) (see http://www.apache.org/licenses/LICENSE-2.0)
* [MIT license](./LICENSE-MIT) (see http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in portal-rc by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Legal

The bundled [Ubuntu Monospace](./assets/UbuntuMono-R.ttf) font is licensed under the [Ubuntu Font Licence Version 1.0](./assets/ubuntu-font-license-1.0.txt) (see https://ubuntu.com/legal/font-licence) and NOT covered by the above lincenses.
