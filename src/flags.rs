use std::{path::PathBuf, str::FromStr, collections::BTreeSet};


#[derive(Debug)]
pub struct ColumnsSpecifier(pub BTreeSet<usize>);
#[derive(Debug)]
pub struct DelimiterSpecifier(pub u8);

impl FromStr for ColumnsSpecifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p = number_range::NumberRange::default();
        Ok(ColumnsSpecifier(p.parse_str(s)?.collect()))
    }
}

impl FromStr for DelimiterSpecifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_ascii() && s.len() == 1 {
            Ok(DelimiterSpecifier(s.as_bytes()[0]))
        } else {
            Err(anyhow::anyhow!("Delimiter should be exactly one ASCII character"))
        }
    }
}

impl Csvdimreduce {
    pub fn get_csv_reader(&self) -> csv::ReaderBuilder {
        let mut b = csv::ReaderBuilder::new();
        if self.no_header {
            b.has_headers(false);
        }
        if let Some(ref x) = self.delimiter {
            b.delimiter(x.0);
        }
        if let Some(ref x) = self.record_delimiter {
            b.terminator(csv::Terminator::Any(x.0));
        }
        b
    }
    pub fn get_csv_writer(&self) -> csv::WriterBuilder {
        let mut b = csv::WriterBuilder::new();
        if self.no_header {
            b.has_headers(false);
        }
        if let Some(ref x) = self.delimiter {
            b.delimiter(x.0);
        }
        if let Some(ref x) = self.record_delimiter {
            b.terminator(csv::Terminator::Any(x.0));
        }
        b
    }

    pub fn get_istream(&self) -> anyhow::Result<Box<dyn std::io::Read>> {
        if let Some(ref f) = self.path {
            Ok(Box::new(std::fs::File::open(f)?))
        } else {
            Ok(Box::new(std::io::stdin()))
        }
    }

    pub fn get_ostream(&self) -> anyhow::Result<Box<dyn std::io::Write>> {
        if let Some(ref f) = self.output {
            Ok(Box::new(std::fs::File::create(f)?))
        } else {
            Ok(Box::new(std::io::stdout()))
        }
    }
}

xflags::xflags! {
    src "./src/flags.rs"

    cmd csvdimreduce {
        /// List of columns to use as coordinates. First column is number 1. Parsing support ranges with steps like 3,4,10:5:100.
        /// See `number_range` Rust crate for details.
        /// Use `xsv headers your_file.csv` to find out column numbers.
        required columns: ColumnsSpecifier
        /// Number of output coordinates (new fields in CSV containing computed values)
        required n_out_coords: usize
        optional path: PathBuf
        optional --debug
        optional --no-header
        optional --delimiter delimiter : DelimiterSpecifier
        optional --record-delimiter delimiter : DelimiterSpecifier
        optional -o,--output path: PathBuf
        optional --random-seed seed: u64
        /// Use this column as weights
        optional -w,--weight column_number: usize
    }
}
// generated start
// The following code is generated by `xflags` macro.
// Run `env UPDATE_XFLAGS=1 cargo build` to regenerate.
#[derive(Debug)]
pub struct Csvdimreduce {
    pub columns: ColumnsSpecifier,
    pub n_out_coords: usize,
    pub path: Option<PathBuf>,

    pub debug: bool,
    pub no_header: bool,
    pub delimiter: Option<DelimiterSpecifier>,
    pub record_delimiter: Option<DelimiterSpecifier>,
    pub output: Option<PathBuf>,
    pub random_seed: Option<u64>,
    pub weight: Option<usize>,
}

impl Csvdimreduce {
    #[allow(dead_code)]
    pub fn from_env_or_exit() -> Self {
        Self::from_env_or_exit_()
    }

    #[allow(dead_code)]
    pub fn from_env() -> xflags::Result<Self> {
        Self::from_env_()
    }

    #[allow(dead_code)]
    pub fn from_vec(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
        Self::from_vec_(args)
    }
}
// generated end
