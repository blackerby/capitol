// TODO: use this as a guide to generating string representation of legislation
// https://www.congress.gov/help/citation-guide

use crate::{
    parse_chamber, parse_congress, parse_positive_integer, AbbreviateType, Chamber, Congress,
    Reference, Url, BASE_URL,
};
use std::fmt::Display;

use anyhow;
use winnow::{
    ascii::alpha0,
    error::{ContextError, ErrMode},
    PResult, Parser,
};

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

impl Display for LegislationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Bill => String::from("bill"),
                Self::Resolution(r) => format!(
                    "{}resolution",
                    match r {
                        ResolutionType::Simple => "",
                        ResolutionType::Concurrent => "concurrent-",
                        ResolutionType::Joint => "joint-",
                    }
                ),
            }
        )
    }
}

#[derive(Debug, PartialEq)]
struct Legislation<'s> {
    congress: Congress<'s>,
    chamber: Chamber,
    leg_type: LegislationType,
    number: &'s str,
}

impl<'s> AbbreviateType for Legislation<'s> {
    fn abbreviate_type(&self) -> &str {
        match (&self.chamber, &self.leg_type) {
            (Chamber::House, LegislationType::Bill) => "H.R.",
            (Chamber::House, LegislationType::Resolution(r)) => match r {
                ResolutionType::Simple => "H.Res",
                ResolutionType::Concurrent => "H.Con.Res",
                ResolutionType::Joint => "H.J.Res",
            },
            (Chamber::Senate, LegislationType::Bill) => "S.",
            (Chamber::Senate, LegislationType::Resolution(r)) => match r {
                ResolutionType::Simple => "S.Res",
                ResolutionType::Concurrent => "S.Con.Res",
                ResolutionType::Joint => "S.J.Res",
            },
        }
    }
}

impl<'s> Legislation<'s> {
    fn parse(&mut input: &mut &'s str) -> anyhow::Result<Self> {
        parse_citation
            .parse(input)
            .map_err(|e| anyhow::format_err!("{e}"))
    }
}

impl<'s> Reference for Legislation<'s> {
    fn reference(&self) -> String {
        let (start, end) = self.congress.years();
        format!(
            "{}{} – {} Congress ({}-{})",
            self.abbreviate_type(),
            self.number,
            self.congress.as_ordinal(),
            start,
            end
        )
    }
}

impl<'s> Url for Legislation<'s> {
    fn to_url(&self) -> String {
        format!(
            "{BASE_URL}/bill/{}-congress/{}-{}/{}",
            self.congress.as_ordinal(),
            self.chamber,
            self.leg_type,
            self.number
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::CURRENT_CONGRESS;

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
    fn test_parse_senate_simple_res() {
        let mut input = "103sres15";
        let output = Legislation::parse(&mut input).unwrap();
        assert_eq!(
            output,
            Legislation {
                congress: Congress("103"),
                chamber: Chamber::Senate,
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

    #[test]
    fn test_congress_to_number() {
        let congress = Congress("119");
        assert_eq!(119, congress.to_number());
    }

    #[test]
    fn test_congress_years() {
        let congress = Congress("119");
        assert_eq!((2025, 2026), congress.years());

        let congress = Congress("118");
        assert_eq!((2023, 2024), congress.years());
    }

    #[test]
    fn test_future_congress_is_err() {
        let future_congress = *CURRENT_CONGRESS + 1;
        let bad_cite = format!("{future_congress}hr51");
        let result = Legislation::parse(&mut bad_cite.as_str());
        assert!(result.is_err())
    }

    #[test]
    fn test_to_url() {
        let legislation = Legislation::parse(&mut "118hr870").unwrap();
        let url = legislation.to_url();
        assert_eq!(
            url,
            "https://www.congress.gov/bill/118th-congress/house-bill/870"
        );

        let legislation = Legislation::parse(&mut "118hres230").unwrap();
        let url = legislation.to_url();
        assert_eq!(
            url,
            "https://www.congress.gov/bill/118th-congress/house-resolution/230"
        );

        let legislation = Legislation::parse(&mut "118sc230").unwrap();
        let url = legislation.to_url();
        assert_eq!(
            url,
            "https://www.congress.gov/bill/118th-congress/senate-concurrent-resolution/230"
        );

        let legislation = Legislation::parse(&mut "113sj230").unwrap();
        let url = legislation.to_url();
        assert_eq!(
            url,
            "https://www.congress.gov/bill/113rd-congress/senate-joint-resolution/230"
        );
    }

    #[test]
    fn test_leg_reference() {
        let legislation = Legislation::parse(&mut "118hr870").unwrap();
        let reference = legislation.reference();

        assert_eq!("H.R.870 – 118th Congress (2023-2024)", reference.as_str());
    }
}
