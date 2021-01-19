use socketpair::socketpair_stream;
use std::{
    io::{self, Read, Write},
    str,
    sync::{Arc, Condvar, Mutex},
    thread,
};

#[test]
fn test() -> anyhow::Result<()> {
    let (mut a, mut b) = socketpair_stream()?;

    let thread_a = thread::spawn(move || -> anyhow::Result<()> {
        writeln!(a, "hello world")?;

        let mut buf = [0u8; 4096];
        let n = a.read(&mut buf)?;
        assert_eq!(str::from_utf8(&buf[..n]).unwrap(), "greetings\n");

        writeln!(a, "goodbye")?;
        Ok(())
    });

    let thread_b = thread::spawn(move || -> anyhow::Result<()> {
        let mut buf = [0u8; 4096];
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
    let (mut a, mut b) = socketpair_stream()?;

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

    let (mut a, mut b) = socketpair_stream()?;

    let _t = thread::spawn(move || -> io::Result<()> {
        let (lock, cvar) = &*pair_clone;
        let mut started = lock.lock().unwrap();

        let n = writeln!(a, "hello world")?;

        *started = true;
        cvar.notify_one();
        Ok(n)
    });

    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }

    let mut buf = vec![0u8; 11];
    assert_eq!(b.peek(&mut buf)?, 11);
    assert_eq!(str::from_utf8(&buf).unwrap(), "hello world");

    let mut buf = String::new();
    b.read_to_string(&mut buf)?;
    assert_eq!(buf, "hello world\n");

    Ok(())
}

#[test]
fn try_clone() -> anyhow::Result<()> {
    let (mut a, mut b) = socketpair_stream()?;

    let _t = thread::spawn(move || -> io::Result<()> { writeln!(a, "hello world") });

    let mut c = b.try_clone()?;

    let mut buf = vec![0u8; 6];
    b.read_exact(&mut buf)?;
    assert_eq!(str::from_utf8(&buf).unwrap(), "hello ");
    c.read_exact(&mut buf)?;
    assert_eq!(str::from_utf8(&buf).unwrap(), "world\n");

    Ok(())
}
