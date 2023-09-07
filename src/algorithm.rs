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
}
pub struct Params<'a> {
    pub coordinate_range_to_consider : Range<usize>,
    pub coordinate_range_to_clamp : Range<usize>,
    pub rate: f64,
    pub central_force: f64,
    /// Dimension is like in coordinates
    pub tmp: Ar1Mut<'a>,
}

impl<'a> State<'a> {
    pub fn step(&mut self, params: &mut Params) {
        assert_eq!(self.coords.dim(), self.forces.dim());
        assert_eq!(self.coords.len_of(Axis(0)), self.weights.len_of(Axis(0)));
        assert_eq!(params.tmp.len(), params.coordinate_range_to_consider.len());
        self.forces.fill(0.0);
        let n = self.coords.len_of(Axis(0));
        let mut vector = &mut params.tmp;
        for j in 0..n {
            for p in 0..n {
                if j==p { continue; }
                vector.fill(0.0);
                let mut sqnorm = 0.0;
                for c in params.coordinate_range_to_consider.clone() {
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
                let repelling_force = 1.0/sqnorm;
                //println!("{repelling_force} {vector}");
                //print!("{repelling_force} ");

                self.forces.slice_mut(s![j, params.coordinate_range_to_consider.clone()]).scaled_add(repelling_force, vector);
            }
            for c in params.coordinate_range_to_clamp.clone() {
                let mut cc = self.coords[(j, c)];
                cc = params.central_force*(cc - 0.5);
                self.forces[(j, c)] -= (n as f64) * cc;
            } 
        }
        let mut maxforcecoord = 0.0;
        for j in 0..n {
            for c in params.coordinate_range_to_consider.clone() {
                maxforcecoord = self.forces[(j,c)].abs().max(maxforcecoord);
            }
        }
        maxforcecoord = maxforcecoord.max(0.0001);
        
        /// Force some coordinate change to be `rate` regardless of forces scale
        let scale = params.rate / maxforcecoord;

        self.coords.scaled_add(scale, &self.forces);

        for j in 0..n {
            for c in params.coordinate_range_to_clamp.clone() {
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
