use clap::Parser;
use novalyn::cli::Cli;

#[test]
fn unknown_subcommand_error() {
    let res = Cli::try_parse_from(["novalyn", "bogus"]);
    assert!(res.is_err());
}

#[test]
fn help_snapshot() {
    use clap::CommandFactory;
    let mut cmd = Cli::command();
    let help = cmd.render_long_help().to_string();
    insta::assert_snapshot!("cli_help", help);
}
