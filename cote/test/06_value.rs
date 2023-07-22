use cote::*;

#[derive(Debug, Cote)]
#[cote()]
pub struct Cli {
    #[arg(value = "tools")]
    name: String,

    #[pos(index = 1.., values = ["a", "b"])]
    args: Vec<String>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse(Args::from(["app", "c"].into_iter()))?;
    assert_eq!(cli.name.as_str(), "tools");
    assert_eq!(
        cli.args,
        vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
    );
    Ok(())
}
