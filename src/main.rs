extern crate clap;
extern crate qrcode;
extern crate termion;

use clap::{App, Arg, SubCommand};
use qrcode::QrCode;
use termion::color;

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

    //println!("Converting input string: {}", &input);

    let code = QrCode::new(input).unwrap();

    let bit_array = code.to_vec();

    let size = bit_array.len();
    let w = code.width();

    for i in 0..size {
        let item = bit_array[i];

        if item {
            print!("{}{}██", color::Fg(color::Black), color::Bg(color::Black)); //▜
        } else {
            print!("{}{}██", color::Fg(color::White), color::Bg(color::White));
        }

        if (i + 1) % w == 0 {
            println!("{}{}", color::Fg(color::Reset), color::Bg(color::Reset));
        }
    }
    //println!("");

    //println!("Hello, world!");
}
