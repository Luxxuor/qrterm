extern crate clap;
extern crate qrcode;
extern crate termion;
extern crate image;

use clap::{App, Arg};
use qrcode::{QrCode, EcLevel};
use termion::color;
use termion::style;
use image::GrayImage;

/*TODO: add arguments for:
- safe zone rendering
- ec level
- size of qr code
- implement different types of qr payloads
*/
fn main() {
    let matches = App::new("qrterm")
        .version("0.1")
        .author("Lukas R. <lukas@bootsmann-games.de>")
        .about("Generates and displays terminal friendly QR-Codes from input strings")
        .arg(Arg::with_name("type")
            .short("t")
            .long("type")
            .value_name("TYPE")
            .help("Sets if a special qr type should be used")
            .takes_value(true))
        .arg(Arg::with_name("INPUT")
            .help("The input string to use")
            .required(true)
            .index(1))
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();

    let code = QrCode::with_error_correction_level(input, EcLevel::H).unwrap();

    let image: GrayImage = code.render().to_image();
    image.save("/tmp/qrcode.png").unwrap();

    let bit_array = code.to_vec();

    let w = code.width();
    let wide = w + 6;

    //Draw the first white safe zone
    for a in 1..(wide * 3) + 1 {
        print!("{}  ", color::Bg(color::LightWhite));
        if a % wide == 0 {
            println!("{}", color::Bg(color::Reset));
        }
    }

    for (i, item) in bit_array.iter().enumerate() {
        //left safe zone
        if i % w == 0 {
            print!("{}      ", color::Bg(color::LightWhite));
        }

        if *item {
            print!("{}  ", color::Bg(color::Black)); //█ ▜
        } else {
            print!("{}  ", color::Bg(color::LightWhite));
        }

        if (i + 1) % w == 0 {
            println!("{}      {}",
                     color::Bg(color::LightWhite),
                     color::Bg(color::Reset));
        }
    }

    //Draw the last white safe zone
    for a in 1..(wide * 3) + 1 {
        print!("{}  ", color::Bg(color::LightWhite));
        if a % wide == 0 {
            println!("{}", color::Bg(color::Reset));
        }
    }
}
