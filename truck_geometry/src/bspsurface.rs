use crate::errors::Error;
use crate::matrix::MatrixEntity;
use crate::tolerance::inv_or_zero;
use crate::*;

impl<T: EntityArray<f64>> BSplineSurface<T> {
    /// constructor.
    /// # Arguments
    /// * `knot_vecs` - the knot vectors
    /// * `control_points` - the vector of the control points
    /// # Panics
    /// There are 3 rules for construct B-spline curve.
    /// * The number of knots is more than the one of control points.
    /// * There exist at least two different knots.
    /// * There are at least one control point.
    #[inline(always)]
    pub fn new(
        knot_vecs: (KnotVec, KnotVec),
        control_points: Vec<Vec<Vector<T>>>,
    ) -> BSplineSurface<T>
    {
        BSplineSurface::try_new(knot_vecs, control_points).unwrap_or_else(|e| panic!("{}", e))
    }

    /// constructor.
    /// # Arguments
    /// * `knot_vecs` - the knot vectors
    /// * `control_points` - the vector of the control points
    /// # Failures
    /// There are 3 rules for construct B-spline curve.
    /// * The number of knots is more than the one of control points.
    /// * There exist at least two different knots.
    /// * There are at least one control point.
    #[inline(always)]
    pub fn try_new(
        knot_vecs: (KnotVec, KnotVec),
        control_points: Vec<Vec<Vector<T>>>,
    ) -> Result<BSplineSurface<T>>
    {
        if control_points.is_empty() {
            Err(Error::EmptyControlPoints)
        } else if control_points[0].is_empty() {
            Err(Error::EmptyControlPoints)
        } else if knot_vecs.0.len() <= control_points.len() {
            Err(Error::TooShortKnotVector(
                knot_vecs.0.len(),
                control_points.len(),
            ))
        } else if knot_vecs.1.len() <= control_points[0].len() {
            Err(Error::TooShortKnotVector(
                knot_vecs.1.len(),
                control_points[0].len(),
            ))
        } else if knot_vecs.0.range_length().so_small() || knot_vecs.1.range_length().so_small() {
            Err(Error::ZeroRange)
        } else {
            let len = control_points[0].len();
            if control_points
                .iter()
                .fold(false, |flag, vec| flag || vec.len() != len)
            {
                Err(Error::IrregularControlPoints)
            } else {
                Ok(BSplineSurface::new_unchecked(knot_vecs, control_points))
            }
        }
    }

    /// constructor.
    /// # Arguments
    /// * `knot_vecs` - the knot vectors
    /// * `control_points` - the vector of the control points
    /// # Failures
    /// This method is prepared only for performance-critical development and is not recommended.  
    /// This method does NOT check the 3 rules for constructing B-spline surface.  
    /// The programmer must guarantee these conditions before using this method.  
    #[inline(always)]
    pub fn new_unchecked(
        knot_vecs: (KnotVec, KnotVec),
        control_points: Vec<Vec<Vector<T>>>,
    ) -> BSplineSurface<T>
    {
        BSplineSurface {
            knot_vecs,
            control_points,
        }
    }

    /// Returns the reference of the knot vectors
    #[inline(always)]
    pub fn knot_vecs(&self) -> &(KnotVec, KnotVec) { &self.knot_vecs }

    /// Returns the u knot vector.
    #[inline(always)]
    pub fn uknot_vec(&self) -> &KnotVec { &self.knot_vecs.0 }
    /// Returns the v knot vector.
    #[inline(always)]
    pub fn vknot_vec(&self) -> &KnotVec { &self.knot_vecs.1 }

    /// Returns the `idx`th u knot.
    #[inline(always)]
    pub fn uknot(&self, idx: usize) -> f64 { self.knot_vecs.0[idx] }
    /// returns the `idx`th v knot.
    #[inline(always)]
    pub fn vknot(&self, idx: usize) -> f64 { self.knot_vecs.1[idx] }

    /// Returns the reference of the vector of the control points
    #[inline(always)]
    pub fn control_points(&self) -> &Vec<Vec<Vector<T>>> { &self.control_points }

    /// Returns the reference of the control point corresponding to the index `(idx0, idx1)`.
    #[inline(always)]
    pub fn control_point(&self, idx0: usize, idx1: usize) -> &Vector<T> {
        &self.control_points[idx0][idx1]
    }

