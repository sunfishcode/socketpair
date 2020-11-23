//! Cross-platform socketpair functionality
//!
//! A socketpair stream is a bidirectional bytestream. The `socketpair_stream`
//! function creates a pair of `SocketpairStream` objects connected to opposite
//! ends of a stream, and both can be written to and read from.
//!
//! ```rust
//! use socketpair::socketpair_stream;
//! use std::{
//!     io::{self, Read, Write},
//!     thread,
//! };
//!
//! fn main() -> anyhow::Result<()> {
//!     let (mut a, mut b) = socketpair_stream()?;
//!
//!     let _t = thread::spawn(move || -> io::Result<()> { writeln!(a, "hello world") });
//!
//!     let mut buf = String::new();
//!     b.read_to_string(&mut buf)?;
//!     assert_eq!(buf, "hello world\n");
//!
//!     Ok(())
//! }
//! ```

#![deny(missing_docs)]
#![cfg_attr(can_vector, feature(can_vector))]
#![cfg_attr(write_all_vectored, feature(write_all_vectored))]

#[cfg(not(windows))]
mod posish;
#[cfg(windows)]
mod winx;

#[cfg(not(windows))]
pub use crate::posish::{socketpair_stream, SocketpairStream};
#[cfg(windows)]
pub use crate::winx::{socketpair_stream, SocketpairStream};
