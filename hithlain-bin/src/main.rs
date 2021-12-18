use clap::{App, Arg, SubCommand, ArgMatches};
use clap::{crate_version, crate_authors, crate_description};
use hithlain::parse::lexer::lex;
use hithlain::parse::source::Source;
use hithlain::error::{HithlainError, NiceUnwrap};
use hithlain::parse::parser::Parser;
use hithlain::parse::desugar::desugar_program;
use hithlain::sim::Simulator;
use hithlain::sim::config::{SimulationConfig, VcdPath};
use hithlain::time::Duration;
use hithlain::miette;


fn main() {
    let matches = App::new("Hithlain")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("input")
                .required(true)
        )
        .subcommand(
            SubCommand::with_name("transpile")
                .arg(
                    Arg::with_name("output-format")
                        .short("f")
                        .long("format")
                        .help("Automatically determined based on output file name when format is omitted.")
                        .possible_values(&["vhdl", "verilog"])
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                )
        ).subcommand(
        SubCommand::with_name("simulate")
            .alias("sim")
            .alias("s").arg(
                Arg::with_name("entry")
                    .long("entry")
                    .short("e")
                    .takes_value(true)
                    .required(true)
                    .help("A test (or process without parameters and returns) to run")
            )
            .arg(
                Arg::with_name("time")
                    .long("time")
                    .short("t")
                    .help("How long to simulate for. (number followed by time unit in [ns, us, ms, s]). Leave empty to run to completion.")
            )
            .arg(
                Arg::with_name("overshoot")
                    .long("overshoot")
                    .help("Time buffer to add to the end of the generated vcd (number followed by time unit in [ns, us, ms, s])")
            )
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .short("output")
                    .help("File to output vcd to. Prints to stdout when `-` is provided")
                    .default_value("output.vcd")
            )
        ).subcommand(
        SubCommand::with_name("test")
                .alias("t")
                .arg(
                    Arg::with_name("name")
                        .takes_value(true)
                )
        )
        .get_matches();

    let filename = matches.value_of("input").expect("input file required");


    let lexed = lex(Source::file(filename).nice_unwrap()).nice_unwrap();
    let mut parser = Parser::new(lexed);
    let parsed = parser.parse_program().nice_unwrap();
    let desugared = desugar_program(parsed).nice_unwrap();


    match matches.subcommand() {
        ("transpile", args) => {

        }
        ("simulate", Some(args)) => {
            let entrypoint = args.value_of("entry").expect("entry point required");

            let mut cfg = SimulationConfig::default();

            cfg.create_vcd = true;
            cfg.vcd_path = VcdPath::Path(args.value_of("output").expect("has default").into());

            let sim = Simulator::new(desugared, cfg).nice_unwrap();
            sim.run_test(entrypoint).nice_unwrap();
        }
        ("test", Some(args)) => {
            let cfg = SimulationConfig::default();
            let sim = Simulator::new(desugared, cfg).nice_unwrap();

            if let Some(test_name) = args.value_of("test") {
                sim.run_test(test_name).nice_unwrap();
            } else {
                sim.run_all_tests().nice_unwrap();
            }
        }
        (s, _) => unreachable!("no such subcommand: {}", s)
    }
}
