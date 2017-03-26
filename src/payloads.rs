use regex::Regex;

#[derive(Debug)]
#[allow(dead_code)]
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

#[allow(unknown_lints)]
#[allow(zero_ptr)]
fn is_hexstyle(arg: &str) -> bool {
    lazy_static! {
        static ref LE: Regex = Regex::new(r"\A\b[0-9a-fA-F]+\b\z").unwrap();
        static ref RE: Regex = Regex::new(r"\A\b(0[xX])?[0-9a-fA-F]+\b\z").unwrap();
    }

    LE.is_match(arg) || RE.is_match(arg)
}
