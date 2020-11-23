<div align="center">
  <h1><code>socketpair</code></h1>

  <p>
    <strong>Cross-platform socketpair functionality</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/socketpair<img src="https://github.com/sunfishcode/socketpair/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://crates.io/crates/socketpair"><img src="https://img.shields.io/crates/v/socketpair.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/socketpair"><img src="https://docs.rs/socketpair/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

This crate wraps [`socketpair`] with `AF_UNIX` and `SOCK_STREAM` on Posix-ish
platforms, and emulates this interface using `CreateNamedPipe` on Windows.

[`socketpair`]: https://man7.org/linux/man-pages/man2/socketpair.2.html
