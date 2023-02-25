//! The command line interface for the datalog engine.

use clap::{Parser, ValueEnum};
use miette::{Diagnostic, GraphicalReportHandler, IntoDiagnostic, NamedSource, Report, Result};
use rustyline::{error::ReadlineError, Editor};

use std::{ffi::OsString, fs};

use datalog::{BlockList, DataSet, Error, Program, Query, Repl};

#[derive(Debug, clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The name of an input file to load as a set of facts.
    #[arg()]
    filename: Option<OsString>,

    /// A query to run. If this is not specified, a repl is started.
    #[arg(short, long)]
    query: Option<String>,

    /// Filter out left-hand letters for a keyboard layout.
    #[arg(long, value_enum, default_value_t)]
    filter: Filter,

    /// Launch the interactive repl after loading a file. This is the default
    /// behaviour when no FILENAME is given.
    #[arg(long, short, conflicts_with = "query")]
    repl: bool,
}

#[derive(Debug, Default, Clone, Copy, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
pub enum Filter {
    Off,
    #[default]
    Qwerty,
    Dvorak,
    Colemak,
}

impl From<Filter> for BlockList {
    fn from(val: Filter) -> Self {
        BlockList::from_disallowed(match val {
            Filter::Off => "",
            Filter::Qwerty => "qwertasdfgzxczvb123456",
            Filter::Dvorak => "aoeuptqjkx123456",
            Filter::Colemak => "qwfpgarstdzxcvb12345",
        })
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut data = DataSet::default();

    let blocked = args.filter.into();

    if let Some(filename) = args.filename.as_deref() {
        let input = fs::read_to_string(filename).into_diagnostic()?;

        let program = Program::parse(input.as_str(), blocked).map_err(|errors| {
            Report::from(errors)
                .with_source_code(NamedSource::new(filename.to_string_lossy(), input))
        })?;

        data.program(&program);

        if args.query.is_none() {
            println!(
                "...loaded file {} successfully.",
                filename.to_string_lossy()
            );
        }
    }

    if let Some(query) = args.query {
        let query = Query::parse(query.as_str(), blocked).map_err(|errors| {
            Report::from(errors).with_source_code(NamedSource::new("--query", query))
        })?;

        data.run();
        print_query_answers(&query, &mut data);
        Ok(())
    } else if args.repl || args.filename.is_none() {
        repl(data, blocked)
    } else {
        data.run();
        println!("{data}");
        Ok(())
    }
}

fn print_query_answers(query: &Query, data: &mut DataSet) {
    let answers = data.query(query);

    if answers.is_empty() {
        println!("<no answers>");
    } else {
        for answer in answers {
            println!("{}", answer);
        }
    }
}

fn repl(mut data: DataSet, blocked: BlockList) -> Result<()> {
    let mut rl = Editor::<()>::new().into_diagnostic()?;
    let mut line_count = 1;
    let handler = GraphicalReportHandler::new();

    loop {
        let line = rl.readline(">> ");
        let mut buf = String::new();

        match line {
            Ok(line) => {
                if let Err(error) = repl_step(&line, &mut data, blocked) {
                    if line == "quit" || line == "exit" {
                        println!("hint: use control-d to leave");
                    }

                    buf.clear();
                    let diagnostic = error
                        .with_source_code(NamedSource::new(format!("<repl:{line_count}>"), line));
                    let _ = handler.render_report(&mut buf, &diagnostic as &dyn Diagnostic);

                    println!("{}", buf);
                }

                line_count += 1;
            }

            // Control-C goes back to fresh prompt, like in the shell.
            Err(ReadlineError::Interrupted) => {
                continue;
            }

            // Control-D quits
            Err(ReadlineError::Eof) => {
                println!("goodbye!");
                return Ok(());
            }

            Err(e) => {
                return Err(e).into_diagnostic();
            }
        }
    }
}

fn repl_step(input: &str, data: &mut DataSet, blocked: BlockList) -> Result<(), Error> {
    let syntax = Repl::parse(input, blocked).map_err(Error::from)?;

    match syntax {
        Repl::Program(p) => data.program(&p),
        Repl::Query(query) => {
            data.run();
            print_query_answers(&query, data);
        }
    }

    Ok(())
}
