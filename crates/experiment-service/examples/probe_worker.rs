//! Controlled external process for service failure/resource demonstrations.
//! It does not provide numerical acceptance evidence; the real CLI is used separately.
use std::{
    fs,
    io::{self, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    let value = |key: &str| -> Result<PathBuf, io::Error> {
        args.windows(2)
            .find(|pair| pair[0] == key)
            .map(|pair| PathBuf::from(&pair[1]))
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, key.to_owned()))
    };
    let seed = value("--seed")?;
    let output = value("--output")?;
    let parent = output
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing parent"))?;
    if seed == Path::new("9002") {
        let bytes = [b'a'; 65536];
        for _ in 0..32 {
            io::stdout().write_all(&bytes)?;
        }
        io::stdout().flush()?;
        let bytes = [b'b'; 65536];
        for _ in 0..32 {
            io::stderr().write_all(&bytes)?;
        }
        io::stderr().flush()?;
        std::process::exit(7);
    }
    let listener = TcpListener::bind("127.0.0.1:0")?;
    fs::write(
        parent.join("worker.ready"),
        format!(
            "{{\"pid\":{},\"address\":\"{}\"}}",
            std::process::id(),
            listener.local_addr()?
        ),
    )?;
    if seed == Path::new("9003") {
        let mut file = fs::File::create(output)?;
        let bytes = vec![b'x'; 1024 * 1024];
        for _ in 0..257 {
            file.write_all(&bytes)?;
        }
        file.flush()?;
    }
    thread::sleep(Duration::from_secs(180));
    drop(listener);
    std::process::exit(9);
}
