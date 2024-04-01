use cote::{aopt::prelude::AFwdParser, *};
use regex::Regex;

#[derive(Debug, CoteVal, CoteOpt, PartialEq)]
#[coteval(mapstr = Meal::new)]
pub enum Meal {
    BreakFast,

    Lunch,

    Dinner,
}

impl Meal {
    pub fn new(value: &str) -> Result<Self, CoteError> {
        match value {
            "breakfast" => Ok(Self::BreakFast),
            "lunch" => Ok(Self::Lunch),
            "dinner" => Ok(Self::Dinner),
            name => {
                panic!("Unknow {name} for Meal")
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, CoteOpt, CoteVal)]
#[coteval(mapraw = Point::new)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn new(raw: Option<&RawVal>, _: &Ctx) -> Result<Self, CoteError> {
        let regex = Regex::new(r"[\{\[\(]\s*(\d+)\s*\,\s*(\d+)\s*[\}\]\)]").unwrap();
        if let Some(captures) = regex.captures(raw2str(raw)?) {
            let mut x = 0;
            let mut y = 0;

            if let Some(mat) = captures.get(1) {
                x = mat.as_str().parse::<i32>().expect(&format!(
                    "Point.x must be a valid number: `{}`",
                    mat.as_str()
                ));
            }
            if let Some(mat) = captures.get(2) {
                y = mat.as_str().parse::<i32>().expect(&format!(
                    "Point.y must be a valid number: `{}`",
                    mat.as_str()
                ));
            }
            return Ok(Point { x, y });
        }
        panic!("Can not parsing value of Point")
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut parser = AFwdParser::default();

    parser.add_opt_i::<Point>("-p;--point")?;
    parser.add_opt_i::<Meal>("--meal")?;
    parser.parse(ARef::new(Args::from(
        ["app", "-p={42,2}", "--meal=lunch"].into_iter(),
    )))?;
    
    assert_eq!(
        Point::fetch("--point", parser.optset_mut())?,
        Point { x: 42, y: 2 }
    );
    assert_eq!(Meal::fetch("--meal", parser.optset_mut())?, Meal::Lunch);
    Ok(())
}
