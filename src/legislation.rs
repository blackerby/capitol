// TODO: use this as a guide to generating string representation of legislation
// https://www.congress.gov/help/citation-guide

// TODO: understand and improve Winnow errors

use crate::{
    parse_chamber, parse_congress, parse_positive_integer, AbbreviateType, Chamber, Congress,
    Reference, Url, BASE_URL,
};
use std::fmt::Display;

use winnow::{
    ascii::alpha0,
    combinator::alt,
    error::{ContextError, ErrMode},
    PResult, Parser,
};

const BILL_VERSIONS: [&str; 38] = [
    "as", "ash", "ath", "ats", "cdh", "cds", "cph", "cps", "eah", "eas", "eh", "enr", "es", "fph",
    "fps", "hds", "ih", "iph", "ips", "is", "lth", "lts", "pap", "pcs", "pp", "rch", "rcs", "rds",
    "rfh", "rfs", "rh", "rhuc", "rih", "rs", "rth", "rts", "sc", "",
];

fn parse_legislation_type<'s>(input: &mut &'s str) -> PResult<LegislationType<'s>> {
    let leg_type = alpha0.parse_next(input).map_err(ErrMode::cut)?;
    match leg_type {
        "e" | "res" => Ok(LegislationType::Resolution(ResolutionType::Simple)),
        "c" | "cres" | "conres" => Ok(LegislationType::Resolution(ResolutionType::Concurrent)),
        "j" | "jres" => Ok(LegislationType::Resolution(ResolutionType::Joint)),
        "" | "r" => Ok(LegislationType::Bill(leg_type)),
        _ => Err(ErrMode::Cut(ContextError::new())),
    }
}

fn parse_bill_version<'s>(input: &mut &'s str) -> PResult<Option<BillVersion<'s>>> {
    let bill_version = alt(BILL_VERSIONS).parse_next(input).map_err(ErrMode::cut)?;

    if bill_version.is_empty() {
        Ok(None)
    } else {
        Ok(Some(BillVersion(bill_version)))
    }
}

fn parse_citation<'s>(input: &mut &'s str) -> PResult<Legislation<'s>> {
    let (congress, chamber, leg_type, number, bill_version) = (
        parse_congress,
        parse_chamber,
        parse_legislation_type,
        parse_positive_integer,
        parse_bill_version,
    )
        .parse_next(input)
        .map_err(ErrMode::cut)?;

    if let LegislationType::Bill("r") = leg_type {
        if chamber == Chamber::Senate {
            return Err(ErrMode::Cut(ContextError::new()));
        }
    }

    Ok(Legislation {
        congress,
        chamber,
        leg_type,
        number,
        bill_version,
    })
}

#[derive(Debug, PartialEq)]
enum ResolutionType {
    Simple,
    Concurrent,
    Joint,
}

#[derive(Debug, PartialEq)]
enum LegislationType<'s> {
    Bill(&'s str),
    Resolution(ResolutionType),
}

impl Display for LegislationType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Bill(_) => String::from("bill"),
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
    leg_type: LegislationType<'s>,
    number: &'s str,
    bill_version: Option<BillVersion<'s>>,
}

impl AbbreviateType for Legislation<'_> {
    fn abbreviate_type(&self) -> &str {
        match (&self.chamber, &self.leg_type) {
            (Chamber::House, LegislationType::Bill(_)) => "H.R.",
            (Chamber::House, LegislationType::Resolution(r)) => match r {
                ResolutionType::Simple => "H.Res",
                ResolutionType::Concurrent => "H.Con.Res",
                ResolutionType::Joint => "H.J.Res",
            },
            (Chamber::Senate, LegislationType::Bill(_)) => "S.",
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

impl Reference for Legislation<'_> {
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

impl Url for Legislation<'_> {
    fn to_url(&self, with_ver: bool) -> String {
        let mut base = format!(
            "{BASE_URL}/bill/{}-congress/{}-{}/{}",
            self.congress.as_ordinal(),
            self.chamber,
            self.leg_type,
            self.number
        );

        if with_ver {
            base.push_str("/text/");
            base.push_str(self.bill_version.as_ref().unwrap().0);
        }

        base
    }
}

#[derive(Debug, PartialEq)]
struct BillVersion<'s>(&'s str);

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
                leg_type: LegislationType::Bill("r"),
                number: "8070",
                bill_version: None
            }
        );
    }

    #[test]
    fn test_parse_senate_bill() {
        let mut input = "118s15";
        let output = Legislation::parse(&mut input).unwrap();
        assert_eq!(
            output,
            Legislation {
                congress: Congress("118"),
                chamber: Chamber::Senate,
                leg_type: LegislationType::Bill(""),
                number: "15",
                bill_version: None
            }
        );
    }

    #[test]
    fn test_parse_house_bill_with_version() {
        let mut input = "118hr8070ih";
        let output = Legislation::parse(&mut input).unwrap();
        assert_eq!(
            output,
            Legislation {
                congress: Congress("118"),
                chamber: Chamber::House,
                leg_type: LegislationType::Bill("r"),
                number: "8070",
                bill_version: Some(BillVersion("ih"))
            }
        );
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
                leg_type: LegislationType::Bill(""),
                number: "8070",
                bill_version: None
            }
        );
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
                number: "15",
                bill_version: None
            }
        );
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
                number: "15",
                bill_version: None,
            }
        );
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
                number: "15",
                bill_version: None
            }
        );
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
    fn test_sn_is_error() {
        let mut input = "1sn1";
        let output = Legislation::parse(&mut input);

        assert!(output.is_err());
    }

    #[test]
    fn test_future_congress_is_err() {
        let future_congress = *CURRENT_CONGRESS + 1;
        let bad_cite = format!("{future_congress}hr51");
        let result = Legislation::parse(&mut bad_cite.as_str());
        assert!(result.is_err());
    }

    #[test]
    fn test_to_url() {
        let legislation = Legislation::parse(&mut "118hr870").unwrap();
        let url = legislation.to_url(false);
        assert_eq!(
            url,
            "https://www.congress.gov/bill/118th-congress/house-bill/870"
        );

        let legislation = Legislation::parse(&mut "118hr870ih").unwrap();
        let url = legislation.to_url(true);
        assert_eq!(
            url,
            "https://www.congress.gov/bill/118th-congress/house-bill/870/text/ih"
        );

        let legislation = Legislation::parse(&mut "118hr870ih").unwrap();
        let url = legislation.to_url(false);
        assert_eq!(
            url,
            "https://www.congress.gov/bill/118th-congress/house-bill/870"
        );

        let legislation = Legislation::parse(&mut "118hres230").unwrap();
        let url = legislation.to_url(false);
        assert_eq!(
            url,
            "https://www.congress.gov/bill/118th-congress/house-resolution/230"
        );

        let legislation = Legislation::parse(&mut "118sc230").unwrap();
        let url = legislation.to_url(false);
        assert_eq!(
            url,
            "https://www.congress.gov/bill/118th-congress/senate-concurrent-resolution/230"
        );

        let legislation = Legislation::parse(&mut "113sj230").unwrap();
        let url = legislation.to_url(false);
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
