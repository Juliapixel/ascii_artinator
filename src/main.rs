use actix_web::{Responder, get};
use image::{DynamicImage, GenericImageView};
use serde::Deserialize;

fn resize_from_path(path: &str) -> image::DynamicImage {
    let img = image::open(path).unwrap();
    let aspect_ratio = img.width() as f32 / img.height() as f32;
    let target_height = (58.0 / aspect_ratio) as u32;
    return img.resize(58, target_height, image::imageops::Triangle);
}

fn resize_img(img: image::DynamicImage) -> image::DynamicImage {
    let aspect_ratio = img.width() as f32 / img.height() as f32;
    let target_height = (58.0 / aspect_ratio) as u32;
    return img.resize(58, target_height, image::imageops::Triangle);
}

fn img_to_braille(img: DynamicImage) -> String {
    let braille_img = String::new();
    let mut gray_img = image::GrayImage::new(img.width(), img.height());

    let compute_lightness = |rgba: &[f32; 4]| -> u8 {
        return ((rgba[0] * 0.2126 + rgba[1] * 0.7152 + rgba[2] * 0.0722) * 255.0 * rgba[3])
          .clamp(0.0, 255.0)
          .round() as u8;
    };

    for (x, y, pix) in img.pixels() {
        let lightness = compute_lightness(
            &[
            pix.0[0] as f32 / 255.0,
            pix.0[1] as f32 / 255.0,
            pix.0[2] as f32 / 255.0,
            pix.0[3] as f32 / 255.0
            ]
        );
        gray_img.put_pixel(x, y, image::Luma::<u8>([lightness]));
    }

    #[cfg(debug_assertions)]
    gray_img.save("gray.png");

    let add_error = |img: &mut image::GrayImage, x: Option<u32>, y: Option<u32>, err: &i32, importance: i32| {
        if let Some(xpos) = x {
            if let Some(ypos) = y {
                if let Some(pix) = img.get_pixel_mut_checked(xpos, ypos) {
                    *pix = image::Luma([(pix.0[0] as i32 + err * importance).clamp(0, 255) as u8]);
                }
            }
        }
    };

    for y in 0..gray_img.height() {
        for x in 0..gray_img.width() {
            let cur_pix = gray_img.get_pixel_mut(x, y);
            let error = if cur_pix.0[0] > 127 {
                cur_pix.0[0] as i32 - 255
            } else {
                cur_pix.0[0] as i32
            } >> 5;
            if cur_pix.0[0] > 127 {
                cur_pix.0[0] = 255;
            } else {
                cur_pix.0[0] = 0;
            }

            let xpos = x as i64;
            let ypos = y as i64;

            add_error(&mut gray_img, x.checked_add(1), Some(y)    , &error, 5);
            add_error(&mut gray_img, x.checked_add(2), Some(y)    , &error, 3);
            add_error(&mut gray_img, x.checked_sub(2), Some(y + 1), &error, 2);
            add_error(&mut gray_img, x.checked_sub(1), Some(y + 1), &error, 4);
            add_error(&mut gray_img, Some(x)         , Some(y + 1), &error, 5);
            add_error(&mut gray_img, x.checked_add(1), Some(y + 1), &error, 4);
            add_error(&mut gray_img, x.checked_add(2), Some(y + 1), &error, 2);
            add_error(&mut gray_img, x.checked_sub(1), Some(y + 2), &error, 2);
            add_error(&mut gray_img, Some(x)         , Some(y + 2), &error, 3);
            add_error(&mut gray_img, x.checked_add(1), Some(y + 2), &error, 2);
        }
    }

    #[cfg(debug_assertions)]
    gray_img.save("dithered.png");

    let mut braille_img = BrailleImg::new(gray_img.width(), gray_img.height());
    for (x, y, pix) in gray_img.enumerate_pixels() {
        if pix.0[0] > 80{
            braille_img.set_dot(x, y, true);
        }
    }
    return braille_img.to_str(true);
}



