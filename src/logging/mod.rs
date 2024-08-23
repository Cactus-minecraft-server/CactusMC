use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

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

    builder.filter_level(log_level);

    builder.init();
}
