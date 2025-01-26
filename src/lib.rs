// TODO: add tests for each legislation type
// TODO: add CLI
// TODO: test sad path
// TODO: impl FromStr for Citation?
mod constants;
mod error;

use crate::constants::{BILL_VERSIONS, CURRENT_CONGRESS};
use crate::error::Error;

#[allow(dead_code)]
type Result<T> = std::result::Result<T, Error>;

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
    fn parse(input: &[u8]) -> Self {
        match String::from_utf8(input.to_vec()) {
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
        if input.eq_ignore_ascii_case(&b'h') {
            Self::House
        } else {
            Self::Senate
        }
    }
}

#[derive(Debug, PartialEq)]
enum CongObjectType {
    SenateBill,
    HouseBill,
    SenateResolution,
    HouseResolution,
    SenateConcurrentResolution,
    HouseConcurrentResolution,
    SenateJointResolution,
    HouseJointResolution,
    // TODO: add committee report types
}

impl CongObjectType {
    fn parse(input: &[u8], chamber: &Chamber) -> Self {
        match input.to_ascii_lowercase().as_slice() {
            b"" | b"r" if *chamber == Chamber::House => Self::HouseBill,
            b"" if *chamber == Chamber::Senate => Self::SenateBill,
            b"res" if *chamber == Chamber::House => Self::HouseResolution,
            b"res" if *chamber == Chamber::Senate => Self::SenateResolution,
            b"conres" if *chamber == Chamber::House => Self::HouseConcurrentResolution,
            b"conres" if *chamber == Chamber::Senate => Self::SenateConcurrentResolution,
            b"jres" if *chamber == Chamber::House => Self::HouseJointResolution,
            b"jres" if *chamber == Chamber::Senate => Self::SenateJointResolution,
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

        parts.congress.clone_from(&congress_bytes);

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
    #[must_use]
    pub fn parse(input: &str) -> Self {
        let bytes = Self::tokenize(input);
        let congress = Congress::parse(&bytes.congress);
        let chamber = Chamber::parse(bytes.chamber);
        let object_type = CongObjectType::parse(&bytes.object_type, &chamber);
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
            object_type: CongObjectType::HouseBill,
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
