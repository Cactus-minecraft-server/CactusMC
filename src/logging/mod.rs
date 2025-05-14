use std::{
    fs::{File, OpenOptions},
    os::fd::AsRawFd,
};

use env_logger::Builder;
use log::LevelFilter;

/// Initializes the logging for the whole application
pub fn init(log_level: LevelFilter) {
    let mut builder = Builder::new();

    // TODO: Customize logging format. Making the logging level the right color is time consuming.

    //builder.format(|buf, record| {
    //    writeln!(
    //        buf,
    //        "[{}] [{}] [{}] {}",
    //        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
    //        record.level(),
    //        record.target(),
    //        record.args()
    //    )
    //});

    // And add use::io::Write; for the above code.

    builder.filter_level(log_level);

    builder.init();
}
