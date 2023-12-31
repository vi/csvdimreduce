#![allow(unused)]

use std::ops::Range;

use ndarray::{Axis, s, azip};

pub type Ar2Mut<'a> = ndarray::ArrayViewMut2<'a, f64>;
pub type Ar2Ref<'a> = ndarray::ArrayView2<'a, f64>;
pub type Ar1Mut<'a> = ndarray::ArrayViewMut1<'a, f64>;
pub type Ar1Ref<'a> = ndarray::ArrayView1<'a, f64>;

pub struct State<'a> {
    /// First dimension - particle index, Second dimension - coordinate
    pub coords: Ar2Mut<'a>,
    /// First dimension - particle index, Second dimension - coordinate
    pub forces: Ar2Mut<'a>,
    /// Dimensions the same as above.
    pub inertias: Ar2Mut<'a>,
    pub weights: Ar1Ref<'a>,
    pub affinities: Ar2Ref<'a>,
    /// Dimension is like in coordinates
    pub tmp: Ar1Mut<'a>,
    pub movement_scaler : f64,
}
pub struct Params {
    pub rate: f64,
    pub central_force: f64,
    pub squeeze_from: usize,
    /// Like `central_force`, but applies to `squeeze_from` coordinate number.
    pub squeeze_force: f64,
    /// Like `squeeze_force`, but for coordinates above `squeeze_from`.
    pub squeeze_force2: f64,
    pub inertia_multiplier: f64,
    pub debug: bool,
}

impl<'a> State<'a> {
    pub fn step(&mut self, params: &mut Params) {
        let n = self.coords.len_of(Axis(0));
        let cn = self.coords.len_of(Axis(1));
        assert_eq!(self.coords.dim(), self.forces.dim());
        assert_eq!(n, self.weights.len_of(Axis(0)));
        assert_eq!(cn, self.coords.len_of(Axis(1)));
        assert_eq!(self.affinities.dim(), (n,n));

        let central_force = params.central_force;
        let squeeze_from = params.squeeze_from;
        let squeeze_force = params.squeeze_force;
        let squeeze_force2 = params.squeeze_force2;
        
        self.forces.fill(0.0);
        let mut vector = &mut self.tmp;

        let coords = self.coords.view();
        let mut forces = self.forces.view_mut();
        let affinities = self.affinities.view();
        let weights = self.weights.view();
        for j in 0..n {
            let my_coords = coords.slice(s![j, ..]);
            let my_weight = weights[j];
            let affinities_shard = affinities.slice(s![j, ..]);
            let mut my_forces = forces.slice_mut(s![j, ..]);
            azip!((
                index (p),
                their_coords in coords.rows(),
                affinity in affinities_shard,
                their_weight in weights,
            ) {
                if j != p {
                    vector.fill(0.0);
                    let mut sqnorm = 0.0;
                    azip!((vc in vector.view_mut(), myc in my_coords, theirc in their_coords) {
                        *vc = myc - theirc;
                        sqnorm += *vc * *vc;
                    });

                    if sqnorm < 0.00001 {
                        sqnorm = 0.00001;
                    }
                    let norm = sqnorm.sqrt();
                    for x in vector.iter_mut() {
                        *x /= norm;
                    }
                    let repelling_force = affinity/sqnorm*their_weight/my_weight;
                    my_forces.scaled_add(repelling_force, vector);
                }
            });
            azip!((
                index (c),
                cc in my_coords,
                ff in &mut my_forces,
            ) {
                let mut cc = *cc;
                cc = cc - 0.5;
                if c == squeeze_from {
                    cc = squeeze_force*cc;
                } else if c > squeeze_from {
                    cc = squeeze_force2*cc;
                } else {
                    cc = central_force*cc;

                }
                *ff -= (n as f64) * cc;
            });
        }
        let mut maxforcecoord = 0.0;
        for f in forces {
            maxforcecoord = f.abs().max(maxforcecoord);
        }
        if params.debug { println!("movement {maxforcecoord}"); }
        maxforcecoord = maxforcecoord.max(0.0001);
        
        self.movement_scaler = self.movement_scaler * 0.8 + maxforcecoord * 0.2;

        /// Force some coordinate change to be `rate` regardless of forces scale
        let scale = params.rate / self.movement_scaler;

        self.inertias.scaled_add(scale, &self.forces);
        self.coords.scaled_add(1.0, &self.inertias);
        self.inertias.map_inplace(|x|*x *= params.inertia_multiplier);

        for cc in self.coords.iter_mut() {
            *cc = cc.clamp(0.0, 1.0);
        }
    }
}

pub fn build_particle_affinities<'a,'b>(input: Ar2Ref<'a>, mut output:Ar2Mut<'b>, same_particle_force: f64) {
    let n = input.len_of(Axis(0));
    let cn = input.len_of(Axis(1));
    assert_eq!(output.dim(), (n,n));
    output.fill(same_particle_force);
    for j in 0..n {
        for k in 0..n {
            if n == k {
                continue;
            }
            for c in 0..cn {
                output[(j,k)] += (input[(j,c)] - input[(k,c)]).abs();
            }
        }
    }
}

pub fn average_affinity<'a>(matrix: Ar2Ref<'a>) -> f64 {
    matrix.sum() / matrix.len() as f64
}

pub fn normalize<'a>(mut inputvals: Ar2Mut<'a>) {
    let n_input_coords = inputvals.len_of(Axis(1));
    for j in 0..n_input_coords {
        let mut s = inputvals.slice_mut(s![.., j]);
        let avg = s.sum() / s.len() as f64;
        s -= avg;
        let scale : f64 = s.dot(&s).sqrt();
        s /= scale;
    }
}
