#![allow(dead_code)]
// TODO: add tests for each legislation type

use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const FIRST_CONGRESS: u64 = 1789;
static CURRENT_YEAR: LazyLock<u64> = LazyLock::new(|| {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap() // TODO: change to expect?
        .as_secs()
        / 31536000 // seconds in year
        + 1970 // UNIX_EPOCH year
});
pub(crate) static CURRENT_CONGRESS: LazyLock<u64> =
    LazyLock::new(|| (*CURRENT_YEAR - FIRST_CONGRESS) / 2 + 1);
pub(crate) const BASE_URL: &str = "https://www.congress.gov";

const BILL_VERSIONS: [&str; 37] = [
    "as", "ash", "ath", "ats", "cdh", "cds", "cph", "cps", "eah", "eas", "eh", "enr", "es", "fph",
    "fps", "hds", "ih", "iph", "ips", "is", "lth", "lts", "pap", "pcs", "pp", "rch", "rcs", "rds",
    "rfh", "rfs", "rh", "rhuc", "rih", "rs", "rth", "rts", "sc",
];

#[derive(Debug, PartialEq)]
struct Version<'s>(&'s str);

#[derive(Debug, Default, PartialEq)]
struct CiteBytes {
    congress: Vec<u8>,
    chamber: u8,
    object_type: Vec<u8>,
    number: Vec<u8>,
    ver: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq)]
struct Congress(u64);

impl Congress {
    fn parse(input: Vec<u8>) -> Self {
        match String::from_utf8(input) {
            Ok(s) => {
                let congress = s
                    .parse::<u64>()
                    .expect("could not convert input to integer");
                Congress(congress)
            }
            _ => {
                todo!()
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum Chamber {
    House,
    Senate,
}

impl Chamber {
    fn parse(input: u8) -> Self {
        if input.to_ascii_lowercase() == b'h' {
            Self::House
        } else {
            Self::Senate
        }
    }
}

#[derive(Debug, PartialEq)]
enum CongObjectType {
    S,
    Hr,
    SRes,
    HRes,
    SConRes,
    HConRes,
    SJRes,
    HJRes,
    // TODO: add committee report types
}

//impl CongObjectType {
//    fn parse(input: Vec<u8>, chamber: Chamber) -> Self {
//        match input {
//        }
//    }
//}

#[derive(Debug, PartialEq)]
struct Citation<'s> {
    congress: Congress,
    chamber: Chamber,
    object_type: CongObjectType,
    number: usize,
    ver: Option<Version<'s>>,
}

impl Citation<'_> {
    pub fn tokenize(input: &str) -> CiteBytes {
        let mut iter = input.as_bytes().iter().peekable();

        // initialize containers for various parts of the citation
        let mut congress_bytes: Vec<u8> = Vec::with_capacity(3);
        let mut type_bytes: Vec<u8> = Vec::with_capacity(7);
        let mut number_bytes: Vec<u8> = Vec::new();
        let mut ver_bytes: Vec<u8> = Vec::new();

        // initialize parts container
        let mut parts = CiteBytes::default();

        while let Some(&ch) = iter.next_if(|&&ch| ch > b'0' && ch <= b'9') {
            congress_bytes.push(ch);
        }

        parts.congress = congress_bytes.clone();

        if let Some(&ch) = iter.next_if(|&&ch| ch == b'h' || ch == b'H' || ch == b's' || ch == b'S')
        {
            parts.chamber = ch;
        }

        while let Some(&ch) = iter.next_if(|&&ch| ch.is_ascii_alphabetic()) {
            type_bytes.push(ch);
        }

        parts.object_type = type_bytes;

        while let Some(&ch) = iter.next_if(|&&ch| ch.is_ascii_digit()) {
            number_bytes.push(ch);
        }

        parts.number = number_bytes;

        while let Some(&ch) = iter.next_if(|&&ch| ch.is_ascii_alphabetic()) {
            ver_bytes.push(ch);
        }

        if ver_bytes.is_empty() {
            parts.ver = None;
        } else {
            parts.ver = Some(ver_bytes);
        }

        parts
    }

    //pub fn parse(input: CiteBytes) -> Self {
    //    let congress = Congress::parse(input.congress);
    //    let chamber = Chamber::parse(input.chamber);
    //}
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenize_no_ver_house_bill() {
        let mut input = "118hr8070";
        let expected = CiteBytes {
            congress: b"118".to_vec(),
            chamber: b'h',
            object_type: b"r".to_vec(),
            number: b"8070".to_vec(),
            ver: None,
        };
        let result = Citation::tokenize(&mut input);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_tokenize_no_ver_senate_bill() {
        let mut input = "118s5";
        let expected = CiteBytes {
            congress: b"118".to_vec(),
            chamber: b's',
            object_type: Vec::new(),
            number: b"5".to_vec(),
            ver: None,
        };
        let result = Citation::tokenize(&mut input);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_tokenize_with_ver_house_bill() {
        let mut input = "118hr555ih";
        let expected = CiteBytes {
            congress: b"118".to_vec(),
            chamber: b'h',
            object_type: b"r".to_vec(),
            number: b"555".to_vec(),
            ver: Some(b"ih".to_vec()),
        };
        let result = Citation::tokenize(&mut input);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_tokenize_with_ver_senate_bill() {
        let mut input = "118s17is";
        let expected = CiteBytes {
            congress: b"118".to_vec(),
            chamber: b's',
            object_type: Vec::new(),
            number: b"17".to_vec(),
            ver: Some(b"is".to_vec()),
        };
        let result = Citation::tokenize(&mut input);
        assert_eq!(expected, result);
    }
}
