use socketpair::socketpair_seqpacket;
use std::io::{self, Read, Write};
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
/// because one write is paired with two reads. We get an unexpected EOF trying to
/// read to the end.
#[test]
fn try_clone() -> anyhow::Result<()> {
    let (mut a, mut b) = socketpair_seqpacket()?;

    let _t = thread::spawn(move || -> io::Result<()> { write!(a, "hello world") });

    let mut c = b.try_clone()?;

    let mut buf = vec![0_u8; 6];
    b.read_exact(&mut buf)?;
    assert_eq!(str::from_utf8(&buf).unwrap(), "hello ");
    assert_eq!(c.read_exact(&mut buf).unwrap_err().kind(), io::ErrorKind::UnexpectedEof);

    Ok(())
}

/// Like `try_clone` but use two writes so that it works with seqpacket.
#[test]
fn try_clone_two_writes() -> anyhow::Result<()> {
    let (mut a, mut b) = socketpair_seqpacket()?;

    let _t = thread::spawn(move || -> io::Result<()> { write!(a, "hello ")?; writeln!(a, "world") });

    let mut c = b.try_clone()?;

    let mut buf = vec![0_u8; 6];
    b.read_exact(&mut buf)?;
    assert_eq!(str::from_utf8(&buf).unwrap(), "hello ");
    c.read_exact(&mut buf)?;
    assert_eq!(str::from_utf8(&buf).unwrap(), "world\n");

    Ok(())
}
