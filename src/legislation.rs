// TODO: understand and improve Winnow errors

#[allow(unused_imports)]
use crate::{Chamber, Congress, BASE_URL};
use std::fmt::Display;

const BILL_VERSIONS: [&str; 38] = [
    "as", "ash", "ath", "ats", "cdh", "cds", "cph", "cps", "eah", "eas", "eh", "enr", "es", "fph",
    "fps", "hds", "ih", "iph", "ips", "is", "lth", "lts", "pap", "pcs", "pp", "rch", "rcs", "rds",
    "rfh", "rfs", "rh", "rhuc", "rih", "rs", "rth", "rts", "sc", "",
];

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
    congress: Congress,
    chamber: Chamber,
    leg_type: LegislationType<'s>,
    number: &'s str,
    bill_version: Option<BillVersion<'s>>,
}

#[derive(Debug, PartialEq)]
struct BillVersion<'s>(&'s str);

//#[cfg(test)]
//mod test {
//    use super::*;
//    use crate::CURRENT_CONGRESS;
//}
