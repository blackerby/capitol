// TODO: use this as a guide to generating string representation of legislation
// https://www.congress.gov/help/citation-guide

// TODO: understand and improve Winnow errors

use crate::{
    parse_chamber, parse_congress, parse_positive_integer, AbbreviateType, Chamber, Congress,
    Reference, Url, BASE_URL,
};
use std::fmt::Display;

use anyhow;
use winnow::{
    ascii::alpha0,
    combinator::alt,
    error::{ContextError, ErrMode},
    PResult, Parser,
};

const BILL_VERSION: [&str; 38] = [
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
    let bill_version = alt(BILL_VERSION).parse_next(input).map_err(ErrMode::cut)?;

    match bill_version {
        "as" => Ok(Some(BillVersion::AmendmentOrderedPrinted(bill_version))),
        "ash" => Ok(Some(BillVersion::AdditionalSponsorsHouse(bill_version))),
        "ath" => Ok(Some(BillVersion::AgreedToHouse(bill_version))),
        "ats" => Ok(Some(BillVersion::AgreedToSenate(bill_version))),
        "cdh" => Ok(Some(BillVersion::CommitteeDischargedHouse(bill_version))),
        "cds" => Ok(Some(BillVersion::CommitteeDischargedSenate(bill_version))),
        "cph" => Ok(Some(BillVersion::ConsideredPassedHouse(bill_version))),
        "cps" => Ok(Some(BillVersion::ConsideredPassedSenate(bill_version))),
        "eah" => Ok(Some(BillVersion::EngrossedAmendmentHouse(bill_version))),
        "eas" => Ok(Some(BillVersion::EngrossedAmendmentSenate(bill_version))),
        "eh" => Ok(Some(BillVersion::EngrossedHouse(bill_version))),
        "enr" => Ok(Some(BillVersion::Enrolled(bill_version))),
        "es" => Ok(Some(BillVersion::EngrossedSenate(bill_version))),
        "fph" => Ok(Some(BillVersion::FailedPassageHouse(bill_version))),
        "fps" => Ok(Some(BillVersion::FailedPassageSenate(bill_version))),
        "hds" => Ok(Some(BillVersion::HeldDeskSenate(bill_version))),
        "ih" => Ok(Some(BillVersion::IntroducedHouse(bill_version))),
        "iph" => Ok(Some(BillVersion::IndefinitelyPostponedHouse(bill_version))),
        "ips" => Ok(Some(BillVersion::IndefinitelyPostponedSenate(bill_version))),
        "is" => Ok(Some(BillVersion::IntroducedSenate(bill_version))),
        "lth" => Ok(Some(BillVersion::LaidTableHouse(bill_version))),
        "lts" => Ok(Some(BillVersion::LaidTableSenate(bill_version))),
        "pap" => Ok(Some(BillVersion::PrintedAsPassed(bill_version))),
        "pcs" => Ok(Some(BillVersion::PlacedCalendarSenate(bill_version))),
        "pp" => Ok(Some(BillVersion::PublicPrint(bill_version))),
        "rch" => Ok(Some(BillVersion::ReferenceChangeHouse(bill_version))),
        "rcs" => Ok(Some(BillVersion::ReferenceChangeSenate(bill_version))),
        "rds" => Ok(Some(BillVersion::ReceivedSenate(bill_version))),
        "rfh" => Ok(Some(BillVersion::ReferredHouse(bill_version))),
        "rfs" => Ok(Some(BillVersion::ReferredSenate(bill_version))),
        "rh" => Ok(Some(BillVersion::ReportedHouse(bill_version))),
        "rhuc" => Ok(Some(BillVersion::ReturnedHouseUnanimousConsent(
            bill_version,
        ))),
        "rih" => Ok(Some(BillVersion::ReferralInstructionsHouse(bill_version))),
        "rs" => Ok(Some(BillVersion::ReportedSenate(bill_version))),
        "rth" => Ok(Some(BillVersion::ReferredCommitteeHouse(bill_version))),
        "rts" => Ok(Some(BillVersion::ReferredCommitteeSenate(bill_version))),
        "sc" => Ok(Some(BillVersion::SponsorChange(bill_version))),
        "" => Ok(None),
        _ => unreachable!(),
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

impl<'s> Display for LegislationType<'s> {
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

impl<'s> AbbreviateType for Legislation<'s> {
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
            base.push_str(self.bill_version.as_ref().unwrap().tag());
        }

        base
    }
}

#[derive(Debug, PartialEq)]
enum BillVersion<'s> {
    AmendmentOrderedPrinted(&'s str),
    AdditionalSponsorsHouse(&'s str),
    AgreedToHouse(&'s str),
    AgreedToSenate(&'s str),
    CommitteeDischargedHouse(&'s str),
    CommitteeDischargedSenate(&'s str),
    ConsideredPassedHouse(&'s str),
    ConsideredPassedSenate(&'s str),
    EngrossedAmendmentHouse(&'s str),
    EngrossedAmendmentSenate(&'s str),
    EngrossedHouse(&'s str),
    Enrolled(&'s str),
    EngrossedSenate(&'s str),
    FailedPassageHouse(&'s str),
    FailedPassageSenate(&'s str),
    HeldDeskSenate(&'s str),
    IntroducedHouse(&'s str),
    IndefinitelyPostponedHouse(&'s str),
    IndefinitelyPostponedSenate(&'s str),
    IntroducedSenate(&'s str),
    LaidTableHouse(&'s str),
    LaidTableSenate(&'s str),
    PrintedAsPassed(&'s str),
    PlacedCalendarSenate(&'s str),
    PublicPrint(&'s str),
    ReferenceChangeHouse(&'s str),
    ReferenceChangeSenate(&'s str),
    ReceivedSenate(&'s str),
    ReferredHouse(&'s str),
    ReferredSenate(&'s str),
    ReportedHouse(&'s str),
    ReturnedHouseUnanimousConsent(&'s str),
    ReferralInstructionsHouse(&'s str),
    ReportedSenate(&'s str),
    ReferredCommitteeHouse(&'s str),
    ReferredCommitteeSenate(&'s str),
    SponsorChange(&'s str),
}

impl<'s> BillVersion<'s> {
    fn tag(&self) -> &'s str {
        match self {
            Self::AmendmentOrderedPrinted(s)
            | Self::AdditionalSponsorsHouse(s)
            | Self::AgreedToHouse(s)
            | Self::AgreedToSenate(s)
            | Self::CommitteeDischargedHouse(s)
            | Self::CommitteeDischargedSenate(s)
            | Self::ConsideredPassedHouse(s)
            | Self::ConsideredPassedSenate(s)
            | Self::EngrossedAmendmentHouse(s)
            | Self::EngrossedAmendmentSenate(s)
            | Self::EngrossedHouse(s)
            | Self::Enrolled(s)
            | Self::EngrossedSenate(s)
            | Self::FailedPassageHouse(s)
            | Self::FailedPassageSenate(s)
            | Self::HeldDeskSenate(s)
            | Self::IntroducedHouse(s)
            | Self::IndefinitelyPostponedHouse(s)
            | Self::IndefinitelyPostponedSenate(s)
            | Self::IntroducedSenate(s)
            | Self::LaidTableHouse(s)
            | Self::LaidTableSenate(s)
            | Self::PrintedAsPassed(s)
            | Self::PlacedCalendarSenate(s)
            | Self::PublicPrint(s)
            | Self::ReferenceChangeHouse(s)
            | Self::ReferenceChangeSenate(s)
            | Self::ReceivedSenate(s)
            | Self::ReferredHouse(s)
            | Self::ReferredSenate(s)
            | Self::ReportedHouse(s)
            | Self::ReturnedHouseUnanimousConsent(s)
            | Self::ReferralInstructionsHouse(s)
            | Self::ReportedSenate(s)
            | Self::ReferredCommitteeHouse(s)
            | Self::ReferredCommitteeSenate(s)
            | Self::SponsorChange(s) => s,
        }
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
                leg_type: LegislationType::Bill("r"),
                number: "8070",
                bill_version: None
            }
        )
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
                bill_version: Some(BillVersion::IntroducedHouse("ih"))
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
                leg_type: LegislationType::Bill(""),
                number: "8070",
                bill_version: None
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
                number: "15",
                bill_version: None
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
                number: "15",
                bill_version: None,
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
                number: "15",
                bill_version: None
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
    fn test_future_congress_is_err() {
        let future_congress = *CURRENT_CONGRESS + 1;
        let bad_cite = format!("{future_congress}hr51");
        let result = Legislation::parse(&mut bad_cite.as_str());
        assert!(result.is_err())
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
