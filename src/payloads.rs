use regex::Regex;
use urlparse::quote;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Authentication {
    WEP,
    WPA,
    nopass,
}

pub fn wifi_string(ssid: &str, password: &str, mode: &Authentication, is_hidden: bool) -> String {
    let ssid_n = escape_input(ssid, false);
    let sn = "\"".to_string() + &ssid_n + "\"";
    let ssid_n = if is_hexstyle(&ssid_n) { sn } else { ssid_n };
    let password_n = escape_input(password, false);
    let pn = "\"".to_string() + &password_n + "\"";
    let password_n = if is_hexstyle(&password_n) {
        pn
    } else {
        password_n
    };
    let hidden = if is_hidden { "H:true" } else { "" };

    return format!("WIFI:T:{:?};S:{};P:{};{};",
                   mode,
                   ssid_n,
                   password_n,
                   hidden);
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum MailEncoding {
    MAILTO,
    MATMSG,
    SMTP,
}

pub fn mail_string(receiver: &str,
                   subject: &str,
                   message: &str,
                   encoding: &MailEncoding)
                   -> String {
    match *encoding {
        MailEncoding::MAILTO => {
            format!("mailto:{}?subject={}&body={}", receiver, uri_escape(subject), uri_escape(message))
        }
        MailEncoding::MATMSG => {
            format!("MATMSG:TO:{};SUB:{};BODY:{};;",
                        receiver,
                        escape_input(subject, false),
                        escape_input(message, false))
        }
        MailEncoding::SMTP => {
            format!("SMTP:{}:{}:{}",
                        receiver,
                        escape_input(subject, true),
                        escape_input(message, true))
        }
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum SMSEncoding
{
    SMS,
    SMSTO,
    SMS_iOS
}

pub fn sms_string(number: &str, subject: &str, encoding: &SMSEncoding) -> String {
    match *encoding {
        SMSEncoding::SMS => format!("sms:{}?body={}", number, uri_escape(subject)),
        SMSEncoding::SMS_iOS => format!("sms:{};body={}", number, uri_escape(subject)),
        SMSEncoding::SMSTO => format!("SMSTO:{}:{}", number, subject),
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum MMSEncoding
{
    MMS,
    MMSTO
}

pub fn mms_string(number: &str, subject: &str, encoding: &MMSEncoding) -> String {
    match *encoding {
        MMSEncoding::MMSTO => format!("mmsto:{}?subject={}", number, uri_escape(subject)),
        MMSEncoding::MMS => format!("mms:{}?body={}", number, uri_escape(subject))
    }
}

pub fn phone_string(number: &str) -> String {
    String::from(format!("tel:{}", number))
}

pub fn url_string(url: &str) -> String {
    if !url.starts_with("http") {
        "http://".to_string() + url
    } else {
        String::from(url)
    }
}

fn escape_input(inp: &str, simple: bool) -> String {
    let mut forbidden = Vec::new();
    if simple {
        forbidden.push(":");
    } else {
        forbidden.push("\\");
        forbidden.push(";");
        forbidden.push(",");
        forbidden.push(":");
    };

    let mut n = String::from(inp);
    for c in &forbidden {
        n = str::replace(&n, c, &("\\".to_string() + c));
    }
    n
}

fn uri_escape(inp: &str) -> String {
    quote(inp, b"").ok().unwrap()
}

#[allow(unknown_lints)]
#[allow(zero_ptr)]
fn is_hexstyle(arg: &str) -> bool {
    lazy_static! {
        static ref LE: Regex = Regex::new(r"\A\b[0-9a-fA-F]+\b\z").unwrap();
        static ref RE: Regex = Regex::new(r"\A\b(0[xX])?[0-9a-fA-F]+\b\z").unwrap();
    }

    LE.is_match(arg) || RE.is_match(arg)
}
