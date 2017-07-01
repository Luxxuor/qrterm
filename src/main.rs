#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;

extern crate qrcode;
extern crate termion;
extern crate image;
extern crate regex;

use clap::{App, Arg, SubCommand, AppSettings};
use qrcode::{QrCode, EcLevel};
use termion::color;
use image::GrayImage;

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
- Add help texts for the subcommands
- Add error texts for some exit branches (mostly file output and qrcode gen)
- shell completions (see build.rasst file)
- implement different types of qr payloads
*/
fn main() {
    // create the cli interface with all subcommands, flags and args
    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .global_setting(AppSettings::SubcommandsNegateReqs)
        .arg(Arg::with_name("safe_zone")
            .global(true)
            .required(false)
            .short("s")
            .long("safe-zone")
            .help("Sets wether the safe zone around the code should be drawn or not.")
            .takes_value(false)
            .multiple(false))
        .arg(Arg::with_name("output")
            .global(true)
            .short("o").long("output")
            .help("Prints the QR-Code to a file. The image format is derived from the file extension. Currently only jpeg and png files are supported.")
            .takes_value(true)
            .value_name("FILE")
            .multiple(false))
        .arg(Arg::with_name("error")
            .global(true)
            .short("e").long("error")
            .help("Set the desired error correction level")
            .takes_value(true)
            .value_name("LEVEL")
            .possible_values(&["L", "M", "Q", "H"])
            .default_value("H"))
        .arg(Arg::with_name("INPUT")
            .help("The input string to use")
            .required(true))
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
                .value_name("NUMBER"))
        );

    let matches = app.get_matches();

    // deduce the string payload 
    let payload = get_payload(&matches);
    // what error level can we expect? always defaults to "H"
    let error: EcLevel = match matches.value_of("error").unwrap() {
        "L" => EcLevel::L,
        "M" => EcLevel::M,
        "Q" => EcLevel::Q,
        &_ => EcLevel::H,
    };

    //TODO: catch the possible errors
    let code = QrCode::with_error_correction_level(payload, error).unwrap();

    // are we drawing to the terminal or to a file?
    match matches.occurrences_of("output") {
        1 => save(&code, safe, matches.value_of("output").unwrap()),
        _ => draw(&code, safe),
    }
}

// deduces wich kind of string we are going to encode
fn get_payload(matches: &clap::ArgMatches) -> String {
    if let Some(sub) = matches.subcommand_matches(WIFI_COMMAND) {
        let auth = match sub.value_of("mode") {
            Some("WEP") => payloads::Authentication::WEP,
            Some("WPA") => payloads::Authentication::WPA,
            _ => payloads::Authentication::nopass,
        };
        return payloads::wifi_string(sub.value_of("ssid").unwrap(),
                                     sub.value_of("pwd").unwrap(),
                                     &auth,
                                     sub.value_of("hidden").unwrap() == "true");
    } else if let Some(sub) = matches.subcommand_matches(MAIL_COMMAND) {
        let encoding = match sub.value_of("encoding") {
            Some("MATMSG") => payloads::MailEncoding::MATMSG,
            Some("SMTP") => payloads::MailEncoding::SMTP,
            _ => payloads::MailEncoding::MAILTO,
        };
        return payloads::mail_string(sub.value_of("receiver").unwrap(),
                                     sub.value_of("subject").unwrap(),
                                     sub.value_of("message").unwrap(),
                                     &encoding);
    } else if let Some(sub) = matches.subcommand_matches(URL_COMMAND) {
        return payloads::url_string(sub.value_of("url").unwrap());
    } else if let Some(sub) = matches.subcommand_matches(PHONE_COMMAND) {
        return payloads::phone_string(sub.value_of("phone").unwrap());
    } else {
        return String::from(matches.value_of("INPUT").unwrap());
    }
}

    let safe: bool = match matches.occurrences_of("safe_zone") {
        0 => true,
        _ => false,
    };
// save to a file at the path
fn save(code: &QrCode, safe: bool, path: &str) {
    // render to a image struct
    let image: GrayImage = code.render().quiet_zone(safe).to_image();

    // save the image
    match image.save(path) {
        Ok(..) => println!("Image successfully saved to: {:?}", path),
        Err(e) => println!("Tried to create file but there was a problem: {:?}", 
            if let Some(inner_err) = e.get_ref() { inner_err.to_string() } else { e.to_string() }),
    };
}

// draw to the terminal
fn draw(code: &QrCode, safe: bool) {
    // get "bit" array
    let bit_array = code.to_vec();

    // get the code width and add extra space for the safe zone
    let w = code.width();
    let wide = w + 6;

    // draw the first white safe zone
    if safe {
        for a in 1..(wide * 3) + 1 {
            print!("{}  ", color::Bg(color::LightWhite));
            if a % wide == 0 {
                println!("{}", color::Bg(color::Reset));
            }
        }
    }

    // main drawing loop
    for (i, item) in bit_array.iter().enumerate() {
        // left safe zone
        if safe && i % w == 0 {
            print!("{}      ", color::Bg(color::LightWhite));
        }

        // draw black or white blocks
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
