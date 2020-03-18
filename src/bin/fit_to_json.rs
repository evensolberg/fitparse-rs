use fitparser::{parse_file, FitFile};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

/// Parse FIT formatted files and output them as JSON
#[derive(Debug, StructOpt)]
#[structopt(name = "fit_to_json")]
struct Cli {
    /// FIT files to convert to JSON
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,

    /// Output location, if not provided the JSON file will be output alongside the input file. If a
    /// directory is provided all FIT files will be written there using the same filename but with
    /// a JSON extension. If multiple FIT files are provided and the output path isn't a directory
    /// the JSON data will be an array of FIT files.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
}

#[derive(Clone, Debug)]
enum OutputLocation {
    Inplace,
    LocalDirectory(PathBuf),
    LocalFile(PathBuf),
}

impl OutputLocation {
    fn new(location: PathBuf) -> Self {
        if location.is_dir() {
            OutputLocation::LocalDirectory(location)
        } else {
            OutputLocation::LocalFile(location)
        }
    }

    fn write_json_file(&self, filename: &PathBuf, data: &[FitFile]) -> std::io::Result<()> {
        let j = if data.len() == 1 {
            serde_json::to_string(&data[0]).unwrap()
        } else {
            serde_json::to_string(data).unwrap()
        };

        let outname = match self {
            Self::Inplace => filename.with_extension("json"),
            Self::LocalDirectory(dest) => dest
                .clone()
                .join(filename.file_name().unwrap())
                .with_extension("json"),
            Self::LocalFile(dest) => dest.clone(),
        };

        let mut f = File::create(outname).unwrap();
        f.write_all(j.as_bytes())
    }
}

fn main() {
    let opt = Cli::from_args();
    let output_loc = opt
        .output
        .map_or(OutputLocation::Inplace, |loc| OutputLocation::new(loc));
    let collect_all = match output_loc {
        OutputLocation::LocalFile(_) => true,
        _ => false,
    };

    // Read each FIT file and output it
    let mut fit_data: Vec<FitFile> = Vec::new();
    let mut buffer = Vec::new();
    for file in opt.files {
        // read the whole file incase we have chained FIT files
        let mut f = File::open(&file).unwrap();
        f.read_to_end(&mut buffer).unwrap();

        // process FIT files until no bytes remain
        let mut remaining: &[u8] = &buffer;
        while remaining.len() > 0 {
            let (r, fitfile) = parse_file(&remaining).unwrap();
            remaining = r;
            fit_data.push(fitfile)
        }

        // output a single fit file's data into a single output file
        if !collect_all {
            output_loc.write_json_file(&file, &fit_data).unwrap();
            fit_data.clear();
        }

        buffer.clear()
    }
    // output fit data from all files into a single file
    if collect_all {
        output_loc
            .write_json_file(&PathBuf::new(), &fit_data)
            .unwrap();
    }
}
