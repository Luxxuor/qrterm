#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;

extern crate qrterm;

use clap::{App, AppSettings, Arg, Shell, SubCommand};
use qrcode::EcLevel;
use std::fs;
use std::process::exit;

mod payloads;

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

        build_cli().gen_completions("qr", shell, dir);

        println!(
            "Completion file for the {:?} shell was writen to: {:?}",
            shell, dir
        );

        // exit and dont generate a qr-code
        exit(0);
    }

    let mut params = qrterm::Parameters::new();

    // write the completions if they were requested, then exit and dont print any qr-code
    if let Some(comp) = matches.subcommand_matches("completions") {
        params.completions.comp_dir = match comp.value_of("comp_dir") {
            Some(e) => e.to_string(),
            None => "".to_string(),
        };
        params.completions.shell = match comp.value_of("shell") {
            Some(e) => e.to_string(),
            None => "".to_string(),
        };
    }

    // deduce the string payload
    params.payload = get_payload(&matches);

    // should we draw a white border (safe zone) around the code?
    params.safe_zone = match matches.occurrences_of("safe_zone") {
        0 => true,
        _ => false,
    };

    // what error level can we expect? defaults to "H"
    params.error = match matches.value_of("error").unwrap() {
        "L" => EcLevel::L,
        "M" => EcLevel::M,
        "Q" => EcLevel::Q,
        &_ => EcLevel::H,
    };

    // What outputs are there

    params.output = match matches.value_of("output") {
        Some(e) => e.to_string(),
        None => "".to_string(),
    };

    params.generate();
}

// deduces wich kind of string we are going to encode
fn get_payload(matches: &clap::ArgMatches<'_>) -> String {
    if let Some(sub) = matches.subcommand_matches(qrterm::WIFI_COMMAND) {
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
    } else if let Some(sub) = matches.subcommand_matches(qrterm::MAIL_COMMAND) {
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
    } else if let Some(sub) = matches.subcommand_matches(qrterm::URL_COMMAND) {
        return payloads::url_string(sub.value_of("url").unwrap());
    } else if let Some(sub) = matches.subcommand_matches(qrterm::PHONE_COMMAND) {
        return payloads::phone_string(sub.value_of("phone").unwrap());
    } else if let Some(sub) = matches.subcommand_matches(qrterm::SKYPE_COMMAND) {
        return payloads::skype_string(sub.value_of("name").unwrap());
    } else if let Some(sub) = matches.subcommand_matches(qrterm::WHATSAPP_COMMAND) {
        return payloads::whatsapp_string(sub.value_of("message").unwrap());
    } else if let Some(sub) = matches.subcommand_matches(qrterm::SMS_COMMAND) {
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
    } else if let Some(sub) = matches.subcommand_matches(qrterm::MMS_COMMAND) {
        let encoding = match sub.value_of("encoding") {
            Some("MMSTO") => payloads::MMSEncoding::MMSTO,
            _ => payloads::MMSEncoding::MMS,
        };
        return payloads::mms_string(
            sub.value_of("number").unwrap(),
            sub.value_of("subject").unwrap(),
            &encoding,
        );
    } else if let Some(sub) = matches.subcommand_matches(qrterm::GEO_COMMAND) {
        let encoding = match sub.value_of("encoding") {
            Some("GoogleMaps") => payloads::GeolocationEncoding::GoogleMaps,
            _ => payloads::GeolocationEncoding::GEO,
        };
        return payloads::geo_string(
            sub.value_of("latitude").unwrap(),
            sub.value_of("longitude").unwrap(),
            &encoding,
        );
    } else if let Some(sub) = matches.subcommand_matches(qrterm::BOOKMARK_COMMAND) {
        return payloads::bookmark_string(
            sub.value_of("title").unwrap(),
            sub.value_of("url").unwrap(),
        );
    } else if let Some(sub) = matches.subcommand_matches(qrterm::BITCOIN_COMMAND) {
        return payloads::bitcoin_string(
            sub.value_of("address").unwrap(),
            sub.value_of("amount")
                .map(|a| a.parse::<f64>().unwrap_or_default()),
            sub.value_of("label"),
            sub.value_of("message"),
        );
    } else {
        return String::from(matches.value_of("INPUT").unwrap());
    }
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
            Arg::with_name("payload")
                .global(true)
                .short("p")
                .long("payload")
                .help(
                    "If this flag is set, the generated payload will also be printed to the terminal."
                )
                .takes_value(false)
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
            SubCommand::with_name(qrterm::WIFI_COMMAND)
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
            SubCommand::with_name(qrterm::MAIL_COMMAND)
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
            SubCommand::with_name(qrterm::URL_COMMAND)
                .about("formats to an URL QR-Code")
                .arg(Arg::with_name("url").required(true).value_name("URL")),
        )
        .subcommand(
            SubCommand::with_name(qrterm::PHONE_COMMAND)
                .about("formats to a phone number QR-Code")
                .arg(Arg::with_name("number").required(true).value_name("NUMBER")),
        )
        .subcommand(
            SubCommand::with_name(qrterm::SKYPE_COMMAND)
                .about("formats to a skype call QR-Code")
                .arg(Arg::with_name("name").required(true).value_name("HANDLE")),
        )
        .subcommand(
            SubCommand::with_name(qrterm::WHATSAPP_COMMAND)
                .about("formats to a whatsapp message QR-Code")
                .arg(Arg::with_name("message").required(true).value_name(
                    "MESSAGE",
                )),
        )
        .subcommand(
            SubCommand::with_name(qrterm::SMS_COMMAND)
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
            SubCommand::with_name(qrterm::MMS_COMMAND)
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
            SubCommand::with_name(qrterm::GEO_COMMAND)
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
            SubCommand::with_name(qrterm::BOOKMARK_COMMAND)
                .about("formats to a bookmark QR-Code")
                .arg(Arg::with_name("title").required(true))
                .arg(Arg::with_name("url").required(true)),
        )
        .subcommand(
            SubCommand::with_name(qrterm::BITCOIN_COMMAND)
                .about("outputs a bitcoin adress/transaction")
                .arg(Arg::with_name("address").required(true))
                .arg(Arg::with_name("amount"))
                .arg(Arg::with_name("label"))
                .arg(Arg::with_name("message")),
        )
}
