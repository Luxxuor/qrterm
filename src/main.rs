#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;

extern crate qrcode;
extern crate image;
extern crate regex;
extern crate urlparse;
extern crate term;

use clap::{App, AppSettings, Arg, Shell, SubCommand};
use qrcode::{QrCode, EcLevel};
use term::color;
use image::Luma;
use std::io::prelude::*;
use std::process::exit;
use std::fs;

mod payloads;

const WIFI_COMMAND: &'static str = "wifi";
const MAIL_COMMAND: &'static str = "mail";
const SMS_COMMAND: &'static str = "sms";
const MMS_COMMAND: &'static str = "mms";
const GEO_COMMAND: &'static str = "geo";
const PHONE_COMMAND: &'static str = "phone";
const SKYPE_COMMAND: &'static str = "skype";
const WHATSAPP_COMMAND: &'static str = "whatsapp";
const URL_COMMAND: &'static str = "url";
const BOOKMARK_COMMAND: &'static str = "bookmark";
// const BITCOIN_COMMAND: &'static str = "bitcoin";
// const GIRO_COMMAND: &'static str = "giro";
// const CALENDAR_COMMAND: &'static str = "calendar";

/*TODO: add arguments for:
- Add help texts for the subcommands
- Add error texts for some exit branches (mostly file output and qrcode gen)
- implement different types of qr payloads
- write unit tests for the payloads
- write integration tests for edge case inputs
*/
fn main() {
    // match all input args
    let matches = build_cli().get_matches();

    // write the completions if they were requested, then exit and dont print any qr-code
    if let Some(comp) = matches.subcommand_matches("completions") {
        let dir = comp.value_of("comp_dir").unwrap();

        let shell = match comp.value_of("shell") {
            Some("ps") => Shell::PowerShell,
            Some("zsh") => Shell::Zsh,
            Some("fish") => Shell::Fish,
            _ => Shell::Bash,
        };

        // create directory if necessary
        fs::create_dir_all(&dir).unwrap();

        let mut app = build_cli();
        app.gen_completions("qr", shell, dir);

        println!(
            "Completion file for the {:?} shell was writen to: {:?}",
            shell,
            dir
        );

        // exit and dont generate a qr-code
        exit(0);
    }

    // deduce the string payload
    let payload = get_payload(&matches);

    // should we draw a white border (safe zone) around the code?
    let safe: bool = match matches.occurrences_of("safe_zone") {
        0 => true,
        _ => false,
    };

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
        return payloads::wifi_string(
            sub.value_of("ssid").unwrap(),
            sub.value_of("pwd").unwrap(),
            &auth,
            sub.value_of("hidden").unwrap() == "true",
        );
    } else if let Some(sub) = matches.subcommand_matches(MAIL_COMMAND) {
        let encoding = match sub.value_of("encoding") {
            Some("MATMSG") => payloads::MailEncoding::MATMSG,
            Some("SMTP") => payloads::MailEncoding::SMTP,
            _ => payloads::MailEncoding::MAILTO,
        };
        return payloads::mail_string(
            sub.value_of("receiver").unwrap(),
            sub.value_of("subject").unwrap(),
            sub.value_of("message").unwrap(),
            &encoding,
        );
    } else if let Some(sub) = matches.subcommand_matches(URL_COMMAND) {
        return payloads::url_string(sub.value_of("url").unwrap());
    } else if let Some(sub) = matches.subcommand_matches(PHONE_COMMAND) {
        return payloads::phone_string(sub.value_of("phone").unwrap());
    } else if let Some(sub) = matches.subcommand_matches(SKYPE_COMMAND) {
        return payloads::skype_string(sub.value_of("name").unwrap());
    } else if let Some(sub) = matches.subcommand_matches(WHATSAPP_COMMAND) {
        return payloads::whatsapp_string(sub.value_of("message").unwrap());
    } else if let Some(sub) = matches.subcommand_matches(SMS_COMMAND) {
        let encoding = match sub.value_of("encoding") {
            Some("SMSTO") => payloads::SMSEncoding::SMSTO,
            Some("SMS_iOS") => payloads::SMSEncoding::SMS_iOS,
            _ => payloads::SMSEncoding::SMS,
        };
        return payloads::sms_string(
            sub.value_of("number").unwrap(),
            sub.value_of("subject").unwrap(),
            &encoding,
        );
    } else if let Some(sub) = matches.subcommand_matches(MMS_COMMAND) {
        let encoding = match sub.value_of("encoding") {
            Some("MMSTO") => payloads::MMSEncoding::MMSTO,
            _ => payloads::MMSEncoding::MMS,
        };
        return payloads::mms_string(
            sub.value_of("number").unwrap(),
            sub.value_of("subject").unwrap(),
            &encoding,
        );
    } else if let Some(sub) = matches.subcommand_matches(GEO_COMMAND) {
        let encoding = match sub.value_of("encoding") {
            Some("GoogleMaps") => payloads::GeolocationEncoding::GoogleMaps,
            _ => payloads::GeolocationEncoding::GEO,
        };
        return payloads::geo_string(
            sub.value_of("latitude").unwrap(),
            sub.value_of("longitude").unwrap(),
            &encoding,
        );
    } else if let Some(sub) = matches.subcommand_matches(BOOKMARK_COMMAND) {
        return payloads::bookmark_string(
            sub.value_of("title").unwrap(),
            sub.value_of("url").unwrap(),
        );
    } else {
        return String::from(matches.value_of("INPUT").unwrap());
    }
}

