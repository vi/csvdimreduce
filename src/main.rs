use std::io::Write;

use interpolation::lerp;
use rand::{Rng, SeedableRng};
use trimothy::TrimSlice;

mod algorithm;
mod flags;

type Arr2 = ndarray::Array2<f64>;
type Arr1 = ndarray::Array1<f64>;

fn main() -> anyhow::Result<()> {
    let opts = flags::Csvdimreduce::from_env_or_exit();
    let f = opts.get_istream()?;
    let mut f = opts.get_csv_reader().from_reader(f);

    let mut records = Vec::<csv::ByteRecord>::with_capacity(1024);
    let header: Option<csv::ByteRecord> = if f.has_headers() {
        Some(f.byte_headers()?.clone())
    } else {
        None
    };
    for record in f.into_byte_records() {
        let record = record?;
        records.push(record);
    }

    let n_out_coords = opts.n_out_coords;
    let n_rows = records.len();
    let n_input_coords = opts.columns.0.len();
    let mut coords = Arr2::zeros((n_rows, n_out_coords));
    let mut forces = Arr2::zeros((n_rows, n_out_coords));
    let mut inertias = Arr2::zeros((n_rows, n_out_coords));
    let mut weights = Arr1::zeros(n_rows);

    let mut inputvals = Arr2::zeros((n_rows, n_input_coords));
    let mut affinities = Arr2::zeros((n_rows, n_rows));

    for (j, record) in records.iter().enumerate() {
        let mut ctr = 0;
        let mut weight_debt = opts.weight;
        for (i, field) in record.iter().enumerate() {
            if opts.columns.0.contains(&(i + 1)) {
                let field = field.trim();
                let x: f64 = std::str::from_utf8(field)?.parse()?;
                inputvals[(j, ctr)] = x;
                ctr += 1;
            }
            if Some(i + 1) == weight_debt {
                let field = field.trim();
                let x: f64 = std::str::from_utf8(field)?.parse()?;
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
    algorithm::build_particle_affinities(
        inputvals.view(),
        affinities.view_mut(),
        opts.same_particle_force.unwrap_or(0.2),
    );
    drop(inputvals);
    let mut rng = rand::rngs::StdRng::seed_from_u64(opts.random_seed.unwrap_or(1));
    for j in 0..n_rows {
        for i in 0..n_out_coords {
            coords[(j, i)] = rng.gen();
        }
    }
    if opts.weight.is_none() {
        weights.fill(1.0);
    }

    let avgaff = algorithm::average_affinity(affinities.view());

    let n_iters = opts.n_iters.unwrap_or(100);
    let warnup_iters = opts.warmup_iterations.unwrap_or(n_iters/2);
    let rate = opts.rate.unwrap_or(0.01);
    let inertia_multiplier = opts.inertia_multiplier.unwrap_or(0.9);
    let final_rate = opts.final_rate.unwrap_or(0.02 * rate);
    let central_force_param = opts.central_force.unwrap_or(20.0);
    let central_force = avgaff * central_force_param;
    let squeeze_rampup_rate = opts.squeeze_rampup_rate.unwrap_or(rate * 0.2);
    let squeeze_rampup_iters =
        opts.squeeze_rampup_iters
            .unwrap_or(if opts.retain_coords_from_squeezing.is_some() {
                n_iters * 1
            } else {
                0
            });
    let squeeze_final_iters =
        opts.squeeze_final_iters
            .unwrap_or(if opts.retain_coords_from_squeezing.is_some() {
                n_iters
            } else {
                0
            });
    let squeeze_final_force_param = opts
        .squeeze_final_force
        .unwrap_or(10.0 * central_force_param);
    let squeeze_final_force = avgaff * squeeze_final_force_param;
    let squeeze_final_initial_rate = opts
        .squeeze_final_initial_rate
        .unwrap_or(squeeze_rampup_rate);

    if opts.debug {
        println!("params basic_iters={n_iters} warmup_iters={warnup_iters} \
        base_rate={rate} inertia_multiplier={inertia_multiplier} final_rate={final_rate} central_force={central_force_param} \
        squeeze_rampup_rate={squeeze_rampup_rate} squeeze_rampup_iters={squeeze_rampup_iters} squeeze_final_iters={squeeze_final_iters} \
        squeeze_final_force={squeeze_final_force_param} squeeze_final_initial_rate={squeeze_final_initial_rate} avgaff={avgaff}");
    }

    let mut total_iter_count = 0usize;

    let mut inc_iter_and_maybe_save = |cv: ndarray::ArrayView2<'_, f64>| {
        if let Some(se) = opts.save_each_n_iters {
            if total_iter_count % se == 0 {
                let Ok(f) = opts
                    .get_csv_writer()
                    .from_path(format!("debug{:05}.csv", total_iter_count)) else { return };
                let _ = save_csv(&header, n_out_coords, f, &records, cv);
            }
        }
        total_iter_count += 1;
    };

    //println!("{} {}", data, weights);
    let mut tmp = Arr1::zeros(n_out_coords);
    let mut state = algorithm::State {
        coords: coords.view_mut(),
        forces: forces.view_mut(),
        inertias: inertias.view_mut(),
        weights: weights.view(),
        affinities: affinities.view(),
        tmp: tmp.view_mut(),
        movement_scaler: 0.0,
    };
    let mut params = algorithm::Params {
        rate,
        central_force,
        squeeze_from: n_out_coords,
        squeeze_force: central_force,
        squeeze_force2: squeeze_final_force,
        inertia_multiplier,
        debug: opts.debug,
    };
    for q in 0..n_iters {
        inc_iter_and_maybe_save(state.coords.view());

        if q < warnup_iters {
            let t = (q + 1) as f64 / n_iters as f64;
            params.rate = lerp(&(rate * 0.1), &(rate), &t);
        } else if squeeze_final_iters == 0 {
            params.rate = lerp(
                &(rate * rate),
                &(final_rate * final_rate),
                &((q + 1) as f64 / n_iters as f64),
            )
            .sqrt();
        } else {
            params.rate = rate;
        }

        state.step(&mut params);
    }
    let coords_to_squeeze =
        n_out_coords.saturating_sub(opts.retain_coords_from_squeezing.unwrap_or(n_out_coords));
    for squeze_this_number_of_coords in 1..=coords_to_squeeze {
        state.inertias.fill(0.0);
        params.squeeze_from = n_out_coords - squeze_this_number_of_coords;
        for q in 0..squeeze_rampup_iters {
            inc_iter_and_maybe_save(state.coords.view());
            params.squeeze_force = lerp(
                &central_force.ln(),
                &squeeze_final_force.ln(),
                &((q + 1) as f64 / squeeze_rampup_iters as f64),
            )
            .exp();
            params.rate = squeeze_rampup_rate;
            state.step(&mut params);
        }
    }
    params.squeeze_force = params.squeeze_force2;
    params.rate = squeeze_final_initial_rate;
    for q in 0..squeeze_final_iters {
        inc_iter_and_maybe_save(state.coords.view());
        state.step(&mut params);
        params.rate = lerp(
            &(squeeze_final_initial_rate * squeeze_final_initial_rate),
            &(final_rate * final_rate),
            &((q + 1) as f64 / squeeze_final_iters as f64),
        )
        .sqrt();
    }

    let f = opts.get_ostream()?;
    let f = opts.get_csv_writer().from_writer(f);
    save_csv(&header, n_out_coords, f, &records, coords.view())?;

    Ok(())
}

fn save_csv<'a>(
    header: &Option<csv::ByteRecord>,
    n_out_coords: usize,
    mut f: csv::Writer<impl Write>,
    records: &Vec<csv::ByteRecord>,
    coords: ndarray::ArrayView2<'a, f64>,
) -> Result<(), anyhow::Error> {
    if let Some(h) = &header {
        for i in 1..=n_out_coords {
            f.write_field(format!("coord{}", i))?;
        }
        f.write_record(h)?;
    }
    Ok(for (j, record) in records.iter().enumerate() {
        for i in 0..n_out_coords {
            f.write_field(format!("{:.4}", coords[(j, i)]))?;
        }
        f.write_record(record)?;
    })
}
