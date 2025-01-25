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

#[derive(Debug, PartialEq)]
enum Chamber {
    House,
    Senate,
}

fn tokenize(input: &str) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut iter = input.as_bytes().iter().peekable();

    // initialize containers for various parts of the citation
    let mut congress_bytes: Vec<u8> = Vec::with_capacity(3);
    let mut chamber_byte: Vec<u8> = Vec::with_capacity(1);
    let mut type_bytes: Vec<u8> = Vec::with_capacity(7);
    let mut number_bytes: Vec<u8> = Vec::new();
    let mut ver_bytes: Vec<u8> = Vec::new();

    while let Some(&ch) = iter.next_if(|&&ch| ch > b'0' && ch <= b'9') {
        congress_bytes.push(ch);
    }

    while let Some(&ch) = iter.next_if(|&&ch| ch == b'h' || ch == b'H' || ch == b's' || ch == b'S')
    {
        chamber_byte.push(ch);
    }

    while let Some(&ch) = iter.next_if(|&&ch| ch.is_ascii_alphabetic()) {
        type_bytes.push(ch);
    }

    while let Some(&ch) = iter.next_if(|&&ch| ch.is_ascii_digit()) {
        number_bytes.push(ch);
    }

    while let Some(&ch) = iter.next_if(|&&ch| ch.is_ascii_alphabetic()) {
        ver_bytes.push(ch);
    }

    (
        congress_bytes,
        chamber_byte,
        type_bytes,
        number_bytes,
        ver_bytes,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenize_no_ver() {
        let mut input = "118hr8070";
        let expected = (
            b"118".to_vec(),
            b"h".to_vec(),
            b"r".to_vec(),
            b"8070".to_vec(),
            Vec::new(),
        );
        let result = tokenize(&mut input);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_tokenize_with_ver() {
        let mut input = "118hr8070ih";
        let expected = (
            b"118".to_vec(),
            b"h".to_vec(),
            b"r".to_vec(),
            b"8070".to_vec(),
            b"ih".to_vec(),
        );
        let result = tokenize(&mut input);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_year_and_congress() {
        assert_eq!(*CURRENT_YEAR, 2025);
        assert_eq!(*CURRENT_CONGRESS, 119);
    }
}
