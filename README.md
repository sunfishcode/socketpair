<div align="center">
  <h1><code>socketpair</code></h1>

  <p>
    <strong>Cross-platform socketpair functionality</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/socketpair/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/socketpair/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://crates.io/crates/socketpair"><img src="https://img.shields.io/crates/v/socketpair.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/socketpair"><img src="https://docs.rs/socketpair/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

This crate wraps [`socketpair`] with `AF_UNIX` platforms, and emulates this
interface using `CreateNamedPipe` on Windows.

It has a "stream" interface, which corresponds to `SOCK_STREAM` and
`PIPE_TYPE_BYTE`, and a "seqpacket" interface, which corresponds to
`SOCK_SEQPACKET` and `PIPE_TYPE_MESSAGE`.

## Example

```rust
let (mut a, mut b) = socketpair_stream()?;

writeln!(a, "hello world")?;

let mut buf = [0_u8; 4096];
let n = b.read(&mut buf)?;
assert_eq!(str::from_utf8(&buf[..n]).unwrap(), "hello world\n");
```

Support for async-std and tokio is temporarily disabled until those crates
contain the needed implementations of the I/O safety traits.

[`socketpair`]: https://man7.org/linux/man-pages/man2/socketpair.2.html
