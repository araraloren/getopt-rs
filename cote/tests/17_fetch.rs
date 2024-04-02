use aopt::prelude::AFwdParser;
use cote::prelude::*;

#[derive(Debug, PartialEq, Eq, CoteOpt)]
#[infer(val = i32)]
#[fetch(inner = i32, map = Speed)]
pub struct Speed(i32);

#[test]
fn fetch() {
    assert!(fetch_impl().is_ok());
}

fn fetch_impl() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut parser = AFwdParser::default();

    parser.add_opt("--speed".infer::<i32>())?;
    parser.parse(ARef::new(Args::from(["app", "--speed=42"].into_iter())))?;

    assert_eq!(Speed::fetch("--speed", parser.optset_mut())?, Speed(42));

    Ok(())
}
