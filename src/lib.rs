#![allow(dead_code)]

mod legislation;

use chrono::Datelike;
//use std::fmt::Display;
use std::sync::LazyLock;

pub(crate) const FIRST_CONGRESS: usize = 1789;
// only dealing with common era years
// Right to use UTC?
#[allow(clippy::cast_sign_loss)]
static CURRENT_YEAR: LazyLock<usize> = LazyLock::new(|| chrono::Utc::now().year() as usize);
pub(crate) static CURRENT_CONGRESS: LazyLock<usize> =
    LazyLock::new(|| *CURRENT_YEAR - FIRST_CONGRESS / 2 + 1);
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

fn parse(input: &str) -> Congress {
    let iter = input.as_bytes().iter();
    let congress_string: Vec<u8> = iter
        .take_while(|&&ch| ch > b'0' && ch <= b'9')
        .cloned()
        .collect();
    Congress(congress_string)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_hand_rolled() {
        let mut input = "118hr8070";
        let expected = Congress(b"118".to_vec());
        let result = parse(&mut input);
        assert_eq!(expected, result);
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