    /// Returns the iterator over the control points in the `column_idx`th row.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::bezier_knot(1);
    /// let vknot_vec = KnotVec::bezier_knot(2);
    /// let knot_vecs = (uknot_vec, vknot_vec);
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0, 0), vector!(1, 0, 1), vector!(2, 0, 2)],
    ///     vec![vector!(0, 1, 0), vector!(1, 1, 1), vector!(2, 1, 2)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let mut iter = bspsurface.ctrl_pts_row_iter(1);
    /// assert_eq!(iter.next(), Some(&vector!(1, 0, 1)));
    /// assert_eq!(iter.next(), Some(&vector!(1, 1, 1)));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline(always)]
    pub fn ctrl_pts_row_iter(&self, column_idx: usize) -> CPRowIter<'_, T> {
        CPRowIter {
            iter: self.control_points.iter(),
            idx: column_idx,
        }
    }

    /// Returns the iterator over the control points in the `row_idx`th row.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::bezier_knot(1);
    /// let vknot_vec = KnotVec::bezier_knot(2);
    /// let knot_vecs = (uknot_vec, vknot_vec);
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0, 0), vector!(1, 0, 1), vector!(2, 0, 2)],
    ///     vec![vector!(0, 1, 0), vector!(1, 1, 1), vector!(2, 1, 2)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let mut iter = bspsurface.ctrl_pts_column_iter(1);
    /// assert_eq!(iter.next(), Some(&vector!(0, 1, 0)));
    /// assert_eq!(iter.next(), Some(&vector!(1, 1, 1)));
    /// assert_eq!(iter.next(), Some(&vector!(2, 1, 2)));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline(always)]
    pub fn ctrl_pts_column_iter(&self, row_idx: usize) -> CPColumnIter<'_, T> {
        self.control_points[row_idx].iter()
    }

    /// Returns the mutable reference of the control point corresponding to index `(idx0, idx1)`.
    #[inline(always)]
    pub fn control_point_mut(&mut self, idx0: usize, idx1: usize) -> &mut Vector<T> {
        &mut self.control_points[idx0][idx1]
    }

    /// Returns the degrees of B-spline surface
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let vknot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vecs = (uknot_vec, vknot_vec);
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0, 0), vector!(1, 0, 1), vector!(2, 0, 2)],
    ///     vec![vector!(0, 1, 0), vector!(1, 1, 1), vector!(2, 1, 2)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// assert_eq!(bspsurface.udegree(), 1);
    /// ```
    #[inline(always)]
    pub fn udegree(&self) -> usize { self.knot_vecs.0.len() - self.control_points.len() - 1 }

    /// Returns the degrees of B-spline surface
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let vknot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vecs = (uknot_vec, vknot_vec);
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0, 0), vector!(1, 0, 1), vector!(2, 0, 2)],
    ///     vec![vector!(0, 1, 0), vector!(1, 1, 1), vector!(2, 1, 2)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// assert_eq!(bspsurface.vdegree(), 2);
    /// ```
    #[inline(always)]
    pub fn vdegree(&self) -> usize { self.knot_vecs.1.len() - self.control_points[0].len() - 1 }

    /// Returns the degrees of B-spline surface
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let vknot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vecs = (uknot_vec, vknot_vec);
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0, 0), vector!(1, 0, 1), vector!(2, 0, 2)],
    ///     vec![vector!(0, 1, 0), vector!(1, 1, 1), vector!(2, 1, 2)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// assert_eq!(bspsurface.degrees(), (1, 2));
    /// ```
    #[inline(always)]
    pub fn degrees(&self) -> (usize, usize) { (self.udegree(), self.vdegree()) }

    /// Returns whether all control points are same or not.
    /// If the knot vector is clamped, it means whether the curve is constant or not.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::bezier_knot(1);
    /// let vknot_vec = KnotVec::bezier_knot(2);
    /// let pt = vector!(1, 2);
    /// let ctrl_pts = vec![
    ///     vec![pt.clone(), pt.clone(), pt.clone()],
    ///     vec![pt.clone(), pt.clone(), pt.clone()],
    /// ];
    /// let mut bspsurface = BSplineSurface::new((uknot_vec, vknot_vec), ctrl_pts);
    /// assert!(bspsurface.is_const());
    ///
    /// *bspsurface.control_point_mut(1, 2) = vector!(2, 3);
    /// assert!(!bspsurface.is_const());
    /// ```
    /// # Remarks
    /// If the knot vector is not clamped and the BSpline basis function is not partition of unity,
    /// then perhaps returns true even if the surface is not constant.
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::uniform_knot(1, 5);
    /// let vknot_vec = KnotVec::uniform_knot(1, 5);
    /// let pt = vector!(1, 2);
    /// let ctrl_pts = vec![
    ///     vec![pt.clone(), pt.clone(), pt.clone()],
    ///     vec![pt.clone(), pt.clone(), pt.clone()],
    /// ];
    /// let mut bspsurface = BSplineSurface::new((uknot_vec, vknot_vec), ctrl_pts);
    ///
    /// // bspsurface is not constant.
    /// assert_eq!(bspsurface.subs(0.0, 0.0), vector!(0.0, 0.0));
    /// assert_ne!(bspsurface.subs(0.5, 0.5), vector!(0.0, 0.0));
    ///
    /// // bspsurface.is_const() is true.
    /// assert!(bspsurface.is_const());
    /// ```
    #[inline(always)]
    pub fn is_const(&self) -> bool {
        for vec in self.control_points.iter().flat_map(|pts| pts.iter()) {
            if !vec.near(&self.control_points[0][0]) {
                return false;
            }
        }
        true
    }

    /// Returns whether constant curve or not, i.e. all control points are same or not.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::bezier_knot(1);
    /// let vknot_vec = KnotVec::bezier_knot(2);
    /// let pt = rvector!(1, 2);
    /// // allows differences upto scalars
    /// let ctrl_pts = vec![
    ///     vec![pt.clone(), pt.clone() * 2.0, pt.clone() * 3.0],
    ///     vec![pt.clone() * 0.5, pt.clone() * 0.25, pt.clone() * 0.125],
    /// ];
    /// let mut bspsurface = BSplineSurface::new((uknot_vec, vknot_vec), ctrl_pts);
    /// assert!(bspsurface.is_rational_const());
    ///
    /// *bspsurface.control_point_mut(1, 2) = rvector!(2, 3);
    /// assert!(!bspsurface.is_rational_const());
    /// ```
    #[inline(always)]
    pub fn is_rational_const(&self) -> bool {
        let pt = self.control_points[0][0].rational_projection();
        for vec in self.control_points.iter().flat_map(|pts| pts.iter()) {
            if !vec.rational_projection().near(&pt) {
                return false;
            }
        }
        true
    }

    /// Substitutes to a B-spline surface.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0), vector!(0.5, -1), vector!(1, 0)],
    ///     vec![vector!(0, 1), vector!(0.5, 2), vector!(1, 1)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    ///
    /// // bspsurface: (v, 2v(1 - v)(2u - 1) + u)
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let u = (i as f64) / (N as f64);
    ///     for j in 0..=N {
    ///         let v = (j as f64) / (N as f64);
    ///         Vector::assert_near2(
    ///             &bspsurface.subs(u, v),
    ///             &vector!(v, 2.0 * v * (1.0 - v) * (2.0 * u - 1.0) + u),
    ///         );
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn subs(&self, u: f64, v: f64) -> Vector<T> {
        let (degree0, degree1) = self.degrees();
        let (uknot_vec, vknot_vec) = &self.knot_vecs;
        let basis0 = uknot_vec.bspline_basis_functions(degree0, u);
        let basis1 = vknot_vec.bspline_basis_functions(degree1, v);
        let mut res = Vector::zero();
        self.control_points
            .iter()
            .zip(&basis0)
            .for_each(|(vec, b0)| {
                vec.iter()
                    .zip(&basis1)
                    .for_each(|(pt, b1)| res += pt * (b0 * b1))
            });
        res
    }

    /// Returns the closure of substitution.
    #[inline(always)]
    pub fn get_closure(&self) -> impl Fn(f64, f64) -> Vector<T> + '_ { move |u, v| self.subs(u, v) }

    #[inline(always)]
    fn udelta_control_points(&self, i: usize, j: usize) -> Vector<T> {
        if i == 0 {
            self.control_point(i, j).clone()
        } else if i == self.control_points.len() {
            -self.control_point(i - 1, j)
        } else {
            self.control_point(i, j) - self.control_point(i - 1, j)
        }
    }

    #[inline(always)]
    fn vdelta_control_points(&self, i: usize, j: usize) -> Vector<T> {
        if j == 0 {
            self.control_point(i, j).clone()
        } else if j == self.control_points[0].len() {
            -self.control_point(i, j - 1)
        } else {
            self.control_point(i, j) - self.control_point(i, j - 1)
        }
    }

    /// Calculate derived B-spline surface by the first parameter `u`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0), vector!(0.5, -1), vector!(1, 0)],
    ///     vec![vector!(0, 1), vector!(0.5, 2), vector!(1, 1)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let uderivation = bspsurface.uderivation();
    ///
    /// // bspsurface: (v, 2v(1 - v)(2u - 1) + u), uderivation: (0.0, 4v(1 - v) + 1)
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let u = (i as f64) / (N as f64);
    ///     for j in 0..=N {
    ///         let v = (j as f64) / (N as f64);
    ///         Vector::assert_near2(
    ///             &uderivation.subs(u, v),
    ///             &vector!(0.0, 4.0 * v * (1.0 - v) + 1.0),
    ///         );
    ///     }
    /// }
    /// ```
    pub fn uderivation(&self) -> BSplineSurface<T> {
        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();
        let (k, _) = self.degrees();
        let (uknot_vec, vknot_vec) = self.knot_vecs.clone();

        let new_points = if k > 0 {
            (0..=n0)
                .map(|i| {
                    let delta = uknot_vec[i + k] - uknot_vec[i];
                    let coef = (k as f64) * inv_or_zero(delta);
                    (0..n1)
                        .map(|j| coef * self.udelta_control_points(i, j))
                        .collect()
                })
                .collect()
        } else {
            vec![vec![Vector::zero(); n1]; n0]
        };

        BSplineSurface::new_unchecked((uknot_vec, vknot_vec), new_points)
    }

    /// Calculate derived B-spline surface by the second parameter `v`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0), vector!(0.5, -1), vector!(1, 0)],
    ///     vec![vector!(0, 1), vector!(0.5, 2), vector!(1, 1)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let vderivation = bspsurface.vderivation();
    ///
    /// // bspsurface: (v, 2v(1 - v)(2u - 1) + u), vderivation: (1, -2(2u - 1)(2v - 1))
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let u = (i as f64) / (N as f64);
    ///     for j in 0..=N {
    ///         let v = (j as f64) / (N as f64);
    ///         Vector::assert_near2(
    ///             &vderivation.subs(u, v),
    ///             &vector!(1.0, -2.0 * (2.0 * u - 1.0) * (2.0 * v - 1.0)),
    ///         );
    ///     }
    /// }
    /// ```
    pub fn vderivation(&self) -> BSplineSurface<T> {
        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();
        let (_, k) = self.degrees();

        let (uknot_vec, vknot_vec) = self.knot_vecs.clone();

        let new_points = if k > 0 {
            let mut new_points = vec![Vec::with_capacity(n1 + 1); n0];
            for j in 0..=n1 {
                let delta = vknot_vec[j + k] - vknot_vec[j];
                let coef = (k as f64) * inv_or_zero(delta);
                for (i, vec) in new_points.iter_mut().enumerate() {
                    vec.push(coef * self.vdelta_control_points(i, j))
                }
            }
            new_points
        } else {
            vec![vec![Vector::zero(); n1]; n0]
        };

        BSplineSurface::new_unchecked((uknot_vec, vknot_vec), new_points)
    }

    /// Swaps two parameters.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs0 = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts0 = vec![
    ///     vec![vector!(0, 0), vector!(0.5, -1), vector!(1, 0)],
    ///     vec![vector!(0, 1), vector!(0.5, 2), vector!(1, 1)],
    /// ];
    /// let mut bspsurface0 = BSplineSurface::new(knot_vecs0, ctrl_pts0);
    ///
    /// let knot_vecs1 = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(1));
    /// let ctrl_pts1 = vec![
    ///     vec![vector!(0, 0), vector!(0, 1)],
    ///     vec![vector!(0.5, -1), vector!(0.5, 2)],
    ///     vec![vector!(1, 0), vector!(1, 1)],
    /// ];
    /// let mut bspsurface1 = BSplineSurface::new(knot_vecs1, ctrl_pts1);
    /// assert_eq!(bspsurface0.swap_axes(), &bspsurface1);
    /// ```
    pub fn swap_axes(&mut self) -> &mut Self {
        let knot_vec = self.knot_vecs.0.clone();
        self.knot_vecs.0 = self.knot_vecs.1.clone();
        self.knot_vecs.1 = knot_vec;

        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();
        let mut new_points = vec![Vec::with_capacity(n0); n1];
        for pts in &self.control_points {
            for (vec0, pt) in new_points.iter_mut().zip(pts) {
                vec0.push(pt.clone());
            }
        }
        self.control_points = new_points;
        self
    }

    /// Adds a knot `x` of the first parameter `u`, and do not change `self` as a surface.  
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0), vector!(0.5, -1), vector!(1, 0)],
    ///     vec![vector!(0, 1), vector!(0.5, 2), vector!(1, 1)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    /// bspsurface.add_uknot(0.0).add_uknot(0.3).add_uknot(0.5).add_uknot(1.0);
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// assert_eq!(bspsurface.uknot_vec().len(), org_surface.uknot_vec().len() + 4);
    /// ```
    pub fn add_uknot(&mut self, x: f64) -> &mut Self {
        let k = self.udegree();
        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();
        if x < self.knot_vecs.0[0] {
            self.knot_vecs.0.add_knot(x);
            self.control_points.insert(0, vec![Vector::zero(); n1]);
            return self;
        }

        let idx = self.knot_vecs.0.add_knot(x);
        let start = if idx > k { idx - k } else { 0 };
        let end = if idx > n0 {
            self.control_points.push(vec![Vector::zero(); n1]);
            n0 + 1
        } else {
            self.control_points
                .insert(idx - 1, self.control_points[idx - 1].clone());
            idx
        };
        for i in start..end {
            let i0 = end + start - i - 1;
            let delta = self.knot_vecs.0[i0 + k + 1] - self.knot_vecs.0[i0];
            let a = inv_or_zero(delta) * (self.knot_vecs.0[idx] - self.knot_vecs.0[i0]);
            for j in 0..n1 {
                let p = (1.0 - a) * self.udelta_control_points(i0, j);
                self.control_points[i0][j] -= p;
            }
        }
        self
    }

    /// add a knot `x` for the second parameter, and do not change `self` as a surface.  
    /// Return `false` if cannot add the knot, i.e.
    /// * the index of `x` will be lower than the degree, or
    /// * the index of `x` will be higher than the number of control points.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0), vector!(0.5, -1), vector!(1, 0)],
    ///     vec![vector!(0, 1), vector!(0.5, 2), vector!(1, 1)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    /// bspsurface.add_vknot(0.0).add_vknot(0.3).add_vknot(0.5).add_vknot(1.0);
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// assert_eq!(bspsurface.vknot_vec().len(), org_surface.vknot_vec().len() + 4);
    /// ```
    pub fn add_vknot(&mut self, x: f64) -> &mut Self {
        if x < self.knot_vecs.1[0] {
            self.knot_vecs.1.add_knot(x);
            self.control_points
                .iter_mut()
                .for_each(|vec| vec.insert(0, Vector::zero()));
            return self;
        }

        let k = self.vdegree();
        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();

        let idx = self.knot_vecs.1.add_knot(x);
        let start = if idx > k { idx - k } else { 0 };
        let end = if idx > n1 {
            self.control_points
                .iter_mut()
                .for_each(|vec| vec.push(Vector::zero()));
            n1 + 1
        } else {
            self.control_points
                .iter_mut()
                .for_each(|vec| vec.insert(idx - 1, vec[idx - 1].clone()));
            idx
        };
        for j in start..end {
            let j0 = end + start - j - 1;
            let delta = self.knot_vecs.1[j0 + k + 1] - self.knot_vecs.1[j0];
            let a = inv_or_zero(delta) * (self.knot_vecs.1[idx] - self.knot_vecs.1[j0]);
            for i in 0..n0 {
                let p = (1.0 - a) * self.vdelta_control_points(i, j0);
                self.control_points[i][j0] -= p;
            }
        }
        self
    }

    /// Removes a uknot corresponding to the indice `idx`, and do not change `self` as a curve.  
    /// If the knot cannot be removed, returns
    /// [`Error::CannotRemoveKnot`](./errors/enum.Error.html#variant.CannotRemoveKnot).
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0), vector!(0.5, -1), vector!(1, 0)],
    ///     vec![vector!(0, 1), vector!(0.5, 2), vector!(1, 1)],
    ///     vec![vector!(0, 2), vector!(0.5, -1), vector!(1, 2)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    ///
    /// bspsurface.add_uknot(0.3).add_uknot(0.5);
    ///
    /// assert!(bspsurface.try_remove_uknot(3).is_ok());
    /// assert_eq!(bspsurface.try_remove_uknot(2), Err(Error::CannotRemoveKnot(2)));
    ///
    /// assert_eq!(bspsurface.uknot_vec().len(), org_surface.uknot_vec().len() + 1);
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// ```
    pub fn try_remove_uknot(&mut self, idx: usize) -> Result<&mut Self> {
        let k = self.udegree();
        let knot_vec = self.uknot_vec();
        let n = self.control_points.len();

        if idx < k + 1 || idx >= n {
            return Err(Error::CannotRemoveKnot(idx));
        }

        let mut new_points = Vec::with_capacity(k + 1);
        let first_vec = self
            .ctrl_pts_column_iter(idx - k - 1)
            .map(|pt| pt.clone())
            .collect::<Vec<_>>();
        new_points.push(first_vec);
        for i in (idx - k)..idx {
            let delta = knot_vec[i + k + 1] - knot_vec[i];
            let a = inv_or_zero(delta) * (knot_vec[idx] - knot_vec[i]);
            if a.so_small() {
                break;
            } else {
                let vec = self
                    .ctrl_pts_column_iter(i)
                    .zip(new_points.last().unwrap())
                    .map(|(pt0, pt1)| pt0 / a - pt1 * (1.0 - a) / a)
                    .collect();
                new_points.push(vec);
            }
        }

        for (pt0, pt1) in self
            .ctrl_pts_column_iter(idx)
            .zip(new_points.last().unwrap())
        {
            if !pt0.near(pt1) {
                return Err(Error::CannotRemoveKnot(idx));
            }
        }

        for (i, vec) in new_points.into_iter().skip(1).enumerate() {
            self.control_points[idx - k + i] = vec;
        }

        self.control_points.remove(idx);
        self.knot_vecs.0.remove(idx);
        Ok(self)
    }

    /// Removes a uknot corresponding to the indices `idx`, and do not change `self` as a curve.
    /// If cannot remove the knot, do not change `self` and return `self`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0), vector!(0.5, -1), vector!(1, 0)],
    ///     vec![vector!(0, 1), vector!(0.5, 2), vector!(1, 1)],
    ///     vec![vector!(0, 2), vector!(0.5, -1), vector!(1, 2)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    ///
    /// bspsurface.add_uknot(0.3).add_uknot(0.5);
    /// bspsurface.remove_uknot(3).remove_uknot(3);
    ///
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// assert_eq!(bspsurface.uknot_vec().len(), org_surface.uknot_vec().len())
    /// ```
    #[inline(always)]
    pub fn remove_uknot(&mut self, idx: usize) -> &mut Self {
        let _ = self.try_remove_uknot(idx);
        self
    }

    /// Removes a vknot corresponding to the indice `idx`, and do not change `self` as a curve.  
    /// If the knot cannot be removed, returns
    /// [`Error::CannotRemoveKnot`](./errors/enum.Error.html#variant.CannotRemoveKnot).
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0), vector!(0.5, -1), vector!(1, 0)],
    ///     vec![vector!(0, 1), vector!(0.5, 2), vector!(1, 1)],
    ///     vec![vector!(0, 2), vector!(0.5, -1), vector!(1, 2)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    ///
    /// bspsurface.add_vknot(0.3).add_vknot(0.5);
    /// assert!(bspsurface.try_remove_vknot(3).is_ok());
    /// assert_eq!(bspsurface.try_remove_vknot(2), Err(Error::CannotRemoveKnot(2)));
    ///
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// assert_eq!(bspsurface.vknot_vec().len(), org_surface.vknot_vec().len() + 1);
    /// ```
    pub fn try_remove_vknot(&mut self, idx: usize) -> Result<&mut Self> {
        let (_, k) = self.degrees();
        let knot_vec = self.vknot_vec();
        let n = self.control_points[0].len();

        if idx < k + 1 || idx >= n {
            return Err(Error::CannotRemoveKnot(idx));
        }

        let mut new_points = Vec::with_capacity(k + 1);
        let first_vec = self
            .ctrl_pts_row_iter(idx - k - 1)
            .map(|pt| pt.clone())
            .collect::<Vec<_>>();
        new_points.push(first_vec);
        for i in (idx - k)..idx {
            let delta = knot_vec[i + k + 1] - knot_vec[i];
            let a = inv_or_zero(delta) * (knot_vec[idx] - knot_vec[i]);
            if a.so_small() {
                break;
            } else {
                let vec = self
                    .ctrl_pts_row_iter(i)
                    .zip(new_points.last().unwrap())
                    .map(|(pt0, pt1)| pt0 / a - pt1 * (1.0 - a) / a)
                    .collect();
                new_points.push(vec);
            }
        }

        for (pt0, pt1) in self
            .ctrl_pts_row_iter(idx)
            .zip(new_points.last().unwrap())
        {
            if !pt0.near(pt1) {
                return Err(Error::CannotRemoveKnot(idx));
            }
        }

        for (i, vec) in new_points.into_iter().skip(1).enumerate() {
            for (j, pt) in vec.into_iter().enumerate() {
                self.control_points[j][idx - k + i] = pt;
            }
        }

        for vec in &mut self.control_points {
            vec.remove(idx);
        }
        self.knot_vecs.1.remove(idx);
        Ok(self)
    }

    /// Removes a uknot corresponding to the indices `idx`, and do not change `self` as a curve.
    /// If cannot remove the knot, do not change `self` and return `self`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![vector!(0, 0), vector!(0.5, -1), vector!(1, 0)],
    ///     vec![vector!(0, 1), vector!(0.5, 2), vector!(1, 1)],
    ///     vec![vector!(0, 2), vector!(0.5, -1), vector!(1, 2)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    ///
    /// bspsurface.add_vknot(0.3).add_vknot(0.5);
    /// bspsurface.remove_vknot(3).remove_vknot(3);
    ///
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// assert_eq!(bspsurface.vknot_vec().len(), org_surface.vknot_vec().len())
    /// ```
    #[inline(always)]
    pub fn remove_vknot(&mut self, idx: usize) -> &mut Self {
        let _ = self.try_remove_vknot(idx);
        self
    }

    /// Creates a surface with normailized knot vectors connecting two curves.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec0 = KnotVec::bezier_knot(2);
    /// let ctrl_pts0 = vec![vector!(0, 0), vector!(0.5, -1), vector!(1, 0)];
    /// let bspcurve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    /// 
    /// let knot_vec1 = KnotVec::bezier_knot(2);
    /// let ctrl_pts1 = vec![vector!(0, 2), vector!(0.5, 1), vector!(1, 2)];
    /// let bspcurve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    /// 
    /// let homotopy_surface = BSplineSurface::homotopy(bspcurve0, bspcurve1);
    /// assert_eq!(
    ///     homotopy_surface.control_points(),
    ///     &vec![
    ///         vec![vector!(0, 0), vector!(0, 2)],
    ///         vec![vector!(0.5, -1), vector!(0.5, 1)],
    ///         vec![vector!(1, 0), vector!(1, 2)],
    ///     ],
    /// );
    /// ```
    pub fn homotopy(
        mut bspcurve0: BSplineCurve<T>,
        mut bspcurve1: BSplineCurve<T>,
    ) -> BSplineSurface<T>
    {
        bspcurve0.syncro_degree(&mut bspcurve1);

        bspcurve0.optimize();
        bspcurve1.optimize();

        bspcurve0.syncro_knot(&mut bspcurve1);

        let uknot_vec = bspcurve0.knot_vec().clone();
        let vknot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
        let mut control_points = Vec::new();
        for i in 0..bspcurve0.control_points().len() {
            control_points.push(Vec::new());
            control_points[i].push(bspcurve0.control_point(i).clone());
            control_points[i].push(bspcurve1.control_point(i).clone());
        }
        BSplineSurface::new_unchecked((uknot_vec, vknot_vec), control_points)
    }

    pub fn by_boundary(
        mut curve0: BSplineCurve<T>,
        mut curve1: BSplineCurve<T>,
        mut curve2: BSplineCurve<T>,
        mut curve3: BSplineCurve<T>,
    ) -> BSplineSurface<T>
    {
        curve2.invert();
        curve3.invert();
        curve0.syncro_degree(&mut curve2);
        curve0.optimize();
        curve2.optimize();
        curve0.syncro_knot(&mut curve2);
        curve1.syncro_degree(&mut curve3);
        curve1.optimize();
        curve3.optimize();
        curve1.syncro_knot(&mut curve3);

        let knot_vecs = (curve0.knot_vec().clone(), curve3.knot_vec().clone());
        let mut control_points = Vec::new();
        control_points.push(curve3.control_points().clone());
        let n = curve0.control_points().len();
        let m = curve3.control_points().len();
        for i in 1..(n - 1) {
            let u = (i as f64) / (n as f64);
            let pt0 = curve0.control_point(i) * u + curve2.control_point(i) * (1.0 - u);
            let mut new_row = Vec::new();
            new_row.push(curve0.control_point(i).clone());
            for j in 1..(m - 1) {
                let v = (j as f64) / (m as f64);
                let pt1 = curve3.control_point(j) * v + curve1.control_point(j) * (1.0 - v);
                new_row.push((&pt0 + pt1) / 2.0);
            }
            new_row.push(curve2.control_point(i).clone());
            control_points.push(new_row);
        }
        control_points.push(curve1.control_points().clone());
        BSplineSurface::new(knot_vecs, control_points)
    }

    /// Returns whether the knot vectors are clamped or not.
    #[inline(always)]
    pub fn is_clamped(&self) -> bool {
        self.knot_vecs.0.is_clamped(self.udegree()) && self.knot_vecs.1.is_clamped(self.vdegree())
    }

    /// Normalizes the knot vectors
    #[inline(always)]
    pub fn knot_normalize(&mut self) -> &mut Self {
        self.knot_vecs.0.try_normalize().unwrap();
        self.knot_vecs.1.try_normalize().unwrap();
        self
    }

    /// Translates the knot vector of the first parameter `u`.
    #[inline(always)]
    pub fn uknot_translate(&mut self, x: f64) -> &mut Self {
        self.knot_vecs.0.translate(x);
        self
    }
    /// Translates the knot vector of the second parameter `v`.
    #[inline(always)]
    pub fn vknot_translate(&mut self, x: f64) -> &mut Self {
        self.knot_vecs.1.translate(x);
        self
    }
    /// remove knots in order from the back
    pub fn optimize(&mut self) -> &mut Self {
        loop {
            let (n0, n1) = (self.knot_vecs.0.len(), self.knot_vecs.1.len());
            let mut flag = true;
            for i in 1..=n0 {
                flag = flag && self.try_remove_uknot(n0 - i).is_err();
            }
            for j in 1..=n1 {
                flag = flag && self.try_remove_vknot(n1 - j).is_err();
            }
            if flag {
                break;
            }
        }
        self
    }

    pub fn splitted_boundary(&self) -> [BSplineCurve<T>; 4] {
        let (uknot_vec, vknot_vec) = self.knot_vecs.clone();
        let control_points0 = self.control_points.iter().map(|x| x[0].clone()).collect();
        let control_points1 = self.control_points.last().unwrap().clone();
        let control_points2 = self
            .control_points
            .iter()
            .map(|x| x.last().unwrap().clone())
            .collect();
        let control_points3 = self.control_points[0].clone();
        [
            BSplineCurve::new_unchecked(uknot_vec.clone(), control_points0),
            BSplineCurve::new_unchecked(vknot_vec.clone(), control_points1),
            BSplineCurve::new_unchecked(uknot_vec.clone(), control_points2),
            BSplineCurve::new_unchecked(vknot_vec.clone(), control_points3),
        ]
    }

    /// extract boundary of surface
    pub fn boundary(&self) -> BSplineCurve<T> {
        let (uknot_vec, vknot_vec) = self.knot_vecs.clone();
        let (range0, range1) = (uknot_vec.range_length(), vknot_vec.range_length());
        let [mut bspline0, mut bspline1, mut bspline2, mut bspline3] = self.splitted_boundary();
        bspline0
            .concat(&mut bspline1.knot_translate(range0))
            .concat(&mut bspline2.invert().knot_translate(range0 + range1))
            .concat(&mut bspline3.invert().knot_translate(range0 * 2.0 + range1));
        bspline0
    }

    fn sub_near_as_surface<F: Fn(&Vector<T>, &Vector<T>) -> bool>(
        &self,
        other: &BSplineSurface<T>,
        div_coef: usize,
        ord: F,
    ) -> bool
    {
        if !self.knot_vecs.0[0].near(&other.knot_vecs.0[0])
            || !self
                .knot_vecs
                .0
                .range_length()
                .near(&other.knot_vecs.0.range_length())
        {
            return false;
        }
        if !self.knot_vecs.1[0].near(&other.knot_vecs.1[0])
            || !self
                .knot_vecs
                .1
                .range_length()
                .near(&other.knot_vecs.1.range_length())
        {
            return false;
        }

        let (self_degree0, self_degree1) = self.degrees();
        let (other_degree0, other_degree1) = other.degrees();
        let division0 = std::cmp::max(self_degree0, other_degree0) * div_coef;
        let division1 = std::cmp::max(self_degree1, other_degree1) * div_coef;

        for i0 in 1..self.knot_vecs.0.len() {
            let delta0 = self.knot_vecs.0[i0] - self.knot_vecs.0[i0 - 1];
            if delta0.so_small() {
                continue;
            }
            for j0 in 0..division0 {
                let u = self.knot_vecs.0[i0 - 1] + delta0 * (j0 as f64) / (division0 as f64);
                for i1 in 1..self.knot_vecs.1.len() {
                    let delta1 = self.knot_vecs.1[i1] - self.knot_vecs.1[i1 - 1];
                    if delta1.so_small() {
                        continue;
                    }
                    for j1 in 0..division1 {
                        let v =
                            self.knot_vecs.1[i1 - 1] + delta1 * (j1 as f64) / (division1 as f64);
                        if !ord(&self.subs(u, v), &other.subs(u, v)) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
    #[inline(always)]
    pub fn near_as_surface(&self, other: &BSplineSurface<T>) -> bool {
        self.sub_near_as_surface(other, 1, |x, y| x.near(y))
    }
    #[inline(always)]
    pub fn near2_as_surface(&self, other: &BSplineSurface<T>) -> bool {
        self.sub_near_as_surface(other, 1, |x, y| x.near2(y))
    }
    #[inline(always)]
    pub fn near_as_projected_surface(&self, other: &BSplineSurface<T>) -> bool {
        self.sub_near_as_surface(other, 2, |x, y| {
            x.rational_projection().near(&y.rational_projection())
        })
    }
    #[inline(always)]
    pub fn near2_as_projected_surface(&self, other: &BSplineSurface<T>) -> bool {
        self.sub_near_as_surface(other, 2, |x, y| {
            x.rational_projection().near2(&y.rational_projection())
        })
    }
}

impl<T, M> std::ops::MulAssign<&Matrix<T, M>> for BSplineSurface<T>
where
    T: EntityArray<f64>,
    M: MatrixEntity<T>,
{
    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul_assign(&mut self, mat: &Matrix<T, M>) {
        for vecs in &mut self.control_points {
            for vec in vecs {
                *vec *= mat;
            }
        }
    }
}

impl<T, M> std::ops::MulAssign<Matrix<T, M>> for BSplineSurface<T>
where
    T: EntityArray<f64>,
    M: MatrixEntity<T>,
{
    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul_assign(&mut self, mat: Matrix<T, M>) { self.mul_assign(&mat); }
}

impl<T, M> std::ops::Mul<&Matrix<T, M>> for &BSplineSurface<T>
where
    T: EntityArray<f64>,
    M: MatrixEntity<T>,
{
    type Output = BSplineSurface<T>;

    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul(self, mat: &Matrix<T, M>) -> BSplineSurface<T> {
        let mut new_spline = self.clone();
        new_spline *= mat;
        new_spline
    }
}

impl<T, M> std::ops::Mul<Matrix<T, M>> for &BSplineSurface<T>
where
    T: EntityArray<f64>,
    M: MatrixEntity<T>,
{
    type Output = BSplineSurface<T>;

    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul(self, mat: Matrix<T, M>) -> BSplineSurface<T> { self * &mat }
}

impl<T, M> std::ops::Mul<&Matrix<T, M>> for BSplineSurface<T>
where
    T: EntityArray<f64>,
    M: MatrixEntity<T>,
{
    type Output = BSplineSurface<T>;

    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul(mut self, mat: &Matrix<T, M>) -> BSplineSurface<T> {
        self *= mat;
        self
    }
}

impl<T, M> std::ops::Mul<Matrix<T, M>> for BSplineSurface<T>
where
    T: EntityArray<f64>,
    M: MatrixEntity<T>,
{
    type Output = BSplineSurface<T>;

    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul(self, mat: Matrix<T, M>) -> BSplineSurface<T> { self * &mat }
}

impl<T, M> std::ops::Mul<&BSplineSurface<T>> for &Matrix<T, M>
where
    T: EntityArray<f64>,
    M: MatrixEntity<T>,
{
    type Output = BSplineSurface<T>;

    /// A matrix `mat` acts on each control points.
    #[inline(always)]
    fn mul(self, bspline: &BSplineSurface<T>) -> BSplineSurface<T> {
        let mut new_spline = bspline.clone();
        for vecs in &mut new_spline.control_points {
            for vec in vecs {
                *vec = self * &*vec;
            }
        }
        new_spline
    }
}

impl<T, M> std::ops::Mul<&BSplineSurface<T>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: MatrixEntity<T>,
{
    type Output = BSplineSurface<T>;

    /// A matrix `mat` acts on each control points.
    #[inline(always)]
    fn mul(self, bspline: &BSplineSurface<T>) -> BSplineSurface<T> { &self * bspline }
}

impl<T, M> std::ops::Mul<BSplineSurface<T>> for &Matrix<T, M>
where
    T: EntityArray<f64>,
    M: MatrixEntity<T>,
{
    type Output = BSplineSurface<T>;

    /// A matrix `mat` acts on each control points.
    #[inline(always)]
    fn mul(self, mut bspline: BSplineSurface<T>) -> BSplineSurface<T> {
        for vecs in &mut bspline.control_points {
            for vec in vecs {
                *vec = self * &*vec;
            }
        }
        bspline
    }
}

impl<T, M> std::ops::Mul<BSplineSurface<T>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: MatrixEntity<T>,
{
    type Output = BSplineSurface<T>;

    /// A matrix `mat` acts on each control points.
    #[inline(always)]
    fn mul(self, bspline: BSplineSurface<T>) -> BSplineSurface<T> { &self * bspline }
}

impl<T: EntityArray<f64>> std::ops::MulAssign<f64> for BSplineSurface<T> {
    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul_assign(&mut self, scalar: f64) {
        for vecs in &mut self.control_points {
            for vec in vecs {
                *vec *= scalar;
            }
        }
    }
}

impl<T: EntityArray<f64>> std::ops::Mul<f64> for &BSplineSurface<T> {
    type Output = BSplineSurface<T>;

    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul(self, scalar: f64) -> BSplineSurface<T> {
        let mut new_spline = self.clone();
        new_spline *= scalar;
        new_spline
    }
}

impl<T: EntityArray<f64>> std::ops::Mul<f64> for BSplineSurface<T> {
    type Output = BSplineSurface<T>;

    /// A matrix `mat` acts to each control points.
    #[inline(always)]
    fn mul(mut self, scalar: f64) -> BSplineSurface<T> {
        self *= scalar;
        self
    }
}

impl<T: EntityArray<f64>> std::ops::Mul<&BSplineSurface<T>> for f64 {
    type Output = BSplineSurface<T>;

    /// A matrix `mat` acts on each control points.
    #[inline(always)]
    fn mul(self, bspline: &BSplineSurface<T>) -> BSplineSurface<T> {
        let mut new_spline = bspline.clone();
        for vecs in &mut new_spline.control_points {
            for vec in vecs {
                *vec = self * &*vec;
            }
        }
        new_spline
    }
}

impl BSplineSurface<[f64; 3]> {
    /// Returns the normal unit vector at the parameter `(u, v)`.
    pub fn normal_vector(&self, u: f64, v: f64) -> Vector<[f64; 3]> {
        let der0 = self.uderivation().subs(u, v);
        let der1 = self.vderivation().subs(u, v);
        let vec = der0 ^ der1;
        let norm = vec.norm();
        vec / norm
    }
    /// Returns the array of normal vectors at the parameters generated by iterator `params`.
    pub fn normal_vectors<I>(&self, params: I) -> Vec<Vector<[f64; 3]>>
    where I: Iterator<Item = (f64, f64)> {
        let derivation0 = self.uderivation();
        let derivation1 = self.vderivation();
        params
            .map(|(u, v)| {
                let der0 = derivation0.subs(u, v);
                let der1 = derivation1.subs(u, v);
                let vec = der0 ^ der1;
                let norm = vec.norm();
                vec / norm
            })
            .collect()
    }
}

impl BSplineSurface<[f64; 4]> {
    /// Returns the normal unit vector of rational surface at the parameter `(u, v)`.
    pub fn rational_normal_vector(&self, u: f64, v: f64) -> Vector<[f64; 3]> {
        let pt = self.subs(u, v);
        let der0 = self.uderivation().subs(u, v);
        let der1 = self.vderivation().subs(u, v);
        let vec0: Vector<[f64; 3]> = pt.rational_derivation(&der0).into();
        let vec1: Vector<[f64; 3]> = pt.rational_derivation(&der1).into();
        let vec = vec0 ^ vec1;
        let norm = vec.norm();
        vec / norm
    }
    /// Returns the array of normal unit vectors of rational surface at the parameters
    /// generated by iterator `params`.
    pub fn rational_normal_vectors<I>(&self, params: I) -> Vec<Vector<[f64; 3]>>
    where I: Iterator<Item = (f64, f64)> {
        let derivation0 = self.uderivation();
        let derivation1 = self.vderivation();
        params
            .map(|(u, v)| {
                let pt = self.subs(u, v);
                let der0 = derivation0.subs(u, v);
                let der1 = derivation1.subs(u, v);
                let vec0: Vector<[f64; 3]> = pt.rational_derivation(&der0).into();
                let vec1: Vector<[f64; 3]> = pt.rational_derivation(&der1).into();
                let vec = vec0 ^ vec1;
                let norm = vec.norm();
                vec / norm
            })
            .collect()
    }
}

pub type CPColumnIter<'a, T> = std::slice::Iter<'a, Vector<T>>;

pub struct CPRowIter<'a, T> {
    iter: std::slice::Iter<'a, Vec<Vector<T>>>,
    idx: usize,
}

impl<'a, T: EntityArray<f64>> Iterator for CPRowIter<'a, T> {
    type Item = &'a Vector<T>;
    fn next(&mut self) -> Option<&'a Vector<T>> { self.iter.next().map(|arr| &arr[self.idx]) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
    fn count(self) -> usize { self.iter.count() }
}

impl<'a, T: EntityArray<f64>> DoubleEndedIterator for CPRowIter<'a, T> {
    fn next_back(&mut self) -> Option<&'a Vector<T>> {
        self.iter.next_back().map(|arr| &arr[self.idx])
    }
}

impl<'a, T: EntityArray<f64>> ExactSizeIterator for CPRowIter<'a, T> {
    fn len(&self) -> usize { self.iter.len() }
}

impl<'a, T: EntityArray<f64>> std::iter::FusedIterator for CPRowIter<'a, T> {}
