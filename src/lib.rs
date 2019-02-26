#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, Shell, SubCommand};
use image::Luma;
use qrcode;
use qrcode::{EcLevel, QrCode};
use std::fs;
use std::io::prelude::*;
use std::process::exit;
use term;
use term::color;

mod payloads;

pub const WIFI_COMMAND: &str = "wifi";
pub const MAIL_COMMAND: &str = "mail";
pub const SMS_COMMAND: &str = "sms";
pub const MMS_COMMAND: &str = "mms";
pub const GEO_COMMAND: &str = "geo";
pub const PHONE_COMMAND: &str = "phone";
pub const SKYPE_COMMAND: &str = "skype";
pub const WHATSAPP_COMMAND: &str = "whatsapp";
pub const URL_COMMAND: &str = "url";
pub const BOOKMARK_COMMAND: &str = "bookmark";
pub const BITCOIN_COMMAND: &str = "bitcoin";
// const GIRO_COMMAND: &'static str = "giro";
// const CALENDAR_COMMAND: &'static str = "calendar";
// const CONTACT_COMMAND: &'static str = "contact";

#[derive(Debug)]
pub struct Parameters {
    pub safe_zone: bool,
    pub output: String,
    pub payload: String,
    pub error: EcLevel,
    pub input: String,
    pub completions: Completions,
    pub command: String,
}

impl Parameters {
    pub fn new() -> Parameters {
        Parameters {
            safe_zone: false,
            output: "".to_string(),
            payload: "".to_string(),
            error: EcLevel::H,
            input: "".to_string(),
            completions: Completions::new(),
            command: "".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Completions {
    pub comp_dir: String,
    pub shell: String,
}
impl Completions {
    pub fn new() -> Completions {
        Completions {
            comp_dir: "".to_string(),
            shell: "".to_string(),
        }
    }
}

/*TODO: add arguments for:
- Add help texts for the subcommands
- Add error texts for some exit branches (mostly file output and qrcode gen)
- implement different types of qr payloads
- write unit tests for the payloads
- write integration tests for edge case inputs
*/
pub fn generate(data: &Parameters) {
    //TODO: catch the possible errors
    let code = QrCode::with_error_correction_level(&data.payload, data.error).unwrap();

    // are we drawing to the terminal or to a file?
    if data.output.len() == 1 {
        save(&code, data.safe_zone, &data.output);
    } else {
        draw(&code, data.safe_zone)
    }

    // shall we also print the payload to the screen?
    if data.payload.len() > 0 {
        println!("{:?}", data.payload);
    }
}

// save to a file at the path
fn save(code: &QrCode, safe: bool, path: &str) {
    // render to a image struct
    let image = code.render::<Luma<u8>>().quiet_zone(safe).build();

    // save the image
    match image.save(path) {
        Ok(..) => println!("Image successfully saved to: {:?}", path),
        Err(e) => println!(
            "Tried to create file but there was a problem: {:?}",
            if let Some(inner_err) = e.get_ref() {
                inner_err.to_string()
            } else {
                e.to_string()
            }
        ),
    };
}

// draw to the terminal
fn draw(code: &QrCode, safe: bool) {
    // get "bit" array
    let bit_array = code.to_colors();

    // get the terminal output pipe
    let mut t = term::stdout().unwrap();

    // get the code width and add extra space for the safe zone
    let w = code.width();
    let wide = w + 6;

    // draw the first white safe zone
    if safe {
        for a in 1..(wide * 3) + 1 {
            t.bg(color::BRIGHT_WHITE).unwrap();
            write!(t, "  ").unwrap();
            if a % wide == 0 {
                t.reset().unwrap();
                writeln!(t, "").unwrap();
            }
        }
    }

    // main drawing loop
    for (i, item) in bit_array.iter().enumerate() {
        // left safe zone
        if safe && i % w == 0 {
            t.bg(color::BRIGHT_WHITE).unwrap();
            write!(t, "      ").unwrap();
        }

        // draw black or white blocks
        if *item == qrcode::Color::Dark {
            t.bg(color::BLACK).unwrap();
            write!(t, "  ").unwrap();
        } else {
            t.bg(color::BRIGHT_WHITE).unwrap();
            write!(t, "  ").unwrap();
        }

        if (i + 1) % w == 0 {
            if safe {
                // draw right safe zone
                t.bg(color::BRIGHT_WHITE).unwrap();
                write!(t, "      ").unwrap();
            }
            t.reset().unwrap();
            writeln!(t, "").unwrap();
        }
    }

    // draw the last white safe zone
    if safe {
        for a in 1..(wide * 3) + 1 {
            t.bg(color::BRIGHT_WHITE).unwrap();
            write!(t, "  ").unwrap();
            if a % wide == 0 {
                t.reset().unwrap();
                writeln!(t, "").unwrap();
            }
        }
    }

    // reset to normal color and flush write buffer
    t.reset().unwrap();
    t.flush().unwrap();
}
