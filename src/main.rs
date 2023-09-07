use rand::{SeedableRng, Rng};
use trimothy::TrimSlice;

mod flags;
mod algorithm;

type Arr2 = ndarray::Array2<f64>;
type Arr1 = ndarray::Array1<f64>;

fn main() -> anyhow::Result<()>{
    let opts = flags::Csvdimreduce::from_env_or_exit();
    let f = opts.get_istream()?;
    let mut f = opts.get_csv_reader().from_reader(f);

    let mut records = Vec::<csv::ByteRecord>::with_capacity(1024);
    let header : Option<csv::ByteRecord> = if f.has_headers() { Some(f.byte_headers()?.clone()) } else { None };
    for record in f.into_byte_records() {
        let record = record?;
        records.push(record);
    }

    let n_out_coords = opts.n_out_coords;
    let n_rows = records.len();
    let n_input_coords = opts.columns.0.len();
    let mut coords = Arr2::zeros((n_rows, n_out_coords + n_input_coords));
    let mut forces = Arr2::zeros((n_rows, n_out_coords + n_input_coords));
    let mut weights = Arr1::zeros(n_rows);

    for (j,record) in records.iter().enumerate() {
        let mut ctr = 0;
        let mut weight_debt = opts.weight;
        for (i,field) in record.iter().enumerate() {
            if opts.columns.0.contains(&(i+1)) {
                let field = field.trim();
                let x : f64 = std::str::from_utf8(field)?.parse()?;
                coords[(j,n_out_coords+ctr)] = x;
                ctr+=1;
            }
            if Some(i+1) == weight_debt {
                let field = field.trim();
                let x : f64 = std::str::from_utf8(field)?.parse()?;
                weights[j] = x;
                weight_debt = None;
            }
        }
        if ctr != opts.columns.0.len() {
            anyhow::bail!("Field list contains invalid column numbers");
        }
        if weight_debt.is_some() {
            anyhow::bail!("Weight column is not found");
        }
    }
    let mut rng = rand::rngs::StdRng::seed_from_u64(opts.random_seed.unwrap_or(1));
    for j in 0..n_rows {
        for i in 0..n_out_coords {
            coords[(j,i)] = rng.gen();
        }
    }
    if opts.weight.is_none() {
        weights.fill(1.0);
    }

    //println!("{} {}", data, weights);
    let mut state = algorithm::State {
        coords: coords.view_mut(),
        forces: forces.view_mut(),
        weights: weights.view(),
    };
    let mut tmp = Arr1::zeros(n_out_coords);
    let rate_decay = opts.rate_decay.unwrap_or(0.95);
    let mut params = algorithm::Params {
        coordinate_range_to_consider: 0..n_out_coords,
        coordinate_range_to_clamp: 0..n_out_coords,
        rate: opts.rate.unwrap_or(0.1),
        central_force: opts.central_force.unwrap_or(8.0),
        tmp: tmp.view_mut(),
    };
    for _ in 0..opts.n_iters.unwrap_or(100) {
        state.step(&mut params);
        params.rate *= rate_decay;
    }

    let f = opts.get_ostream()?;
    let mut f = opts.get_csv_writer().from_writer(f);
    if let Some(h) = &header {
        for i in 1..=n_out_coords {
            f.write_field(format!("coord{}", i))?;
        }
        f.write_field("")?;
        f.write_byte_record(h)?;
    }

    for (j,record) in records.iter().enumerate() {
        for i in 0..n_out_coords {
            f.write_field(format!("{:.4}", coords[(j,i)]))?;
        }
        f.write_field("")?;
        f.write_byte_record(&record)?;
    }

    Ok(())
}
