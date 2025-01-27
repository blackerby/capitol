#![warn(missing_docs)]
//! Parse citations of United States Congress legislative documents and convert them to
//! Congress.gov URLs.
//!
//! Calling `parse` on a string like `"118hr815"`, or calling `Citation::parse("118hr815")`
//! returns a struct representing the document's citation. Calling `to_url` on such a struct
//! returns `"https://www.congress.gov/bill/118th-congress/house-bill/815"`
//!
//! Legislative citations generally follow the form `<CONGRESS><CONGRESSIONAL_OBJECT_TYPE><NUMBER>`.
//! Measures (bills) can specify a version of the text of the bill with a two-to-three letter
//! string at the end of the citation.

mod constants;
mod error;

use std::fmt::Display;
use std::str::FromStr;

use crate::constants::{BASE_URL, BILL_VERSIONS, CURRENT_CONGRESS};
use crate::error::Error;

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
    fn parse(input: &[u8]) -> Result<Self> {
        match String::from_utf8(input.to_vec()) {
            Ok(s) => {
                let congress = s.parse::<u64>()?;
                if congress <= *CURRENT_CONGRESS {
                    Ok(Congress(congress))
                } else {
                    Err(Error::InvalidCongress)
                }
            }
            Err(e) => Err(Error::FromUtf8(e)),
        }
    }

    fn as_ordinal(&self) -> String {
        let mut ordinal = self.to_string();
        if ordinal.ends_with('1') {
            ordinal.push_str("st");
        } else if ordinal.ends_with('2') {
            ordinal.push_str("nd");
        } else if ordinal.ends_with('3') {
            ordinal.push_str("rd");
        } else {
            ordinal.push_str("th");
        }
        ordinal
    }
}

impl Display for Congress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq)]
enum Chamber {
    House,
    Senate,
}

impl Display for Chamber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::House => "house",
                Self::Senate => "senate",
            }
        )
    }
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
    HouseReport,
    SenateReport,
}

impl CongObjectType {
    fn parse(input: &[u8], chamber: &Chamber) -> Result<Self> {
        match input.to_ascii_lowercase().as_slice() {
            b"" | b"r" if *chamber == Chamber::House => Ok(Self::HouseBill),
            b"" if *chamber == Chamber::Senate => Ok(Self::SenateBill),
            b"res" if *chamber == Chamber::House => Ok(Self::HouseResolution),
            b"res" if *chamber == Chamber::Senate => Ok(Self::SenateResolution),
            b"conres" if *chamber == Chamber::House => Ok(Self::HouseConcurrentResolution),
            b"conres" if *chamber == Chamber::Senate => Ok(Self::SenateConcurrentResolution),
            b"jres" if *chamber == Chamber::House => Ok(Self::HouseJointResolution),
            b"jres" if *chamber == Chamber::Senate => Ok(Self::SenateJointResolution),
            b"rpt" if *chamber == Chamber::House => Ok(Self::HouseReport),
            b"rpt" if *chamber == Chamber::Senate => Ok(Self::SenateReport),
            _ => Err(Error::UnknownCongObjectType),
        }
    }
}

impl Display for CongObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::HouseBill | Self::SenateBill => "bill",
                Self::HouseResolution | Self::SenateResolution => "resolution",
                Self::HouseConcurrentResolution | Self::SenateConcurrentResolution =>
                    "concurrent-resolution",
                Self::HouseJointResolution | Self::SenateJointResolution => "joint-resolution",
                Self::HouseReport | Self::SenateReport => "report",
            }
        )
    }
}

/// Represents a legislative Citation.
///
/// A `Citation` consists of a Congress, a Chamber, a Congressional object type, a number, and
/// optionally for bills, a Version.
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

    /// Parse a legislative citation.
    ///
    /// The method first breaks up the citation into its constituent parts, then parses each of the
    /// parts, validating that the given Congress does not exceed the current Congress.
    ///
    /// Example
    ///
    /// ```rust
    /// use capitol::Citation;
    ///
    /// let citation = Citation::parse("118hr815");
    /// ```
    ///
    /// # Errors
    ///
    /// Will result in an error if the Congress part of the citation is invalid (greater than the
    /// current Congress), if the Congressional object type is unrecognized, if an integer can't be
    /// parsed from the document number, or if the document is a bill and has an unrecognized
    /// version type.
    pub fn parse(input: &str) -> Result<Self> {
        let bytes = Self::tokenize(input);
        let congress = Congress::parse(&bytes.congress)?;
        let chamber = Chamber::parse(bytes.chamber);
        let object_type = CongObjectType::parse(&bytes.object_type, &chamber)?;
        let number = String::from_utf8(bytes.number)?.parse::<usize>()?;
        let ver = if let Some(v) = bytes.ver {
            if BILL_VERSIONS.contains(&v.as_slice()) {
                let text = String::from_utf8(v)?;
                Some(Version(text))
            } else {
                return Err(Error::InvalidBillVersion);
            }
        } else {
            None
        };

        Ok(Citation {
            congress,
            chamber,
            object_type,
            number,
            ver,
        })
    }

    /// Converts a `Citation` to a URL on Congress.gov.
    ///
    /// Example
    ///
    /// ```rust
    /// use capitol::Citation;
    ///
    /// let url = "118hr815".parse::<Citation>().unwrap().to_url();
    /// ```
    pub fn to_url(&self) -> String {
        let collection = match self.object_type {
            CongObjectType::HouseReport | CongObjectType::SenateReport => "congressional-report",
            _ => "bill",
        };
        let mut base = format!(
            "{BASE_URL}/{collection}/{}-congress/{}-{}/{}",
            self.congress.as_ordinal(),
            self.chamber,
            self.object_type,
            self.number
        );

        if let Some(ver) = &self.ver {
            base.push_str("/text/");
            base.push_str(&ver.0);
        }

        base
    }
}

impl FromStr for Citation {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(s)
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
        let result = input.parse();
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_parse_house_bill() {
        let input = "118hrpt529";
        let expected = Citation {
            congress: Congress(118),
            chamber: Chamber::House,
            object_type: CongObjectType::HouseReport,
            number: 529,
            ver: None,
        };
        let result = input.parse();
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_parse_senate_bill() {
        let input = "118srpt17";
        let expected = Citation {
            congress: Congress(118),
            chamber: Chamber::Senate,
            object_type: CongObjectType::SenateReport,
            number: 17,
            ver: None,
        };
        let result = input.parse();
        assert_eq!(expected, result.unwrap());
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

    #[test]
    fn test_house_bill_to_url() {
        let input = "118hr529";
        let expected = "https://www.congress.gov/bill/118th-congress/house-bill/529";
        let citation = input.parse::<Citation>().unwrap();
        let result = citation.to_url();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_house_bill_with_ver_to_url() {
        let input = "118hr529ih";
        let expected = "https://www.congress.gov/bill/118th-congress/house-bill/529/text/ih";
        let citation = input.parse::<Citation>().unwrap();
        let result = citation.to_url();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_house_report_to_url() {
        let input = "118hrpt529";
        let expected =
            "https://www.congress.gov/congressional-report/118th-congress/house-report/529";
        let citation = input.parse::<Citation>().unwrap();
        let result = citation.to_url();
        assert_eq!(expected, result);
    }
}
