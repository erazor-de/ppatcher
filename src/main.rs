use ppatch::{prelude::*, Pattern, PatternSearchType};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path;
use std::process;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opt {
    /// suppress textual output of found matches
    #[structopt(short, long)]
    quiet: bool,

    /// pattern given as hex, octal or binary with ? as wildcard
    #[structopt(short, long)]
    search: Pattern<u8>,

    /// pattern given as hex, octal or binary with ? as wildcard
    #[structopt(short, long)]
    replace: Option<Pattern<u8>>,

    /// ignore first n matches
    #[structopt(long)]
    skip: Option<usize>,

    /// just take n matches, ignoring later matches
    #[structopt(long)]
    take: Option<usize>,

    /// default is stdin
    #[structopt(short, long, parse(from_os_str))]
    infile: Option<path::PathBuf>,

    /// default is stdout
    #[structopt(short, long, parse(from_os_str))]
    outfile: Option<path::PathBuf>,
}

fn run_app(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    // NOTE No binary input via stdin on Windows
    let reader: Box<dyn Read> = match &opt.infile {
        Some(infile) => Box::new(io::BufReader::new(File::open(infile)?)),
        None => Box::new(io::stdin()),
    };

    let mut a: Box<dyn Iterator<Item = Result<PatternSearchType<u8>, std::io::Error>>> =
        Box::new(reader.bytes().search_pattern(&opt.search));

    if let Some(count) = opt.skip {
        a = Box::new(a.skip_pattern(count));
    }

    if let Some(count) = opt.take {
        a = Box::new(a.take_pattern(count));
    }

    if !opt.quiet {
        a = Box::new(a.inspect(|result| {
            if let Ok(PatternSearchType::Match { ref data, index }) = result {
                println!("{:#X}: {:X?}", index, data);
            }
        }))
    }

    match opt.replace {
        Some(ref pattern) => {
            let mut writer: Box<dyn Write> = match &opt.outfile {
                Some(outfile) => Box::new(io::BufWriter::new(File::create(outfile)?)),
                None => Box::new(io::stdout()),
            };

            for result in a.replace_pattern(&pattern) {
                match result {
                    Err(error) => {
                        return Err(error.into());
                    }
                    Ok(value) => {
                        writer.write_all(&[value])?;
                    }
                }
            }
        }
        None => for _ in a {},
    }

    Ok(())
}

fn main() {
    let opt = Opt::from_args();
    process::exit(match run_app(opt) {
        Ok(_) => 0,
        Err(error) => {
            eprintln!("{}", error);
            1
        }
    });
}
