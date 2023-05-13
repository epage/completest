#![allow(dead_code)]

use bpaf::*;

#[test]
fn ambiguity() {
    set_override(false);
    #[derive(Debug, Clone)]
    enum A {
        V(Vec<bool>),
        W(String),
    }

    let a0 = short('a').switch().many().map(A::V);
    let a1 = short('a').argument::<String>("AAAAAA").map(A::W);
    let parser = construct!([a0, a1]).to_options();

    let r = parser
        .run_inner(Args::from(&["-aaaaaa"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "Parser supports -a as both option and option-argument, try to split -aaaaaa into individual options (-a -a ..) or use -a=aaaaa syntax to disambiguate");

    let r = parser
        .run_inner(Args::from(&["-b"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "No such flag: `-b`, did you mean `-a`?");
}

#[test]
fn short_cmd() {
    set_override(false);
    let parser = long("alpha")
        .req_flag(())
        .to_options()
        .command("beta")
        .short('b')
        .to_options();

    let r = parser
        .run_inner(Args::from(&["c"]))
        .unwrap_err()
        .unwrap_stderr();

    assert_eq!(
        r,
        "No such command or positional: `c`, did you mean `beta`?"
    );
}

#[test]
fn double_dashes_no_fallback() {
    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(options)]
    enum Opts {
        Llvm,
        Att,
        #[bpaf(hide)]
        Dummy,
    }

    let r = opts()
        .run_inner(Args::from(&["-llvm"]))
        .unwrap_err()
        .unwrap_stderr();

    assert_eq!(
        r,
        "No such flag: -llvm (with one dash), did you mean `--llvm`?"
    );
}

#[test]
fn double_dashes_fallback() {
    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(options, fallback(Opts::Dummy))]
    enum Opts {
        Llvm,
        Att,
        Dummy,
    }

    let r = opts()
        .run_inner(Args::from(&["-llvm"]))
        .unwrap_err()
        .unwrap_stderr();

    assert_eq!(
        r,
        "No such flag: -llvm (with one dash), did you mean `--llvm`?"
    );
}

#[test]
fn double_dash_with_optional_positional() {
    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(fallback(Opts::Dummy))]
    enum Opts {
        Llvm,
        Att,
        Dummy,
    }

    let pos = positional::<String>("FILE").optional();
    let parser = construct!(opts(), pos).to_options();

    let r = parser
        .run_inner(Args::from(&["make", "-llvm"]))
        .unwrap_err()
        .unwrap_stderr();

    assert_eq!(
        r,
        "No such flag: -llvm (with one dash), did you mean `--llvm`?"
    );
}

#[test]
fn inside_out_command_parser() {
    #[derive(Debug, Bpaf, Clone, PartialEq)]
    #[bpaf(options)]
    enum Cmd {
        #[bpaf(command)]
        Log {
            #[bpaf(long)]
            oneline: bool,
        },
    }

    let ok = cmd().run_inner(Args::from(&["log", "--oneline"])).unwrap();
    assert_eq!(ok, Cmd::Log { oneline: true });

    // Can't parse "--oneline log" because oneline could be an argument instead of a flag
    // so log might not be a command, but we can try to make a better suggestion.
    let r = cmd()
        .run_inner(Args::from(&["--oneline", "log"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "flag: `--oneline` is not valid in this context, did you mean to pass it to command \"log\"?");
}

#[test]
fn flag_specified_twice() {
    let parser = long("flag").switch().to_options();

    let r = parser
        .run_inner(Args::from(&["--flag", "--flag"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "--flag is not expected in this context");
}

#[test]
fn ux_discussion() {
    #[derive(Debug, Clone, Bpaf)]
    #[bpaf(adjacent)]
    pub struct ConfigSetBool {
        /// Set <key> to <bool>
        #[bpaf(long("setBool"))]
        set_bool: (),
        /// Configuration key
        #[bpaf(positional("key"))]
        key: String,
        /// Configuration Value (bool)
        #[bpaf(positional("bool"))]
        value: bool,
    }

    let aa = long("bool-flag").switch();
    let parser = construct!(config_set_bool(), aa).to_options();

    let r = parser
        .run_inner(Args::from(&["--setBool", "key", "tru"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(
        r,
        // everything before ":" comes from bpaf, after ":" - it's an error specific
        // to FromStr instance.
        "Couldn't parse \"tru\": provided string was not `true` or `false`"
    );

    let r = parser
        .run_inner(Args::from(&["--bool-fla"]))
        .unwrap_err()
        .unwrap_stderr();

    assert_eq!(r, "No such flag: `--bool-fla`, did you mean `--bool-flag`?");

    let r = parser
        .run_inner(Args::from(&["--bool-flag", "--bool-flag"]))
        .unwrap_err()
        .unwrap_stderr();

    assert_eq!(
        r,
        "Expected --setBool, got \"--bool-flag\". Pass --help for usage information"
    );
}

#[test]
fn suggest_typo_fix() {
    let p = long("flag").switch().to_options();

    let r = p
        .run_inner(Args::from(&["--fla"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "No such flag: `--fla`, did you mean `--flag`?");

    let r = p
        .run_inner(Args::from(&["--fla", "--fla"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "No such flag: `--fla`, did you mean `--flag`?");

    let r = p
        .run_inner(Args::from(&["--flag", "--flag"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "--flag is not expected in this context");
}

#[test]
fn better_error_message_with_typos() {
    #[derive(Bpaf, Clone, Debug)]
    #[bpaf(options)]
    enum Commands {
        /// Multi
        ///  Line
        ///  Comment
        #[bpaf(command)]
        Lines {},

        #[bpaf(command)]
        Arguments(#[bpaf(external(arguments))] Arguments),
    }

    #[derive(Bpaf, Clone, Debug)]
    struct Arguments {
        #[bpaf(short('e'), argument("Arg"))]
        env: Vec<String>,

        #[bpaf(positional("POS"))]
        args: Vec<String>,
    }

    let r = arguments()
        .to_options()
        .run_inner(Args::from(&["-a", "erg"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "No such flag: `-a`, did you mean `-e`?");

    let r = commands()
        .run_inner(Args::from(&["arguments", "-a", "erg"]))
        .unwrap_err()
        .unwrap_stderr();
    assert_eq!(r, "No such flag: `-a`, did you mean `-e`?");

    let r = arguments()
        .to_options()
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "\
Usage: [-e Arg]... [<POS>]...

Available options:
    -e <Arg>
    -h, --help  Prints help information
";
    assert_eq!(r, expected);

    let r = commands()
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "\
Usage: COMMAND ...

Available options:
    -h, --help  Prints help information

Available commands:
    lines      Multi
               Line
               Comment
    arguments
";
    assert_eq!(r, expected);
}

#[test]
fn big_conflict() {
    let a = short('a').switch();
    let b = short('b').switch();
    let c = short('c').switch();
    let d = short('d').switch();
    let ab = construct!(a, b);
    let cd = construct!(c, d);
    let parser = construct!([ab, cd]).to_options();
    let r = parser
        .run_inner(Args::from(&["-a", "-b", "-c", "-d"]))
        .unwrap_err()
        .unwrap_stderr();
    let expected = "[-c] [-d] cannot be used at the same time as [-a] [-b]";
    assert_eq!(r, expected);
}

#[test]
fn pure_conflict() {
    let a = short('a').switch();
    let b = pure(false);
    let parser = construct!([a, b]).to_options();
    let r = parser.run_inner(Args::from(&[])).unwrap();
    assert!(!r);
    let r = parser.run_inner(Args::from(&["-a"])).unwrap();
    assert!(r);
}
