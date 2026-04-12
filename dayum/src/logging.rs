use flexi_logger::{FileSpec, FlexiLoggerError, Logger};

pub fn init_logger() -> Result<(), FlexiLoggerError> {
    Logger::try_with_str("info")?
        .log_to_file(
            FileSpec::default()
                .directory("logs")
                .basename("app")
                .suffix("log")
        )
        .rotate(
            flexi_logger::Criterion::Size(10_000_000),
            flexi_logger::Naming::Timestamps,
            flexi_logger::Cleanup::KeepLogFiles(20),
        )
        .start()?;

    Ok(())
}

pub fn init_logger_for_tests() -> Result<(), FlexiLoggerError> {
    Logger::try_with_str("info")?
        .log_to_stdout()
        .start()?;

    Ok(())
}
