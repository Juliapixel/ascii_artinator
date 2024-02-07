use std::net::Ipv4Addr;

use actix_web::middleware::{Logger, DefaultHeaders};
use flexi_logger::{LogSpecification, FileSpec, LoggerHandle, DeferredNow, style};
use log::Record;
use reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN;

mod zoazo;
mod braille;

fn colored_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "[{}] {} {}",
        now.format(flexi_logger::TS_DASHES_BLANK_COLONS_DOT_BLANK).to_string(),
        style(level).paint(level.to_string()),
        &record.args().to_string()
    )
}

fn init_log() -> LoggerHandle {
    let log_spec;
    #[cfg(debug_assertions)]
    {
        log_spec = LogSpecification::builder()
            .default(log::LevelFilter::Debug)
            .build();
    }
    #[cfg(not(debug_assertions))]
    {
        log_spec = LogSpecification::builder()
            .default(log::LevelFilter::Info)
            .build();
    }

    let mut log_dir = std::env::current_exe().unwrap();
    log_dir.pop();
    log_dir.push("ascii_artinator_log");
    return flexi_logger::Logger::with(log_spec)
        .format(colored_format)
        .log_to_file(FileSpec::default().directory(log_dir))
        .rotate(
            flexi_logger::Criterion::Age(flexi_logger::Age::Day),
            flexi_logger::Naming::Timestamps,
            flexi_logger::Cleanup::KeepCompressedFiles(7)
        )
        .duplicate_to_stdout(flexi_logger::Duplicate::All)
        .write_mode(flexi_logger::WriteMode::Async)
        .print_message()
        .use_utc()
        .start()
        .unwrap();
}

const BIND_ADDRESS: (Ipv4Addr, u16) = {
    #[cfg(not(debug_assertions))]
    {
        (Ipv4Addr::UNSPECIFIED, 10034)
    }
    #[cfg(debug_assertions)]
    {
        (Ipv4Addr::LOCALHOST, 10035)
    }
};

#[actix_web::main]
async fn main() {
    let _logger = init_log();

    actix_web::HttpServer::new(||
        actix_web::App::new()
            .wrap(Logger::default())
            .wrap(DefaultHeaders::new().add((ACCESS_CONTROL_ALLOW_ORIGIN, "*")))
            .service(braille::braille)
            .service(zoazo::zoazo)
    ).bind(BIND_ADDRESS)
    .unwrap().run().await.unwrap();
}
