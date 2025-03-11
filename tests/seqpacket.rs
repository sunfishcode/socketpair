#![cfg(not(any(target_os = "ios", target_os = "macos")))]

use socketpair::socketpair_seqpacket;
use std::io::{self, Read, Write};
use std::str::FromStr;
use std::sync::{Arc, Condvar, Mutex};
use std::{str, thread};

#[test]
fn test() -> anyhow::Result<()> {
    let (mut a, mut b) = socketpair_seqpacket()?;

    let thread_a = thread::spawn(move || -> anyhow::Result<()> {
        writeln!(a, "hello world")?;

        let mut buf = [0_u8; 4096];
        let n = a.read(&mut buf)?;
        assert_eq!(str::from_utf8(&buf[..n]).unwrap(), "greetings\n");

        writeln!(a, "goodbye")?;
        Ok(())
    });

    let thread_b = thread::spawn(move || -> anyhow::Result<()> {
        let mut buf = [0_u8; 4096];
        let n = b.read(&mut buf)?;
        assert_eq!(str::from_utf8(&buf[..n]).unwrap(), "hello world\n");

        writeln!(b, "greetings")?;

        let n = b.read(&mut buf)?;
        assert_eq!(str::from_utf8(&buf[..n]).unwrap(), "goodbye\n");
        Ok(())
    });

    thread_a.join().unwrap()?;
    thread_b.join().unwrap()?;
    Ok(())
}

#[test]
fn one_way() -> anyhow::Result<()> {
    let (mut a, mut b) = socketpair_seqpacket()?;

    let _t = thread::spawn(move || -> io::Result<()> { writeln!(a, "hello world") });

    let mut buf = String::new();
    b.read_to_string(&mut buf)?;
    assert_eq!(buf, "hello world\n");

    Ok(())
}

#[test]
fn peek() -> anyhow::Result<()> {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair_clone = Arc::clone(&pair);

    let (mut a, mut b) = socketpair_seqpacket()?;

    let _t = thread::spawn(move || -> io::Result<()> {
        let (lock, cvar) = &*pair_clone;
        let mut started = lock.lock().unwrap();

        writeln!(a, "hello world")?;

        *started = true;
        drop(started);
        cvar.notify_one();
        Ok(())
    });

    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }

    assert_eq!(b.num_ready_bytes()?, 12);

    let mut buf = vec![0_u8; 11];
    assert_eq!(b.peek(&mut buf)?, 11);
    assert_eq!(str::from_utf8(&buf).unwrap(), "hello world");

    let mut buf = String::new();
    b.read_to_string(&mut buf)?;
    assert_eq!(buf, "hello world\n");

    Ok(())
}

/// Like `try_clone` in the stream tests, but this doesn't work with seqpacket
/// because one write is paired with two reads. We get an unexpected EOF trying
/// to read to the end.
#[test]
fn try_clone() -> anyhow::Result<()> {
    let (mut a, mut b) = socketpair_seqpacket()?;

    let _t = thread::spawn(move || -> io::Result<()> { write!(a, "hello world") });

    let mut c = b.try_clone()?;

    let mut buf = vec![0_u8; 6];
    b.read_exact(&mut buf)?;
    assert_eq!(str::from_utf8(&buf).unwrap(), "hello ");
    assert_eq!(
        c.read_exact(&mut buf).unwrap_err().kind(),
        io::ErrorKind::UnexpectedEof
    );

    Ok(())
}

/// Like `try_clone` but use two writes so that it works with seqpacket.
#[test]
fn try_clone_two_writes() -> anyhow::Result<()> {
    let (mut a, mut b) = socketpair_seqpacket()?;

    let _t = thread::spawn(move || -> io::Result<()> {
        write!(a, "hello ")?;
        writeln!(a, "world")
    });

    let mut c = b.try_clone()?;

    let mut buf = vec![0_u8; 6];
    b.read_exact(&mut buf)?;
    assert_eq!(str::from_utf8(&buf).unwrap(), "hello ");
    c.read_exact(&mut buf)?;
    assert_eq!(str::from_utf8(&buf).unwrap(), "world\n");

    Ok(())
}

/// `socketpair_seqpacket` should be reliable. Do a simple test to catch
/// obvious unreliability.
#[test]
fn test_reliable() -> anyhow::Result<()> {
    let (mut a, mut c) = socketpair_seqpacket()?;
    let mut b = a.try_clone()?;

    // Write a bunch of messages from multiple threads, to see if they'll
    // interfere with each other.
    let thread_a = thread::spawn(move || -> anyhow::Result<()> {
        for i in 0..0x8000 {
            write!(a, "{}", format!("thread A: {}", i))?;
        }
        Ok(())
    });
    let thread_b = thread::spawn(move || -> anyhow::Result<()> {
        for i in 0..0x8000 {
            write!(b, "{}", format!("thread B: {}", i))?;
        }
        Ok(())
    });

    // Give the threads some time to fill up the socket buffers, to see
    // if they'll start dropping packets.
    std::thread::sleep(std::time::Duration::from_secs(5));

    // Read the packets, and assert they arrive in the right order.
    let thread_c = thread::spawn(move || -> anyhow::Result<()> {
        let mut buf = [0_u8; 4096];

        let mut expect_a = 0;
        let mut expect_b = 0;
        for _ in 0..0x10000 {
            let n = c.read(&mut buf)?;
            let s = str::from_utf8(&buf[..n]).unwrap();
            dbg!(s);
            if let Some(x) = s.strip_prefix("thread A: ") {
                dbg!(x);
                let new_a = u32::from_str(x).unwrap();
                assert_eq!(new_a, expect_a);
                expect_a = new_a + 1;
            } else if let Some(x) = s.strip_prefix("thread B: ") {
                dbg!(x);
                let new_b = u32::from_str(x).unwrap();
                assert_eq!(new_b, expect_b);
                expect_b = new_b + 1;
            } else {
                unreachable!("unexpected message");
            }
        }
        Ok(())
    });

    thread_c.join().unwrap()?;
    thread_a.join().unwrap()?;
    thread_b.join().unwrap()?;
    Ok(())
}