// save to a file at the path
fn save(code: &QrCode, safe: bool, path: &str) {
    // render to a image struct
    let image = code.render::<Luma<u8>>().quiet_zone(safe).build();

    // save the image
    match image.save(path) {
        Ok(..) => println!("Image successfully saved to: {:?}", path),
        Err(e) => {
            println!(
                "Tried to create file but there was a problem: {:?}",
                if let Some(inner_err) = e.get_ref() {
                    inner_err.to_string()
                } else {
                    e.to_string()
                }
            )
        }
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

            t.bg(term::color::BRIGHT_WHITE).unwrap();
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
            t.bg(term::color::BRIGHT_WHITE).unwrap();
            write!(t, "      ").unwrap();
        }

        // draw black or white blocks
        if *item == qrcode::Color::Dark {
            t.bg(color::BLACK).unwrap();
            write!(t, "  ").unwrap();
        } else {
            t.bg(term::color::BRIGHT_WHITE).unwrap();
            write!(t, "  ").unwrap();
        }

        if (i + 1) % w == 0 {
            if safe {
                // draw right safe zone
                t.bg(term::color::BRIGHT_WHITE).unwrap();
                write!(t, "      ").unwrap();
            }
            t.reset().unwrap();
            writeln!(t, "").unwrap();
        }
    }

    // draw the last white safe zone
    if safe {
        for a in 1..(wide * 3) + 1 {
            t.bg(term::color::BRIGHT_WHITE).unwrap();
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

// create the interface for the app with all subcommands, flags and args
fn build_cli() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .global_setting(AppSettings::SubcommandsNegateReqs)
        .arg(
            Arg::with_name("safe_zone")
                .global(true)
                .short("s")
                .long("safe-zone")
                .help(
                    "Sets wether the safe zone around the code should be drawn or not.",
                )
                .takes_value(false),
        )
        .arg(
            Arg::with_name("output")
                .global(true)
                .short("o")
                .long("output")
                .help(
                    "Prints the QR-Code to a file.
            The image format is derived from the file extension.
            Currently only jpeg and png files are supported.",
                )
                .value_name("FILE"),
        )
        .arg(
            Arg::with_name("error")
                .global(true)
                .short("e")
                .long("error")
                .help("Set the desired error correction level.")
                .value_name("LEVEL")
                .possible_values(&["L", "M", "Q", "H"])
                .default_value("H"),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("The input string to use")
                .required(true),
        )
        .subcommand(
            SubCommand::with_name("completions")
                .about("Outputs completion files for various shells.")
                .arg(
                    Arg::with_name("comp_dir")
                        .required(true)
                        .help("The directory to write the completion file to.")
                        .value_name("DIR"),
                )
                .arg(
                    Arg::with_name("shell")
                        .long("shell")
                        .help("For which shell the completions should be generated for.")
                        .value_name("SHELL")
                        .possible_values(&["bash", "zsh", "fish", "ps"])
                        .default_value("bash"),
                ),
        )
        .subcommand(
            SubCommand::with_name(WIFI_COMMAND)
                .about("formats to a wifi access string QR-Code")
                .arg(Arg::with_name("ssid").required(true))
                .arg(Arg::with_name("pwd").required(true))
                .arg(
                    Arg::with_name("mode")
                        .value_name("MODE")
                        .possible_values(&["WEP", "WPA", "nopass"])
                        .default_value("WPA"),
                )
                .arg(
                    Arg::with_name("hidden")
                        .value_name("HIDDEN")
                        .possible_values(&["true", "false"])
                        .default_value("false"),
                ),
        )
        .subcommand(
            SubCommand::with_name(MAIL_COMMAND)
                .about("formats to a mail adress string QR-Code")
                .arg(Arg::with_name("receiver").required(true))
                .arg(Arg::with_name("subject"))
                .arg(Arg::with_name("message"))
                .arg(
                    Arg::with_name("encoding")
                        .value_name("ENCODING")
                        .possible_values(&["MAILTO", "MATMSG", "SMTP"])
                        .default_value("MAILTO"),
                ),
        )
        .subcommand(
            SubCommand::with_name(URL_COMMAND)
                .about("formats to an URL QR-Code")
                .arg(Arg::with_name("url").required(true).value_name("URL")),
        )
        .subcommand(
            SubCommand::with_name(PHONE_COMMAND)
                .about("formats to a phone number QR-Code")
                .arg(Arg::with_name("number").required(true).value_name("NUMBER")),
        )
        .subcommand(
            SubCommand::with_name(SKYPE_COMMAND)
                .about("formats to a skype call QR-Code")
                .arg(Arg::with_name("name").required(true).value_name("HANDLE")),
        )
        .subcommand(
            SubCommand::with_name(WHATSAPP_COMMAND)
                .about("formats to a whatsapp message QR-Code")
                .arg(Arg::with_name("message").required(true).value_name(
                    "MESSAGE",
                )),
        )
        .subcommand(
            SubCommand::with_name(SMS_COMMAND)
                .about("formats to a sms message QR-Code")
                .arg(Arg::with_name("number").required(true))
                .arg(Arg::with_name("subject").default_value(""))
                .arg(
                    Arg::with_name("encoding")
                        .possible_values(&["SMS", "SMSTO", "SMS_iOS"])
                        .default_value("SMS"),
                ),
        )
        .subcommand(
            SubCommand::with_name(MMS_COMMAND)
                .about("formats to a mms message QR-Code")
                .arg(Arg::with_name("number").required(true))
                .arg(Arg::with_name("subject").default_value(""))
                .arg(
                    Arg::with_name("encoding")
                        .possible_values(&["MMS", "MMSTO"])
                        .default_value("MMS"),
                ),
        )
        .subcommand(
            SubCommand::with_name(GEO_COMMAND)
                .about("formats to a geospacial location QR-Code")
                .arg(Arg::with_name("latitude").required(true))
                .arg(Arg::with_name("longitude").required(true))
                .arg(
                    Arg::with_name("encoding")
                        .possible_values(&["GEO", "GoogleMaps"])
                        .default_value("GEO"),
                ),
        )
        .subcommand(
            SubCommand::with_name(BOOKMARK_COMMAND)
                .about("formats to a bookmark QR-Code")
                .arg(Arg::with_name("title").required(true))
                .arg(Arg::with_name("url").required(true)),
        )
}
