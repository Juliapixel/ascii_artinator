use actix_web::{get, Responder};
use log::{error, info};
use make_it_braille::{braille::BrailleImg, dithering::{Bayer4x4, Ditherer, None, Sierra2Row}};
use serde::Deserialize;

fn resize_img(img: image::DynamicImage, width: Option<u32>, height: Option<u32>) -> image::DynamicImage {
    let target_width = width.unwrap_or(58);
    let target_height = if let Some(h) = height {
        h
    } else {
        let aspect_ratio = img.width() as f32 / img.height() as f32;
        (target_width as f32 / aspect_ratio) as u32
    };
    return img.resize(target_width, target_height, image::imageops::Triangle);
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Dithering {
    #[default]
    Sierra2,
    Bayer,
    None
}

impl Dithering {
    pub fn to_ditherer(&self) -> &dyn Ditherer {
        match self {
            Dithering::Sierra2 => &Sierra2Row,
            Dithering::Bayer => &Bayer4x4,
            Dithering::None => &None,
        }
    }
}

#[derive(Deserialize)]
struct Request {
    img_url: String,
    line_break: Option<bool>,
    width: Option<u32>,
    height: Option<u32>,
    limit: Option<bool>,
    allow_empty: Option<bool>,
    #[serde(default)]
    dithering: Dithering
}

#[get("/braille")]
pub async fn braille(req: actix_web::web::Query<Request>) -> impl Responder {
    info!("requesting braille from: {}", req.img_url);
    match reqwest::get(&req.img_url).await {
        Ok(resp) => {
            let mime_type = resp.headers().get("content-type");
            if let Some(mime_type) = mime_type {
                if let Some(img_format) = image::ImageFormat::from_mime_type(mime_type.to_str().unwrap()) {
                    match image::load_from_memory_with_format(&resp.bytes().await.unwrap(), img_format) {
                        Ok(img) => {
                            let ascii = BrailleImg::from_image(resize_img(img, req.width, req.height), req.dithering.to_ditherer(), true)
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
