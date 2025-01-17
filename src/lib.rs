#![allow(dead_code)]

mod legislation;

use chrono::Datelike;
use std::fmt::Display;
use std::sync::LazyLock;

use winnow::{
    ascii::{digit1, Caseless},
    combinator::alt,
    error::{ContextError, ErrMode},
    PResult, Parser,
};

pub(crate) const FIRST_CONGRESS: usize = 1789;
// only dealing with common era years
// Right to use UTC?
#[allow(clippy::cast_sign_loss)]
static CURRENT_YEAR: LazyLock<usize> = LazyLock::new(|| chrono::Utc::now().year() as usize);
pub(crate) static CURRENT_CONGRESS: LazyLock<usize> =
    LazyLock::new(|| *CURRENT_YEAR - FIRST_CONGRESS / 2 + 1);
pub(crate) const BASE_URL: &str = "https://www.congress.gov";

#[derive(Debug, PartialEq)]
struct Congress<'s>(&'s str);

impl Display for Congress<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'s> Congress<'s> {
    fn as_ordinal(&self) -> String {
        if self.0.ends_with('1') {
            format!("{self}st")
        } else if self.0.ends_with('2') {
            format!("{self}nd")
        } else if self.0.ends_with('3') {
            format!("{self}rd")
        } else {
            format!("{self}th")
        }
    }

    fn to_number(&self) -> usize {
        self.0.parse::<usize>().unwrap()
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

trait AbbreviateType {
    fn abbreviate_type(&self) -> &str;
}

trait Url {
    fn to_url(&self, with_ver: bool) -> String;
}

fn parse_positive_integer<'s>(input: &mut &'s str) -> PResult<&'s str> {
    let int = digit1.parse_next(input).map_err(ErrMode::cut)?;

    if int == "0" {
        Err(ErrMode::Cut(ContextError::new()))
    } else {
        Ok(int)
    }
}

fn parse_congress<'s>(input: &mut &'s str) -> PResult<Congress<'s>> {
    let maybe_congress = parse_positive_integer.parse_next(input)?;
    let int = maybe_congress.parse::<usize>().unwrap();
    if int <= *CURRENT_CONGRESS {
        Ok(Congress(maybe_congress))
    } else {
        Err(ErrMode::Cut(ContextError::new()))
    }
}

fn senate(input: &mut &str) -> PResult<Chamber> {
    Caseless("s").parse_next(input).map(|_| Chamber::Senate)
}

fn house(input: &mut &str) -> PResult<Chamber> {
    Caseless("h").parse_next(input).map(|_| Chamber::House)
}

fn parse_chamber(input: &mut &str) -> PResult<Chamber> {
    alt((senate, house)).parse_next(input).map_err(ErrMode::cut)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_congress_as_ordinal() {
        let congress = Congress("111");
        let ordinal = congress.as_ordinal();
        assert_eq!("111st", ordinal);

        let congress = Congress("112");
        let ordinal = congress.as_ordinal();
        assert_eq!("112nd", ordinal);

        let congress = Congress("113");
        let ordinal = congress.as_ordinal();
        assert_eq!("113rd", ordinal);

        let congress = Congress("116");
        let ordinal = congress.as_ordinal();
        assert_eq!("116th", ordinal);
    }
}
