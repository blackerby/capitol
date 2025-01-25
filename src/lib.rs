#![allow(dead_code)]

mod legislation;

//use std::fmt::Display;
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

#[derive(Debug, PartialEq)]
struct Congress(Vec<u8>);

//impl Display for Congress<'_> {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        write!(f, "{}", self.0)
//    }
//}

//impl Congress {
//    fn as_ordinal(&self) -> String {
//        if self.0.ends_with('1') {
//            format!("{self}st")
//        } else if self.0.ends_with('2') {
//            format!("{self}nd")
//        } else if self.0.ends_with('3') {
//            format!("{self}rd")
//        } else {
//            format!("{self}th")
//        }
//    }
//
//    fn to_number(&self) -> usize {
//        self.0.parse::<usize>().unwrap()
//    }
//}

#[derive(Debug, PartialEq)]
enum Chamber {
    House,
    Senate,
}

// TODO: tokenize first
fn tokenize(input: &str) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut iter = input.as_bytes().iter().peekable();
    let mut congress_bytes: Vec<u8> = Vec::with_capacity(3);
    let mut chamber_byte: Vec<u8> = Vec::with_capacity(1);
    //let mut type_bytes: Vec<u8> = Vec::with_capacity(7);
    while let Some(&ch) = iter.next_if(|&&ch| ch > b'0' && ch <= b'9') {
        congress_bytes.push(ch);
    }
    // TODO: peek instead?
    while let Some(&ch) = iter.next_if(|&&ch| ch == b'h' || ch == b'H' || ch == b's' || ch == b'S')
    {
        chamber_byte.push(ch);
    }

    let remainder: Vec<u8> = iter.map(|b| *b).collect();
    (congress_bytes, chamber_byte, remainder)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_hand_rolled() {
        let mut input = "118hr8070";
        let expected = (b"118".to_vec(), b"h".to_vec(), b"r8070".to_vec());
        let result = tokenize(&mut input);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_year_and_congress() {
        assert_eq!(*CURRENT_YEAR, 2025);
        assert_eq!(*CURRENT_CONGRESS, 119);
    }

    //#[test]
    //fn test_congress_as_ordinal() {
    //    let congress = Congress("111");
    //    let ordinal = congress.as_ordinal();
    //    assert_eq!("111st", ordinal);
    //
    //    let congress = Congress("112");
    //    let ordinal = congress.as_ordinal();
    //    assert_eq!("112nd", ordinal);
    //
    //    let congress = Congress("113");
    //    let ordinal = congress.as_ordinal();
    //    assert_eq!("113rd", ordinal);
    //
    //    let congress = Congress("116");
    //    let ordinal = congress.as_ordinal();
    //    assert_eq!("116th", ordinal);
    //}
}
