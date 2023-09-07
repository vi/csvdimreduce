use interpolation::lerp;
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
    let mut coords = Arr2::zeros((n_rows, n_out_coords));
    let mut forces = Arr2::zeros((n_rows, n_out_coords));
    let mut weights = Arr1::zeros(n_rows);

    let mut inputvals = Arr2::zeros((n_rows, n_input_coords));
    let mut affinities = Arr2::zeros((n_rows, n_rows));

    for (j,record) in records.iter().enumerate() {
        let mut ctr = 0;
        let mut weight_debt = opts.weight;
        for (i,field) in record.iter().enumerate() {
            if opts.columns.0.contains(&(i+1)) {
                let field = field.trim();
                let x : f64 = std::str::from_utf8(field)?.parse()?;
                inputvals[(j,ctr)] = x;
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
    algorithm::build_particle_affinities(inputvals.view(), affinities.view_mut(), opts.same_particle_force.unwrap_or(0.01));
    drop(inputvals);
    let mut rng = rand::rngs::StdRng::seed_from_u64(opts.random_seed.unwrap_or(1));
    for j in 0..n_rows {
        for i in 0..n_out_coords {
            coords[(j,i)] = rng.gen();
        }
    }
    if opts.weight.is_none() {
        weights.fill(1.0);
    }

    let squeeze_n_coords = opts.squeeze_n_coords.unwrap_or(0);
    let squeeze_rampup_rate = opts.squeeze_rampup_rate.unwrap_or(0.001);
    let squeeze_rampup_iters = opts.squeeze_rampup_iters.unwrap_or(0);
    let squeeze_final_force = opts.squeeze_final_force.unwrap_or(1000.0);
    let squeeze_final_initial_rate = opts.squeeze_final_initial_rate.unwrap_or(0.001);
    let squeeze_final_iters = opts.squeeze_final_iters.unwrap_or(0);
    let n_iters = opts.n_iters.unwrap_or(100);
    let rate = opts.rate.unwrap_or(0.1);
    let rate_decay = opts.rate_decay.unwrap_or(0.95);
    let central_force = opts.central_force.unwrap_or(30.0);

    //println!("{} {}", data, weights);
    let mut state = algorithm::State {
        coords: coords.view_mut(),
        forces: forces.view_mut(),
        weights: weights.view(),
        affinities: affinities.view(),
    };
    let mut tmp = Arr1::zeros(n_out_coords);
    let mut params = algorithm::Params {
        rate,
        central_force,
        tmp: tmp.view_mut(),
        squeeze_from: squeeze_n_coords,
        squeeze_force: central_force,
    };
    for _ in 0..n_iters {
        state.step(&mut params);
        params.rate *= rate_decay;
    }
    for q in 0..squeeze_rampup_iters {
        params.squeeze_force = lerp(&central_force.ln(), &squeeze_final_force.ln(), &((q+1) as f64 / squeeze_final_iters as f64)).exp();
        params.rate = squeeze_rampup_rate;
        state.step(&mut params);
    }
    params.rate = squeeze_final_initial_rate;
    for _ in 0..squeeze_final_iters {
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
