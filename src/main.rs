use actix_web::{Responder, get};
use flexi_logger::{LogSpecification, FileSpec, LoggerHandle, DeferredNow, style};
use log::{info, error, Record};
use ascii_artinator::commons::braille_img::BrailleImg;
use rand::Rng;
use serde::Deserialize;

fn resize_img(img: image::DynamicImage, width: Option<u32>, height: Option<u32>) -> image::DynamicImage {
    let mut target_width = 58;
    if let Some(w) = width {
        target_width = w;
    }
    let target_height;
    if let Some(h) = height {
        target_height = h;
    } else {
        let aspect_ratio = img.width() as f32 / img.height() as f32;
        target_height = (target_width as f32 / aspect_ratio) as u32;
    }
    return img.resize(target_width, target_height, image::imageops::Triangle);
}

#[derive(Deserialize)]
struct Request {
    img_url: String,
    line_break: Option<bool>,
    width: Option<u32>,
    height: Option<u32>,
    limit: Option<bool>,
    allow_empty: Option<bool>,
}

#[get("/braille")]
async fn braille(req: actix_web::web::Query<Request>) -> impl Responder {
    info!("requesting braille from: {}", req.img_url);
    match reqwest::get(&req.img_url).await {
        Ok(resp) => {
            let mime_type = resp.headers().get("content-type");
            if let Some(mime_type) = mime_type {
                if let Some(img_format) = image::ImageFormat::from_mime_type(mime_type.to_str().unwrap()) {
                    match image::load_from_memory_with_format(&resp.bytes().await.unwrap(), img_format) {
                        Ok(img) => {
                            let ascii = BrailleImg::from_image(resize_img(img, req.width, req.height).to_rgba8())
                                .to_str(!req.allow_empty.unwrap_or(false), req.line_break.unwrap_or(true));
                            if ascii.chars().count() > 500 && req.limit.unwrap_or(false) {
                                error!("image requested too tall.");
                                return "image too tall".to_owned()
                            } else {
                                return ascii
                            }
                        },
                        Err(_) => {
                            error!("failed to read requested image.");
                            "failed to read image.".to_owned()
                        },
                    }
                } else {
                    error!("link is not an image or bad request. MIME type: {}", mime_type.to_str().unwrap());
                    return "link is not an image.".to_owned()
                }
            } else {
                error!("link is not an image or bad request. MIME type not found");
                return "link is not an image.".to_owned()
            }

        },
        Err(_) => {
            error!("failed to request to url");
            "failed to request image.".to_owned()
        }
    }
}

const WORDS: &'static str = include_str!("../10000-english-no-swears.txt");

fn load_words() -> Vec<String> {
    WORDS
        .lines()
        .map(|c| c.to_owned())
        .collect()
}

lazy_static::lazy_static! {
    static ref WORD_LIST: Vec<String> = load_words();
}

fn generate_zoazo() -> String{
    let mut rng = rand::thread_rng();
    let zoazo_short = rng.gen_bool(0.5);
    let zoazo_len = if zoazo_short { 1 } else { rng.gen_range(2..=5) };
    let mut zoazo_words: Vec<&str> = Vec::with_capacity(zoazo_len);

    (0..zoazo_len).into_iter().for_each(|_| {zoazo_words.push(&WORD_LIST[rng.gen_range(0..WORD_LIST.len())])});

    let mut zoazo_emote = String::with_capacity(5 + zoazo_words.iter().map(|w| w.len()).sum::<usize>());
    zoazo_emote += "zoazo";
    for w in zoazo_words.into_iter() {
        w.chars().enumerate().for_each(|(i, c)| {
            if i == 0 {
                zoazo_emote.push(c.to_ascii_uppercase())
            } else {
                zoazo_emote.push(c)
            }});
    };

    return zoazo_emote;
}

#[get("/zoazo")]
async fn zoazo() -> impl Responder {
    let zoazo_emote = generate_zoazo();
    info!("zoazoEmote: {}", zoazo_emote);
    return zoazo_emote;
}

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

#[actix_web::main]
async fn main() {
    let _logger = init_log();

    #[cfg(not(debug_assertions))]
    {
        actix_web::HttpServer::new(||
            actix_web::App::new()
                .service(braille)
                .service(zoazo)
        ).bind(("0.0.0.0", 10034))
        .unwrap().run().await.unwrap();
    }

    #[cfg(debug_assertions)]
    {
        actix_web::HttpServer::new(||
            actix_web::App::new()
                .service(braille)
                .service(zoazo)
        ).bind(("127.0.0.1", 10035))
        .unwrap().run().await.unwrap();
    }
}
