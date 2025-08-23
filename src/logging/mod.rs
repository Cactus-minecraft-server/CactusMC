use env_logger::{Builder, Env, Target};
use log::LevelFilter;
use std::fs::File;
use std::io::{self, Write};
struct Tee<W1: Write, W2: Write> {
    a: W1,
    b: W2,
}

impl<W1: Write, W2: Write> Write for Tee<W1, W2> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.a.write(buf)?;
        let _ = self.b.write(buf)?;
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.a.flush()?;
        self.b.flush()
    }
}

pub fn init(log_level: LevelFilter, to_file: bool) {
    // TODO fix the colors when saving to file
    if to_file {
        let file = File::create("logs/latest.log").unwrap();
        let stderr = io::stderr();
        let tee = Tee { a: file, b: stderr };

        Builder::from_env(Env::default())
            .target(Target::Pipe(Box::new(tee)))
            .filter_level(log_level)
            .init();
        return;
    }
    let mut builder = Builder::new();
    builder.filter_level(log_level);
    builder.init();
}
