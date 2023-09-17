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
    pub weights: Ar1Ref<'a>,
    pub affinities: Ar2Ref<'a>,
}
pub struct Params<'a> {
    pub rate: f64,
    pub central_force: f64,
    /// Dimension is like in coordinates
    pub tmp: Ar1Mut<'a>,
    pub squeeze_from: usize,
    // Like `central_force`, but applies starting from `squeeze_from` coordinate number
    pub squeeze_force: f64,
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
        
        self.forces.fill(0.0);
        let mut vector = &mut params.tmp;

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

                    if sqnorm < 0.0001 {
                        sqnorm = 0.0001;
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
                if c >= squeeze_from {
                    cc = squeeze_force*cc;
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
        maxforcecoord = maxforcecoord.max(0.0001);
        
        /// Force some coordinate change to be `rate` regardless of forces scale
        let scale = params.rate / maxforcecoord;

        self.coords.scaled_add(scale, &self.forces);

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
