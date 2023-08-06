use std::env;

extern crate chrono;
extern crate fern;

pub fn setup_logger() -> Result<String, fern::InitError> {
    let temp_dir = env::temp_dir();
    let log_file = temp_dir.join("server.log");
    let log_file_path = log_file.as_path();
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_file_path)?)
        .apply()?;
    Ok(log_file_path.display().to_string())
}