const BRAILLE_CHARS: [char; 256] = [
'⠀', '⠁', '⠂', '⠃', '⠄', '⠅', '⠆', '⠇', '⠈', '⠉', '⠊', '⠋', '⠌', '⠍', '⠎', '⠏', '⠐', '⠑', '⠒', '⠓', '⠔', '⠕', '⠖', '⠗', '⠘', '⠙',
'⠚', '⠛', '⠜', '⠝', '⠞', '⠟', '⠠', '⠡', '⠢', '⠣', '⠤', '⠥', '⠦', '⠧', '⠨', '⠩', '⠪', '⠫', '⠬', '⠭', '⠮', '⠯', '⠰', '⠱', '⠲',
'⠳', '⠴', '⠵', '⠶', '⠷', '⠸', '⠹', '⠺', '⠻', '⠼', '⠽', '⠾', '⠿', '⡀', '⡁', '⡂', '⡃', '⡄', '⡅', '⡆', '⡇', '⡈', '⡉', '⡊', '⡋',
'⡌', '⡍', '⡎', '⡏', '⡐', '⡑', '⡒', '⡓', '⡔', '⡕', '⡖', '⡗', '⡘', '⡙', '⡚', '⡛', '⡜', '⡝', '⡞', '⡟', '⡠', '⡡', '⡢', '⡣', '⡤',
'⡥', '⡦', '⡧', '⡨', '⡩', '⡪', '⡫', '⡬', '⡭', '⡮', '⡯', '⡰', '⡱', '⡲', '⡳', '⡴', '⡵', '⡶', '⡷', '⡸', '⡹', '⡺', '⡻', '⡼', '⡽',
'⡾', '⡿', '⢀', '⢁', '⢂', '⢃', '⢄', '⢅', '⢆', '⢇', '⢈', '⢉', '⢊', '⢋', '⢌', '⢍', '⢎', '⢏', '⢐', '⢑', '⢒', '⢓', '⢔', '⢕', '⢖',
'⢗', '⢘', '⢙', '⢚', '⢛', '⢜', '⢝', '⢞', '⢟', '⢠', '⢡', '⢢', '⢣', '⢤', '⢥', '⢦', '⢧', '⢨', '⢩', '⢪', '⢫', '⢬', '⢭', '⢮', '⢯',
'⢰', '⢱', '⢲', '⢳', '⢴', '⢵', '⢶', '⢷', '⢸', '⢹', '⢺', '⢻', '⢼', '⢽', '⢾', '⢿', '⣀', '⣁', '⣂', '⣃', '⣄', '⣅', '⣆', '⣇', '⣈',
'⣉', '⣊', '⣋', '⣌', '⣍', '⣎', '⣏', '⣐', '⣑', '⣒', '⣓', '⣔', '⣕', '⣖', '⣗', '⣘', '⣙', '⣚', '⣛', '⣜', '⣝', '⣞', '⣟', '⣠', '⣡',
'⣢', '⣣', '⣤', '⣥', '⣦', '⣧', '⣨', '⣩', '⣪', '⣫', '⣬', '⣭', '⣮', '⣯', '⣰', '⣱', '⣲', '⣳', '⣴', '⣵', '⣶', '⣷', '⣸', '⣹', '⣺',
'⣻', '⣼', '⣽', '⣾', '⣿'
];

struct BrailleImg {
    braille_vals: Vec<u8>,
    dot_width: u32,
    dot_height: u32,
    char_width: u32,
    char_height: u32,
}

impl BrailleImg {
    pub fn new(width: u32, height: u32) -> Self {
        let mut vals: Vec<u8> = Vec::new();
        let x_size = width / 2 + (width % 2);
        let extra_row = if height % 4 != 0 {
            1
        } else {
            0
        };
        let y_size = height / 4 + extra_row;
        vals.reserve((x_size * y_size) as usize);
        for _ in 0..x_size * y_size {
            vals.push(0);
        }
        BrailleImg {
            braille_vals: vals,
            dot_width: width,
            dot_height: height,
            char_width: x_size,
            char_height: y_size,
        }
    }

    fn get_bit_mask(x: u32, y: u32) -> u8 {
        if x % 2 == 0 {
            match y % 4 {
                0 => 0b00000001,
                1 => 0b00000010,
                2 => 0b00000100,
                _ => 0b01000000
            }
        } else {
            match y % 4 {
                0 => 0b00001000,
                1 => 0b00010000,
                2 => 0b00100000,
                _ => 0b10000000
            }
        }
    }

    pub fn set_dot(&mut self, x: u32, y: u32, raised: bool) {
        let x_val_pos = x / 2;
        let y_val_pos = y / 4;
        let val = self.braille_vals.get_mut((x_val_pos + y_val_pos * self.char_width) as usize).unwrap();
        let mask = BrailleImg::get_bit_mask(x, y);
        if raised {
            *val |= mask;
        } else {
            *val &= !mask;
        }
    }

    pub fn to_str(self, no_empty_chars: bool) -> String {
        let mut braille_string = String::new();
        for (i, val) in self.braille_vals.into_iter().enumerate() {
            if i % self.char_width as usize == 0 {
                braille_string.push('\n');
            }
            if val == 0 && no_empty_chars {
                braille_string.push(BRAILLE_CHARS[1])
            } else {
                braille_string.push(BRAILLE_CHARS[val as usize])
            }
        }
        return braille_string
    }
}

#[derive(Deserialize)]
struct Request {
    img_url: String
}

#[get("/braille")]
async fn braille(req: actix_web::web::Query<Request>) -> impl Responder {
    println!("{}: {}", chrono::Utc::now(), req.img_url);
    match reqwest::get(&req.img_url).await {
        Ok(resp) => {
            if let Some(img_format) = image::ImageFormat::from_mime_type(resp.headers().get("content-type").unwrap().to_str().unwrap()) {
                match image::load_from_memory_with_format(&resp.bytes().await.unwrap(), img_format) {
                    Ok(img) => img_to_braille(resize_img(img)),
                    Err(_) => "failed to read image INSANECAT".to_owned(),
                }
            } else {
                return "link is not an image KEEEEEEEEEK".to_owned()
            }
        },
        Err(_) => "failed to request image S OMEGALUL BAD".to_owned()
    }
}

#[actix_web::main]
async fn main() {
    #[cfg(not(debug_assertions))]
    actix_web::HttpServer::new(||
        actix_web::App::new()
            .service(braille)
    ).bind(("0.0.0.0", 10034))
    .unwrap().run().await.unwrap();
    #[cfg(debug_assertions)]
    actix_web::HttpServer::new(||
        actix_web::App::new()
            .service(braille)
    ).bind(("127.0.0.1", 10035))
    .unwrap().run().await.unwrap();
}
