// TODO: use this as a guide to generating string representation of legislation
// https://www.congress.gov/help/citation-guide

use std::fmt::Display;

use anyhow;
use winnow::ascii::alpha0;
use winnow::ascii::digit1;
use winnow::ascii::Caseless;
use winnow::combinator::{alt, opt, terminated};
use winnow::error::ContextError;
use winnow::error::ErrMode;
use winnow::{PResult, Parser};

fn parse_positive_integer<'s>(input: &mut &'s str) -> PResult<&'s str> {
    let int = digit1.parse_next(input).map_err(ErrMode::cut)?;

    if int == "0" {
        Err(ErrMode::Cut(ContextError::new()))
    } else {
        Ok(int)
    }
}
fn parse_congress<'s>(input: &mut &'s str) -> PResult<Congress<'s>> {
    parse_positive_integer.parse_next(input).map(Congress)
}

fn senate(input: &mut &str) -> PResult<Chamber> {
    Caseless("s").parse_next(input).map(|_| Chamber::Senate)
}

fn house(input: &mut &str) -> PResult<Chamber> {
    terminated(Caseless("h"), opt(Caseless("r")))
        .parse_next(input)
        .map(|_| Chamber::House)
}

fn parse_chamber(input: &mut &str) -> PResult<Chamber> {
    alt((senate, house)).parse_next(input).map_err(ErrMode::cut)
}

fn parse_legislation_type(input: &mut &str) -> PResult<LegislationType> {
    let leg_type = alpha0.parse_next(input).map_err(ErrMode::cut)?;
    match leg_type {
        "e" | "es" => Ok(LegislationType::Resolution(ResolutionType::Simple)),
        "c" | "cres" | "conres" => Ok(LegislationType::Resolution(ResolutionType::Concurrent)),
        "j" | "jres" => Ok(LegislationType::Resolution(ResolutionType::Joint)),
        "" => Ok(LegislationType::Bill),
        _ => Err(ErrMode::Cut(ContextError::new())),
    }
}

fn parse_citation<'s>(input: &mut &'s str) -> PResult<Legislation<'s>> {
    let (congress, chamber, leg_type, number) = (
        parse_congress,
        parse_chamber,
        parse_legislation_type,
        parse_positive_integer,
    )
        .parse_next(input)
        .map_err(ErrMode::cut)?;

    Ok(Legislation {
        congress,
        chamber,
        leg_type,
        number,
    })
}

#[derive(Debug, PartialEq)]
struct Congress<'s>(&'s str);

impl<'s> Display for Congress<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'s> Congress<'s> {
    fn as_ordinal(&self) -> String {
        if self.0.ends_with("1") {
            format!("{self}st")
        } else if self.0.ends_with("2") {
            format!("{self}nd")
        } else if self.0.ends_with("3") {
            format!("{self}rd")
        } else {
            format!("{self}th")
        }
    }
}

#[derive(Debug, PartialEq)]
enum Chamber {
    House,
    Senate,
}

#[derive(Debug, PartialEq)]
enum ResolutionType {
    Simple,
    Concurrent,
    Joint,
}

#[derive(Debug, PartialEq)]
enum LegislationType {
    Bill,
    Resolution(ResolutionType),
}

#[derive(Debug, PartialEq)]
struct Legislation<'s> {
    congress: Congress<'s>,
    chamber: Chamber,
    leg_type: LegislationType,
    number: &'s str,
}

impl<'s> Legislation<'s> {
    fn parse(&mut input: &mut &'s str) -> anyhow::Result<Self> {
        parse_citation
            .parse(input)
            .map_err(|e| anyhow::format_err!("{e}"))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_house_bill() {
        let mut input = "118hr8070";
        let output = Legislation::parse(&mut input).unwrap();
        assert_eq!(
            output,
            Legislation {
                congress: Congress("118"),
                chamber: Chamber::House,
                leg_type: LegislationType::Bill,
                number: "8070"
            }
        )
    }

    #[test]
    fn test_parse_house_bill_short() {
        let mut input = "118h8070";
        let output = Legislation::parse(&mut input).unwrap();
        assert_eq!(
            output,
            Legislation {
                congress: Congress("118"),
                chamber: Chamber::House,
                leg_type: LegislationType::Bill,
                number: "8070"
            }
        )
    }

    #[test]
    fn test_parse_house_simple_res() {
        let mut input = "103hres15";
        let output = Legislation::parse(&mut input).unwrap();
        assert_eq!(
            output,
            Legislation {
                congress: Congress("103"),
                chamber: Chamber::House,
                leg_type: LegislationType::Resolution(ResolutionType::Simple),
                number: "15"
            }
        )
    }

    #[test]
    fn test_parse_house_simple_res_short() {
        let mut input = "103he15";
        let output = Legislation::parse(&mut input).unwrap();
        assert_eq!(
            output,
            Legislation {
                congress: Congress("103"),
                chamber: Chamber::House,
                leg_type: LegislationType::Resolution(ResolutionType::Simple),
                number: "15"
            }
        )
    }

    #[test]
    fn test_zero_congress_is_error() {
        let mut input = "0hr1";
        let output = Legislation::parse(&mut input);

        assert!(output.is_err());
    }

    #[test]
    fn test_zero_bill_num_is_error() {
        let mut input = "1hr0";
        let output = Legislation::parse(&mut input);

        assert!(output.is_err());
    }

    #[test]
    fn test_sr_is_error() {
        let mut input = "1sr1";
        let output = Legislation::parse(&mut input);

        assert!(output.is_err());
    }

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
