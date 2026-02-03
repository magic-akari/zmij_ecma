# Żmij ECMA
[<img alt="github" src="https://img.shields.io/badge/github-magic_akari/zmij_ecma-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/magic-akari/zmij_ecma)
[<img alt="crates.io" src="https://img.shields.io/crates/v/zmij_ecma.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/zmij_ecma)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-zmij_ecma-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/zmij_ecma)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/magic-akari/zmij_ecma/ci.yml?branch=ecma&style=for-the-badge" height="20">](https://github.com/magic-akari/zmij_ecma/actions?query=branch%3Aecma)

Żmij ECMA is a fork of the [Żmij][zmij-crate] crate adjusted to comply with the ECMAScript [number-to-string][number-to-string] algorithm.

[zmij-crate]: https://crates.io/crates/zmij
[number-to-string]: https://tc39.es/ecma262/#sec-numeric-types-number-tostring

---

Pure Rust implementation of Żmij, an algorithm to quickly convert floating point
numbers to decimal strings.

This Rust implementation is a line-by-line port of Victor Zverovich's
implementation in C++, [https://github.com/vitaut/zmij][upstream].

[upstream]: https://github.com/vitaut/zmij/tree/d8cb94516d0c480d6d70851ca677beae6ba721fc

## Example

```rust
fn main() {
    let mut buffer = zmij_ecma::Buffer::new();
    let printed = buffer.format(1.234);
    assert_eq!(printed, "1.234");
}
```

## Performance

The [dtoa-benchmark] compares this library and other Rust floating point
formatting implementations across a range of precisions. The vertical axis in
this chart shows nanoseconds taken by a single execution of
`zmij_ecma::Buffer::new().format_finite(value)` so a lower result indicates a faster
library.

[dtoa-benchmark]: https://github.com/dtolnay/dtoa-benchmark

![performance](https://raw.githubusercontent.com/dtolnay/zmij/master/dtoa-benchmark.png)

<br>

#### License

<a href="LICENSE-MIT">MIT license</a>.
