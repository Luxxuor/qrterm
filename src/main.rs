#[macro_use]
extern crate lazy_static;

extern crate clap;
extern crate qrcode;
extern crate termion;
extern crate image;
extern crate regex;

use clap::{App, Arg, SubCommand};
use qrcode::{QrCode, EcLevel};
use termion::color;

mod payloads;

/*TODO: add arguments for:
- ec Level
- size of qr code
- implement different types of qr payloads
- change strings into constants
*/
fn main() {
    let app = App::new("qrterm")
        .version("0.1")
        .author("Lukas R. <lukas@bootsmann-games.de>")
        .about("Generates and displays terminal friendly QR-Codes from input strings")
        .arg(Arg::with_name("safe_zone") //TODO: change this into an off switch
            .global(true)
            .required(false)
            .short("s")
            .long("safe-zone")
            //.value_name("BOOL")
            .help("Sets wether the safe zone around the code should be drawn or not.")
            .takes_value(false)
            //.default_value("true")
            //.possible_values(&["true", "false"])
            //.value_names(&["true", "false"])
            )
            //TODO: subcmds are not working like expected, if you have one the system assumes you have many
        .subcommand(SubCommand::with_name("wifi")
            .about("formats to a wifi access string")
            .arg(Arg::with_name("ssid").required(true))
            .arg(Arg::with_name("pwd").required(true))
            .arg(Arg::with_name("mode")
                .required(true)
                .possible_values(&["WEP", "WPA", "nopass"])
                .value_name("MODE"))
            .arg(Arg::with_name("hidden")
                .required(false)
                .default_value("false")
                .possible_values(&["true", "false"])
                .value_name("HIDDEN")))
        .arg(Arg::with_name("INPUT")
            .help("The input string to use")
            .required_unless_one(&["wifi"]));

    let matches = app.get_matches();

    let payload: String;
    if let Some(sub) = matches.subcommand_matches("wifi") {
        let auth = match sub.value_of("mode") {
            Some("WEP") => payloads::Authentication::WEP,
            Some("WPA") => payloads::Authentication::WPA,
            _ => payloads::Authentication::nopass,
        };
        payload = payloads::wifi_string(sub.value_of("ssid").unwrap(),
                                        sub.value_of("pwd").unwrap(),
                                        &auth,
                                        sub.value_of("hidden").unwrap() == "true");
    } else {
        payload = String::from(matches.value_of("INPUT").unwrap());
    }

    let safe: bool = match matches.occurrences_of("safe_zone") {
        0 => true,
        _ => false,
    };

    let code = QrCode::with_error_correction_level(payload, EcLevel::H).unwrap();

    let bit_array = code.to_vec();

    let w = code.width();
    let wide = w + 6;

    //Draw the first white safe zone
    if safe {
        for a in 1..(wide * 3) + 1 {
            print!("{}  ", color::Bg(color::LightWhite));
            if a % wide == 0 {
                println!("{}", color::Bg(color::Reset));
            }
        }
    }

    for (i, item) in bit_array.iter().enumerate() {
        //left safe zone
        if safe && i % w == 0 {
            print!("{}      ", color::Bg(color::LightWhite));
        }

        if *item {
            print!("{}  ", color::Bg(color::Black));
        } else {
            print!("{}  ", color::Bg(color::LightWhite));
        }

        if (i + 1) % w == 0 {
            if safe {
                //draw right safe zone
                println!("{}      {}",
                         color::Bg(color::LightWhite),
                         color::Bg(color::Reset));
            } else {
                println!("{}", color::Bg(color::Reset));
            }
        }
    }

    //Draw the last white safe zone
    if safe {
        for a in 1..(wide * 3) + 1 {
            print!("{}  ", color::Bg(color::LightWhite));
            if a % wide == 0 {
                println!("{}", color::Bg(color::Reset));
            }
        }
    }
}
