use changelogen::cli::Cli;
use clap::Parser;

#[test]
fn unknown_subcommand_error() {
    let res = Cli::try_parse_from(["changelogen", "bogus"]);
    assert!(res.is_err());
}

#[test]
fn help_snapshot() {
    use clap::CommandFactory;
    let mut cmd = Cli::command();
    let help = cmd.render_long_help().to_string();
    insta::assert_snapshot!("cli_help", help);
}
