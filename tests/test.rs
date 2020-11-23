use socketpair::socketpair_stream;
use std::{
    io::{self, Read, Write},
    str, thread,
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
