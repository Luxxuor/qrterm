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

const WIFI_COMMAND: &'static str = "wifi";
const MAIL_COMMAND: &'static str = "mail";
// const SMS_COMMAND: &'static str = "sms";
// const MMS_COMMAND: &'static str = "mms";
// const GEO_COMMAND: &'static str = "geo";
const PHONE_COMMAND: &'static str = "phone";
// const SKYPE_COMMAND: &'static str = "skype";
const URL_COMMAND: &'static str = "url";
// const BOOKMARK_COMMAND: &'static str = "bookmark";
// const BITCOIN_COMMAND: &'static str = "bitcoin";
// const GIRO_COMMAND: &'static str = "giro";
// const CALENDAR_COMMAND: &'static str = "calendar";

/*TODO: add arguments for:
- ec Level
- size of qr code
- implement different types of qr payloads
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
            .help("Sets wether the safe zone around the code should be drawn or not.")
            .takes_value(false)
            )
            //TODO: subcmds are not working like expected, if you have one the system assumes you have many
        .subcommand(SubCommand::with_name(WIFI_COMMAND)
            .about("formats to a wifi access string")
            .arg(Arg::with_name("ssid").required(true))
            .arg(Arg::with_name("pwd").required(true))
            .arg(Arg::with_name("mode")
                .possible_values(&["WEP", "WPA", "nopass"])
                .default_value("WPA")
                .value_name("MODE"))
            .arg(Arg::with_name("hidden")
                .required(false)
                .possible_values(&["true", "false"])
                .default_value("false")
                .value_name("HIDDEN")))
        .subcommand(SubCommand::with_name(MAIL_COMMAND)
            .about("formats to a mail adress string")
            .arg(Arg::with_name("receiver").required(true))
            .arg(Arg::with_name("subject"))
            .arg(Arg::with_name("message"))
            .arg(Arg::with_name("encoding")
                .possible_values(&["MAILTO", "MATMSG", "SMTP"])
                .default_value("MAILTO")
                .value_name("ENCODING")))
        .subcommand(SubCommand::with_name(URL_COMMAND)
            .about("formats to an URL")
            .arg(Arg::with_name("url")
                .required(true)
                .value_name("URL")))
        .subcommand(SubCommand::with_name(PHONE_COMMAND)
            .about("formats to a phone number")
            .arg(Arg::with_name("number")
                .required(true)
                .value_name("NUMBER")))
        .arg(Arg::with_name("INPUT")
            .help("The input string to use")
            .required_unless_one(&[WIFI_COMMAND, MAIL_COMMAND, URL_COMMAND, PHONE_COMMAND]));

    let matches = app.get_matches();

    let payload: String;
    if let Some(sub) = matches.subcommand_matches(WIFI_COMMAND) {
        let auth = match sub.value_of("mode") {
            Some("WEP") => payloads::Authentication::WEP,
            Some("WPA") => payloads::Authentication::WPA,
            _ => payloads::Authentication::nopass,
        };
        payload = payloads::wifi_string(sub.value_of("ssid").unwrap(),
                                        sub.value_of("pwd").unwrap(),
                                        &auth,
                                        sub.value_of("hidden").unwrap() == "true");
    } else if let Some(sub) = matches.subcommand_matches(MAIL_COMMAND) {
        let encoding = match sub.value_of("encoding") {
            Some("MAILTO") => payloads::MailEncoding::MAILTO,
            Some("MATMSG") => payloads::MailEncoding::MATMSG,
            Some("SMTP") => payloads::MailEncoding::SMTP,
            _ => payloads::MailEncoding::MAILTO,
        };
        payload = payloads::mail_string(sub.value_of("receiver").unwrap(),
                                        sub.value_of("subject").unwrap(),
                                        sub.value_of("message").unwrap(),
                                        &encoding);
    } else if let Some(sub) = matches.subcommand_matches(URL_COMMAND) {
        payload = payloads::url_string(sub.value_of("url").unwrap());
    } else if let Some(sub) = matches.subcommand_matches(PHONE_COMMAND) {
        payload = payloads::phone_string(sub.value_of("phone").unwrap());
    } else {
        payload = String::from(matches.value_of("INPUT").unwrap());
    }

    let safe: bool = match matches.occurrences_of("safe_zone") {
        0 => true,
        _ => false,
    };

    let code = QrCode::with_error_correction_level(payload, EcLevel::H).unwrap();

    let bit_array = code.to_colors();

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

        if *item == qrcode::types::Color::Dark {
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
