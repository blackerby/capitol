// TODO: add tests for each legislation type
// TODO: add CLI
// TODO: test sad path

use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

const FIRST_CONGRESS: u64 = 1789;
static CURRENT_YEAR: LazyLock<u64> = LazyLock::new(|| {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap() // TODO: change to expect?
        .as_secs()
        / 31536000 // seconds in year
        + 1970 // UNIX_EPOCH year
});
static CURRENT_CONGRESS: LazyLock<u64> = LazyLock::new(|| (*CURRENT_YEAR - FIRST_CONGRESS) / 2 + 1);
//const BASE_URL: &str = "https://www.congress.gov";

const BILL_VERSIONS: [&[u8]; 37] = [
    b"as", b"ash", b"ath", b"ats", b"cdh", b"cds", b"cph", b"cps", b"eah", b"eas", b"eh", b"enr",
    b"es", b"fph", b"fps", b"hds", b"ih", b"iph", b"ips", b"is", b"lth", b"lts", b"pap", b"pcs",
    b"pp", b"rch", b"rcs", b"rds", b"rfh", b"rfs", b"rh", b"rhuc", b"rih", b"rs", b"rth", b"rts",
    b"sc",
];

#[derive(Debug, PartialEq)]
struct Version(String);

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
                if congress <= *CURRENT_CONGRESS {
                    Congress(congress)
                } else {
                    todo!()
                }
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

impl CongObjectType {
    fn parse(input: Vec<u8>, chamber: &Chamber) -> Self {
        match input.to_ascii_lowercase().as_slice() {
            b"" | b"r" if *chamber == Chamber::House => Self::Hr,
            b"" if *chamber == Chamber::Senate => Self::S,
            b"res" if *chamber == Chamber::House => Self::HRes,
            b"res" if *chamber == Chamber::Senate => Self::SRes,
            b"conres" if *chamber == Chamber::House => Self::HConRes,
            b"conres" if *chamber == Chamber::Senate => Self::SConRes,
            b"jres" if *chamber == Chamber::House => Self::HJRes,
            b"jres" if *chamber == Chamber::Senate => Self::SJRes,
            _ => todo!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Citation {
    congress: Congress,
    chamber: Chamber,
    object_type: CongObjectType,
    number: usize,
    ver: Option<Version>,
}

impl Citation {
    fn tokenize(input: &str) -> CiteBytes {
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

    // TODO: convert this to a Result type with a custom error
    pub fn parse(input: &str) -> Self {
        let bytes = Self::tokenize(input);
        let congress = Congress::parse(bytes.congress);
        let chamber = Chamber::parse(bytes.chamber);
        let object_type = CongObjectType::parse(bytes.object_type, &chamber);
        let number = String::from_utf8(bytes.number)
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let ver = if let Some(v) = bytes.ver {
            if BILL_VERSIONS.contains(&v.as_slice()) {
                let text = String::from_utf8(v).unwrap();
                Some(Version(text))
            } else {
                todo!()
            }
        } else {
            None
        };

        Citation {
            congress,
            chamber,
            object_type,
            number,
            ver,
        }
    }
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
    fn test_parse_no_ver_house_bill() {
        let input = "118hr8070";
        let expected = Citation {
            congress: Congress(118),
            chamber: Chamber::House,
            object_type: CongObjectType::Hr,
            number: 8070,
            ver: None,
        };
        let result = Citation::parse(input);
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
