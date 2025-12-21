use flexi_logger::{Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming, WriteMode};

use log::LevelFilter;

use crate::fs_manager;
/// TODO: resolve the issue where info doesn't have color (minor issue)
pub fn init(log_level: LevelFilter, to_file: bool) {
    // Call init ONLY once else result in panic

    if to_file {
        fs_manager::create_dirs();

        Logger::try_with_str(log_level.as_str())
            .unwrap()
            .log_to_file(
                FileSpec::default()
                    .directory("logs")
                    .basename("app")
                    .suffix("log"),
            )
            .rotate(
                Criterion::AgeOrSize(Age::Day, 10_000_000),
                Naming::Timestamps,
                Cleanup::KeepLogFiles(10),
            )
            .duplicate_to_stderr(Duplicate::All)
            .write_mode(WriteMode::BufferAndFlush)
            .start()
            .unwrap();

        return;
    }

    Logger::try_with_str(log_level.as_str())
        .unwrap()
        .duplicate_to_stderr(Duplicate::All)
        .format_for_stderr(flexi_logger::colored_default_format)
        .write_mode(WriteMode::BufferAndFlush)
        .start()
        .unwrap();
}
