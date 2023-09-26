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
        /// 
        /// This includes temporary coordinates used for squeezing (-S).
        required n_out_coords: usize
        /// Input csv file. Use stdin if absent.
        optional path: PathBuf
        optional --save-each-n-iters n : usize
        /// First line of the CSV is not headers
        optional --no-header
        /// Field delimiter in CSV files. Comma by default.
        optional --delimiter delimiter : DelimiterSpecifier
        /// Override line delimiter in CSV files.
        optional --record-delimiter delimiter : DelimiterSpecifier
        /// Save file there instead of stdout
        optional -o,--output path: PathBuf
        /// Initial particle positions
        optional --random-seed seed: u64
        /// Use this column as weights
        optional -w,--weight column_number: usize
        /// Basic number of iterations. Default is 100.
        /// Note that complexity of each iteration is quadratic of number of lines in CSV.
        optional -n, --n-iters n: usize
        /// Initial rate of change i.e. distance the fastest particle travels per iteration.
        /// Default is 0.01.
        optional -r,--rate rate: f64
        /// Apply each movement multiplpe times, decaying it by this factor. Default is 0.9.
        optional --inertia-multiplier x: f64
        /// Ramp down rate of change to this value at the end.
        optional -R,--final-rate final_decay: f64
        /// Attract particles' coordinates to 0.5 with this strenght (relative to average inter-particle forces).
        optional -c,--central-force f: f64
        /// Additional repelling force between particles (even those with the same parameters). Default is 0.2
        optional -F,--same-particle-force f: f64
        /// After doing usual iterations, perform additional steps to "flatten" the shape into fewer dimension count (squeeze phase).
        /// Specified number of coodinaes are retained. For others, the `-c` central force is crancked up to `-C`, so they
        /// (should) become flat "0.5" in the end.
        /// This produces better results compared to just having that number of coordinates from the beginning.
        optional -S,--retain_coords_from_squeezing n: usize
        /// Use this `-r` rate when doing the squeeze phase.
        optional --squeeze-rampup-rate rate: f64
        /// User this number of iterations of the first phase of squeezing phase.
        /// This applies to each squeezed dimension sequentially.
        optional --squeeze-rampup-iters n: usize
        /// This this central force for the squeezed dimensions.
        /// The force is gradually increased from `-C` to this value during the rampup phase.
        optional -C,--squeeze-final-force f: f64
        /// Override `-r` rate for the second phase of squeezing. It decays with `-d` each iteration.
        optional --squeeze-final-initial-rate rate : f64
        /// Number of iterations of the second phase of squeezeing (where central force no longer changes, just to increase precision)
        optional --squeeze-final-iters n : usize
        /// Gradually increase rate from zero during this number of iterations. Defaults to 10.
        optional --warmup-iterations n : usize
        /// Print various values, including algorithm parameter values
        optional --debug
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

    pub save_each_n_iters: Option<usize>,
    pub no_header: bool,
    pub delimiter: Option<DelimiterSpecifier>,
    pub record_delimiter: Option<DelimiterSpecifier>,
    pub output: Option<PathBuf>,
    pub random_seed: Option<u64>,
    pub weight: Option<usize>,
    pub n_iters: Option<usize>,
    pub rate: Option<f64>,
    pub inertia_multiplier: Option<f64>,
    pub final_rate: Option<f64>,
    pub central_force: Option<f64>,
    pub same_particle_force: Option<f64>,
    pub retain_coords_from_squeezing: Option<usize>,
    pub squeeze_rampup_rate: Option<f64>,
    pub squeeze_rampup_iters: Option<usize>,
    pub squeeze_final_force: Option<f64>,
    pub squeeze_final_initial_rate: Option<f64>,
    pub squeeze_final_iters: Option<usize>,
    pub warmup_iterations: Option<usize>,
    pub debug: bool,
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
