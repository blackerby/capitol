// TODO: is "joint" necessary?
// TODO: make congress and bill_num "0" unrepresentable

use winnow::ascii::alpha0;
use winnow::ascii::digit1;
use winnow::error::ErrMode;
use winnow::token::one_of;
use winnow::{PResult, Parser};

fn parse_congress<'s>(input: &mut &'s str) -> PResult<Congress<'s>> {
    let congress = digit1.parse_next(input).map_err(ErrMode::cut)?;

    if congress == "0" {
        todo!()
    } else {
        Ok(Congress(congress))
    }
}

fn parse_chamber(input: &mut &str) -> PResult<Chamber> {
    let chamber = one_of(('h', 'H', 's', 'S', 'j', 'J'))
        .parse_next(input)
        .map_err(ErrMode::cut)?;

    Ok(match chamber {
        'h' | 'H' => Chamber::House(chamber),
        'j' | 'J' => Chamber::Joint(chamber),
        's' | 'S' => Chamber::Senate(chamber),
        _ => unreachable!(),
    })
}

fn parse_legislation_type<'s>(input: &mut &'s str) -> PResult<LegislationType<'s>> {
    let leg_type = alpha0.parse_next(input).map_err(ErrMode::cut)?;
    Ok(match leg_type {
        "" => LegislationType::Bill(""),
        // TODO: how to exclude "sr"?
        "r" => LegislationType::Bill("r"),
        "e" | "res" => LegislationType::Resolution(ResolutionType::Simple),
        "c" | "cres" | "conres" => {
            LegislationType::Resolution(ResolutionType::Concurrent(leg_type))
        }
        "j" | "jres" => LegislationType::Resolution(ResolutionType::Joint(leg_type)),
        _ => unreachable!(),
    })
}

fn parse<'s>(input: &mut &'s str) -> PResult<Citation<'s>> {
    let (congress, chamber, leg_type, number) = (
        parse_congress,
        parse_chamber,
        parse_legislation_type,
        digit1,
    )
        .parse_next(input)
        .map_err(ErrMode::cut)?;

    if number == "0" {
        todo!()
    } else {
        Ok(Citation {
            congress,
            chamber,
            leg_type,
            number,
        })
    }
}

#[derive(Debug, PartialEq)]
struct Congress<'s>(&'s str);

#[derive(Debug, PartialEq)]
enum Chamber {
    House(char),
    Senate(char),
    Joint(char),
}

#[derive(Debug, PartialEq)]
enum ResolutionType<'s> {
    Simple,
    Concurrent(&'s str),
    Joint(&'s str),
}

#[derive(Debug, PartialEq)]
enum LegislationType<'s> {
    Bill(&'s str),
    Resolution(ResolutionType<'s>),
}

#[derive(Debug, PartialEq)]
struct Citation<'s> {
    congress: Congress<'s>,
    chamber: Chamber,
    leg_type: LegislationType<'s>,
    number: &'s str,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_house_bill() {
        let mut input = "118hr8070";
        let output = parse.parse_next(&mut input).unwrap();
        assert_eq!(input, "");
        assert_eq!(
            output,
            Citation {
                congress: Congress("118"),
                chamber: Chamber::House('h'),
                leg_type: LegislationType::Bill("r"),
                number: "8070"
            }
        )
    }

    #[test]
    fn test_parse_house_bill_short() {
        let mut input = "118h8070";
        let output = parse.parse_next(&mut input).unwrap();
        assert_eq!(input, "");
        assert_eq!(
            output,
            Citation {
                congress: Congress("118"),
                chamber: Chamber::House('h'),
                leg_type: LegislationType::Bill(""),
                number: "8070"
            }
        )
    }

    #[test]
    fn test_parse_house_simple_res() {
        let mut input = "103hres15";
        let output = parse.parse_next(&mut input).unwrap();
        assert_eq!(input, "");
        assert_eq!(
            output,
            Citation {
                congress: Congress("103"),
                chamber: Chamber::House('h'),
                leg_type: LegislationType::Resolution(ResolutionType::Simple),
                number: "15"
            }
        )
    }

    #[test]
    fn test_parse_house_simple_res_short() {
        let mut input = "103he15";
        let output = parse.parse_next(&mut input).unwrap();
        assert_eq!(input, "");
        assert_eq!(
            output,
            Citation {
                congress: Congress("103"),
                chamber: Chamber::House('h'),
                leg_type: LegislationType::Resolution(ResolutionType::Simple),
                number: "15"
            }
        )
    }
}
