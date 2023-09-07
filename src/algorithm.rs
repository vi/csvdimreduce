#![allow(unused)]

use std::ops::Range;

use ndarray::{Axis, s};

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
        
        self.forces.fill(0.0);
        let mut vector = &mut params.tmp;
        for j in 0..n {
            for p in 0..n {
                if j==p { continue; }
                vector.fill(0.0);
                let mut sqnorm = 0.0;
                for c in 0..cn {
                    vector[c] = self.coords[(j,c)] - self.coords[(p,c)];
                    sqnorm += vector[c] * vector[c];
                }
                if sqnorm < 0.0001 {
                    sqnorm = 0.0001;
                }
                let norm = sqnorm.sqrt();
                for x in vector.iter_mut() {
                    *x /= norm;
                }
                let repelling_force = self.affinities[(j,p)]/sqnorm;
                //println!("{repelling_force} {vector}");
                //print!("{repelling_force} ");

                self.forces.slice_mut(s![j, ..]).scaled_add(repelling_force, vector);
            }
            for c in 0..cn {
                let mut cc = self.coords[(j, c)];
                cc = cc - 0.5;
                if c >= params.squeeze_from {
                    cc = params.squeeze_force*cc;
                } else {
                    cc = params.central_force*cc;

                }
                self.forces[(j, c)] -= (n as f64) * cc;
            } 
        }
        let mut maxforcecoord = 0.0;
        for j in 0..n {
            for c in 0..cn {
                maxforcecoord = self.forces[(j,c)].abs().max(maxforcecoord);
            }
        }
        maxforcecoord = maxforcecoord.max(0.0001);
        
        /// Force some coordinate change to be `rate` regardless of forces scale
        let scale = params.rate / maxforcecoord;

        self.coords.scaled_add(scale, &self.forces);

        for j in 0..n {
            for c in 0..cn {
                let cc = self.coords[(j, c)];
                if cc <= -0.0 {
                    self.coords[(j,c)] = 0.0;
                }
                if cc >= 1.0 {
                    self.coords[(j,c)] = 1.0;
                }
            }
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
