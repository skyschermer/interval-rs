// The MIT License (MIT)
// 
// Copyright (c) 2017 Skylor R. Schermer
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in 
// all copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
////////////////////////////////////////////////////////////////////////////////
//!
//! Provides a managed collection of intervals.
//!
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use bound::Bound;
use interval::{
	Interval,
	Normalize,
	LeftIterable,
	RightIterable,
};

// Standard imports.
use std::collections::btree_set;
use std::collections::BTreeSet;
use std::cmp::Ordering;
use std::fmt;



////////////////////////////////////////////////////////////////////////////////
// Split
////////////////////////////////////////////////////////////////////////////////
/// A type which may contain zero, one, or two of a value.
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub enum Split<T> {
	/// No value present.
	None,
	/// One value present.
	One(T),
	/// Two values present.
	Two(T, T),
}



////////////////////////////////////////////////////////////////////////////////
// Selection
////////////////////////////////////////////////////////////////////////////////
/// A possibly non-contiguous selection of intervals.
#[derive(Debug, PartialEq, Clone)]
pub struct Selection<T>(TineSet<T>) where T: PartialOrd + Ord + Clone;

impl<T> Selection<T> where T: PartialOrd + Ord + Clone {
	/// Returns an empty `Selection`.
	pub fn empty() -> Self {
		Selection(TineSet::new())
	}

	/// Returns a `Selection` containing all points within the given intervals.
	pub fn from_intervals<I>(intervals: I) -> Self
		where I: IntoIterator<Item=Interval<T>>
	{
		Selection(TineSet::from_intervals(intervals))
	}


	/// Gets an iterator that returns the `Intervals` in the `Selection` in
	/// ascending order.
	pub fn iter(&self) -> Iter<T> {
		self.0.iter()
	}

	/// Returns whether the `Selection` is full (i.e., contains all points.)
	pub fn is_full(&self) -> bool {
		self.0 == TineSet::from_intervals(Some(Interval::full()))
	}
	
	/// Returns whether the `Selection` is bounded on both sides.
	pub fn is_bounded(&self) -> bool {
		self.0.is_bounded()
	}
	
	/// Returns the lower bound of the `Selection`.
	pub fn lower_bound(&self) -> Option<Bound<T>> {
		self.0.lower_bound()
	}
	
	/// Returns the upper bound of the `Selection`.
	pub fn upper_bound(&self) -> Option<Bound<T>> {
		self.0.upper_bound()
	}

	/// Returns the greatest lower bound of the `Selection`.
	pub fn infimum(&self) -> Option<T> {
		self.0.infimum()
	}

	/// Returns the least upper bound of the `Selection`.
	pub fn supremum(&self) -> Option<T> {
		self.0.supremum()
	}

	/// Returns the smallest `Interval` containing all of the points in the 
	/// `Selection`.
	pub fn enclosing_interval(&self) -> Interval<T> {
		self.0.enclosing_interval()
	}

	/// Returns whether the `Interval` contains the given point.
	pub fn contains(&self, point: &T) -> bool {
		self.0.contains_pt(point)
	}

	/// Returns the largest `Selection` whose points are all contained
	/// entirely within this `Selection` and the given `Selection`.
	pub fn intersect(&self, other: &Self) -> Self {
		Selection(self.0.intersect(&other.0))
	}

	/// Returns a `Selection` containing all of the points contained
	/// within this `Selection` and the given `Selection`.
	pub fn union(&self, other: &Self) -> Self {
		Selection(self.0.union(&other.0))
	}

	/// Returns a `Selection`s containing all of the points contained
	/// within this `Selection` that are not in the given `Selection`.
	pub fn minus(&self, other: &Self) -> Self {
		Selection(self.0.minus(&other.0))
	}

	/// Returns a `Selection` containing all of the points not in 
	/// this `Selection`.
	pub fn complement(&self) -> Self {
		Selection(self.0.complement())
	}

	/// Returns the smallest closed `Selection` containing all of the points in 
	/// this `Selection`.
	pub fn closure(&self) -> Self {
		Selection(self.0.closure())
	}

	/// Returns whether the `Interval` contains the given point.
	pub fn contains_interval(&self, interval: &Interval<T>) -> bool {
		self.0.contains_interval(interval)
	}

	/// Returns the largest `Selection` whose points are all contained
	/// entirely within this `Selection` and the given `Selection`.
	pub fn intersect_interval(&mut self, interval: &Interval<T>) {
		self.0.intersect_interval(interval)
	}

	/// Inserts all of the points in the given interval into the `Selection`.
	pub fn union_interval(&mut self, interval: Interval<T>) {
		self.0.union_interval(interval);
	}
}


impl<T> IntoIterator for Selection<T> where T: PartialOrd + Ord + Clone {
	type Item = Interval<T>;
    type IntoIter = SelectionIter<T>;
    fn into_iter(self) -> Self::IntoIter {
    	SelectionIter {
    		tine_iter: (self.0).0.into_iter(),
    		saved: None,
    	}
    }
}




////////////////////////////////////////////////////////////////////////////////
// TineSet
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, PartialEq, Clone)]
struct TineSet<T>(BTreeSet<Tine<T>>);

impl<T> TineSet<T>
	where T: PartialOrd + Ord + Clone + NextLower + NextUpper
{
	/// Constructs an empty `TineSet`.
	pub fn new() -> Self {
		TineSet(BTreeSet::new())
	}

	/// Constructs a `TineSet` containing the given `Intervals` unioned
	/// together.
	pub fn from_intervals<I>(intervals: I) -> Self
		where I: IntoIterator<Item=Interval<T>>
	{
		let mut dm = TineSet::new();
		for interval in intervals.into_iter() {
			dm.union_interval(interval);
		}
		dm
	}

	/// Gets an iterator that returns the `Intervals` in the `TineSet` in
	/// ascending order.
	pub fn iter(&self) -> Iter<T> {
		Iter {
			tine_iter: self.0.iter(),
			saved:None,
		}
	}

	/// Returns whether the `TineSet` is bounded on both sides.
	pub fn is_bounded(&self) -> bool {
		let l_unbounded = Tine {
			point: None,
			lb: true,
			ub: false,
			incl: false,
		};
		let u_unbounded = Tine {
			point: None,
			lb: true,
			ub: false,
			incl: false,
		};
		
		!self.0.contains(&l_unbounded) && !self.0.contains(&u_unbounded)
	}

	/// Returns the lower bound of the `TineSet`.
	pub fn lower_bound(&self) -> Option<Bound<T>> {
		self.0
			.iter()
			.next()
			.cloned()
			.and_then(Tine::into_bound_lower)
	}
	
	/// Returns the upper bound of the `TineSet`.
	pub fn upper_bound(&self) -> Option<Bound<T>> {
		self.0
			.iter()
			.rev()
			.next()
			.cloned()
			.and_then(Tine::into_bound_upper)
	}

	/// Returns the greatest lower bound of the `TineSet`.
	pub fn infimum(&self) -> Option<T> {
		self.lower_bound().map(|b| b.as_ref().clone())
	}

	/// Returns the least upper bound of the `TineSet`.
	pub fn supremum(&self) -> Option<T> {
		self.upper_bound().map(|b| b.as_ref().clone())
	}

	/// Returns the smallest `Interval` containing all of the points in the 
	/// `TineSet`.
	pub fn enclosing_interval(&self) -> Interval<T> {
		Interval::new(self.lower_bound(), self.upper_bound())
	}

	/// Returns the largest `TineSet` whose points are all contained
	/// entirely within this `TineSet` and the given `TineSet`.
	pub fn intersect(&self, other: &Self) -> Self {
		let mut a_intervals = self.iter();
		let mut b_intervals = other.iter();

		let mut intersection = Self::new();
		while let Some(a_interval) = a_intervals.next() {
			'segment: loop {
				if let Some(b_interval) = b_intervals.next() {
					let i = a_interval.intersect(&b_interval);
					if !i.is_empty() {
						intersection.union_interval(i);
					} else {
						// Nothing more overlapping in this segment.
						break 'segment;
					}

				} else {
					// Nothing more overlapping anywhere.
					return intersection;
				}
			}
		}
		intersection
	}

	/// Returns a `TineSet` containing all of the points contained
	/// within this `TineSet` and the given `TineSet`.
	pub fn union(&self, other: &Self) -> Self {
		let mut union = self.clone();
		for interval in other.iter() {
			union.union_interval(interval);
		}
		union
	}

	/// Returns a `TineSet`s containing all of the points contained
	/// within this `TineSet` that are not in the given `TineSet`.
	pub fn minus(&self, other: &Self) -> Self {
		other.complement()
			.intersect(self)
	}

	/// Returns a `TineSet` containing all of the points not in 
	/// this `TineSet`.
	pub fn complement(&self) -> Self {
		TineSet::from(Interval::full()).intersect(self)
	}

	/// Returns the smallest closed `TineSet` containing all of the
	/// points in this `TineSet`.
	pub fn closure(&self) -> Self {
		TineSet(self.0
			.iter()
			.cloned()
			.map(|mut tine| {
				if tine.point.is_some() {
					tine.incl = true;
				}
				tine
			})
			.collect()
		)
	}

	/// Returns whether the `TineSet` contains all of the points in the
	/// given `Interval`.
	pub fn contains_interval(&self, interval: &Interval<T>) -> bool {
		let check = self
			.intersect(&interval.clone().into())
			.iter()
			.next()
			.map(|ref int| int == interval);

		check == Some(true)
	}

	/// 
	pub fn intersect_interval(&mut self, interval: &Interval<T>) {
		*self = self.intersect(&interval.clone().into());
	}

	/// Inserts the given `Interval` by unioning with the current contents.
	pub fn union_interval(&mut self, interval: Interval<T>) {
		if interval.is_empty() {return}

		let tines = self.widen(Tine::from_interval(interval));

		match tines {
			Split::None		 => unreachable!(),
			Split::One(pt)	 => self.union_pt(pt),
			Split::Two(lb, ub) => self.union_normal(lb, ub),
		}
	}

	/// Returns whether the given point is in the map.
	pub fn contains_pt(&self, point: &T) -> bool {
		let lb = Tine {
			point: Some(point.clone()),
			lb: true,
			ub: false,
			incl: true,
		};
		if self.0.contains(&lb) {return true}

		let ub = Tine {
			point: Some(point.clone()),
			lb: false,
			ub: true,
			incl: true,
		};
		if self.0.contains(&ub) {return true}

		let pt = Tine {
			point: Some(point.clone()),
			lb: false,
			ub: false,
			incl: true,
		};
		self.0.contains(&pt)
	}

	/// Widens the given `Tine` set to encompass neighboring points if their
	/// `Interval` normalization would include them.
	fn widen(&self, tines: Split<Tine<T>>) -> Split<Tine<T>> {
		use self::Split::*;
		match tines {
			None		=> None,
			One(pt)		=> {
				// Try to widen the point on both sides.
				let mut lb = pt.clone();
				lb.ub = false;
				lb.lb = true;

				let mut ub = pt.clone();
				ub.ub = true;
				ub.lb = false;
				
				let new_l = self.widen_l(lb);
				let new_r = self.widen_r(ub);

				if new_l.point == new_r.point {
					// Widening didn't do anthing.
					One(pt)
				} else {
					// Widening worked, use new tines.
					Two(new_l, new_r)
				}
			}
			Two(lb, ub) => Two(self.widen_l(lb), self.widen_r(ub)),
		}
	}

	/// Widens the given `Tine` to encompass leftward neigbors if its
	/// `Interval`'s lower bound normalization would include them.
	fn widen_l(&self, mut tine: Tine<T>) -> Tine<T> {
		while let Some(lb) = tine.point.clone().and_then(|pt| pt.next_lower()) {
			if self.contains_pt(&lb) {
				tine.point = Some(lb);
			} else {
				break
			}
		}
		tine
	}

	/// Widens the given `Tine` to encompass rightward neigbors if its
	/// `Interval`'s upper bound normalization would include them.
	fn widen_r(&self, mut tine: Tine<T>) -> Tine<T> {
		while let Some(ub) = tine.point.clone().and_then(|pt| pt.next_upper()) {
			if self.contains_pt(&ub) {
				tine.point = Some(ub);
			} else {
				break
			}
		}
		tine
	}

	/// Inserts a point `Interval` into the map.
	fn union_pt(&mut self, tine: Tine<T>) {
		// Should have proper bounds set...
		debug_assert!(tine.lb);
		debug_assert!(tine.ub);

		if self.0.is_empty() {
			self.0.insert(tine);
			return
		}

		// Split TineSet.
		let mut r_set = self.0.split_off(&tine);
		let before = self.0.iter().next_back().cloned();
		let after = r_set.iter().next().cloned();

		// Check Tines immediately before (<) and after (>=) the given point...
		match (before, after) {
			// Merge the point with the left bound of after if they overlap.
			(_,			  Some(ref a)) if tine.point == a.point
				=> if let Some(merged) = tine.merge(a.clone()) {
					r_set.insert(merged);
				} else {
					r_set.remove(&a);
				},
			
			// Point is strictly between two others.
			(Some(ref b), Some(ref a)) if b.ub && a.lb
				=> {r_set.insert(tine);},

			// Point is within an existing interval, so do nothing.
			(Some(_),	  Some(_))
				=> (),

			// Point is (>) everything, (>) everything, or selection is empty.
			_	=> {r_set.insert(tine);},
		};

		self.0.extend(r_set);
	}

	/// Inserts a non-point, non-empty `Interval` into the map.
	fn union_normal(&mut self, l_tine: Tine<T>, r_tine: Tine<T>) {
		// Should have proper bounds set...
		debug_assert!(l_tine.lb);
		debug_assert!(r_tine.ub);

		// Early exit for new or full intervals.
		if self.0.is_empty() 
			|| (l_tine.point.is_none() && r_tine.point.is_none()) 
		{
			self.0 = BTreeSet::new();
			self.0.insert(l_tine);
			self.0.insert(r_tine);
			return
		}


		// Split the TineSet at the left tine.
		let mut r_set = self.0.split_off(&l_tine);
		let before = self.0.iter().next_back().cloned();

		// Merge with (=) tine if needed.
		let l_tine = match r_set.iter().cloned().next() {
			Some(ref n) if l_tine.point == n.point => {
				let rl_tine = r_set.take(n).expect("take matched tine");
				l_tine.merge(rl_tine)
			}
			_								 => Some(l_tine), 
		};

		// Split the TineSet at the right tine.
		let mut r_set = r_set.split_off(&r_tine);

		// Merge with (=) tine if needed.
		let r_tine = match r_set.iter().cloned().next() {
			Some(ref n) if r_tine.point == n.point => {
				let rr_tine = r_set.take(n).expect("take matched tine");
				r_tine.merge(rr_tine)
			}
			_								 => Some(r_tine), 
		};

		let after = r_set.iter().next().cloned();


		match (l_tine, r_tine) {
			// Both points are joining intervals.
			(None, None) => {
				debug_assert!(before.expect("before join is lower bound").lb);
				debug_assert!(after.expect("after join is upper bound").ub);
			},

			// Right point joins two intervals.
			(Some(lb), None) => {
				r_set.insert(lb);
				debug_assert!(after.expect("before join is lower bound").ub);	
			},

			// Left point joins two intervals.
			(None, Some(ub)) => {
				r_set.insert(ub);
				debug_assert!(before.expect("after join is upper bound").lb);
			},

			// Neither point joins intervals.
			(Some(lb), Some(ub)) => {
				if before.is_none() 
					|| before.as_ref().map(|b| b.ub) == Some(true)
					|| (before.map(|b| b.lb) == Some(true) && lb.ub)

				{
					r_set.insert(lb);
				}
				if after.is_none() 
					|| after.as_ref().map(|a| a.lb) == Some(true) 
					|| (after.map(|a| a.ub) == Some(true)  && ub.lb)
				{
					r_set.insert(ub);
				}

			}
		}
		
		self.0.extend(r_set);
	}
}

impl<T> IntoIterator for TineSet<T> where T: PartialOrd + Ord + Clone {
	type Item = Interval<T>;
    type IntoIter = SelectionIter<T>;
    fn into_iter(self) -> Self::IntoIter {
    	SelectionIter {
    		tine_iter: self.0.into_iter(),
    		saved: None,
    	}
    }
}

impl<T> From<Interval<T>> for TineSet<T> where T: PartialOrd + Ord + Clone {
	fn from(interval: Interval<T>) -> Self {
		let mut new = Self::new();
		new.union_interval(interval);
		new
	}
}


////////////////////////////////////////////////////////////////////////////////
// Iter
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the intervals in the `TineSet`.
pub struct Iter<'t, T> where T: PartialOrd + Ord + Clone + 't {
	/// The `TineSet` yet to process.
	tine_iter: btree_set::Iter<'t, Tine<T>>,
	
	/// A `Tine` from a previous interval that contains information for the next
	/// `Interval`.
	saved: Option<&'t Tine<T>>,
}

impl<'t, T> Iterator for Iter<'t, T> where T: PartialOrd + Ord + Clone + 't {
	type Item = Interval<T>;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(saved) = self.saved.take() {
			debug_assert!(saved.lb);
			let second = self.tine_iter
				.next()
				.expect("upper bound matching lower bound");
			debug_assert!(second.ub);
			if second.lb {self.saved = Some(second);}

			return Some(Interval::new(
				Tine::into_bound_lower(saved.clone()),
				Tine::into_bound_upper(second.clone())));

		}
		
		if let Some(first) = self.tine_iter.next() {
			debug_assert!(first.lb);

			if first.is_point() {
				return Some(Interval::point(
					first.clone().point.expect("point tine")
				));
			}
			let second = self.tine_iter.next().expect("upper bound");
			if second.lb {self.saved = Some(second);}

			Some(Interval::new(
				Tine::into_bound_lower(first.clone()),
				Tine::into_bound_upper(second.clone())))
		} else {
			None
		}
	}
}



////////////////////////////////////////////////////////////////////////////////
// SelectionIter
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the intervals in the `Selection`.
pub struct SelectionIter<T> where T: PartialOrd + Ord + Clone {
	/// The `Tine` map yet to process.
	tine_iter: btree_set::IntoIter<Tine<T>>,
	
	/// A `Tine` from a previous interval that contains information for the next
	/// `Interval`.
	saved: Option<Tine<T>>,
}

impl<T> Iterator for SelectionIter<T> where T: PartialOrd + Ord + Clone {
	type Item = Interval<T>;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(saved) = self.saved.take() {
			debug_assert!(saved.lb);
			let second = self.tine_iter
				.next()
				.expect("upper bound matching lower bound");
			debug_assert!(second.ub);
			if second.lb {self.saved = Some(second.clone());}

			return Some(Interval::new(
				Tine::into_bound_lower(saved),
				Tine::into_bound_upper(second)));

		}
		
		if let Some(first) = self.tine_iter.next() {
			debug_assert!(first.lb);

			if first.is_point() {
				return Some(Interval::point(first.point.expect("point tine")));
			}
			let second = self.tine_iter.next().expect("upper bound");
			if second.lb {self.saved = Some(second.clone());}

			Some(Interval::new(
				Tine::into_bound_lower(first),
				Tine::into_bound_upper(second)))
		} else {
			None
		}
	}
}



////////////////////////////////////////////////////////////////////////////////
// Tine
////////////////////////////////////////////////////////////////////////////////
/// A portion of an interval.
///
/// Tines are used to implement ordering over the interval bounds in such a way
/// that the `BTreeSet` will always be able to split at the appropriate place
/// for a given bound type.
///
/// Tines representing point intervals are neither upper bounds nor lower
/// bounds.
#[derive(Debug, PartialEq, Clone)]
struct Tine<T> {
	/// The point of the `Tine`.
	pub point: Option<T>,
	/// Whether the `Tine` represents the upper bound of an interval.
	pub lb: bool,
	/// Whether the `Tine` represents the lower bound of an interval.
	pub ub: bool,
	/// Whether the `Tine` point is included in the interval.
	pub incl: bool,
}


impl<T> Tine<T> where T: PartialOrd + Ord + Clone {
	/// Returns whether the `Tine` represents a point interval.
	pub fn is_point(&self) -> bool {
		!self.lb && !self.ub && self.incl && self.point.is_some()
	}

	/// Returns the set of `Tine`s representing the given interval.
	pub fn from_interval(interval: Interval<T>) -> Split<Self> {
		if interval.is_empty() {
			Split::None
		} else if interval.is_degenerate() {
			Split::One(Self::from_degenerate_interval(interval))
		} else {
			let (lb, ub) = Self::from_nondegenerate_interval(interval);
			Split::Two(lb, ub)
		}
	}

	/// Returns the `Tine` representing a point interval.
	fn from_degenerate_interval(mut interval: Interval<T>) -> Self {
		interval.normalize();
		debug_assert!(!interval.is_empty());
		debug_assert!(interval.is_degenerate());
		Tine {
			point: interval.infimum(),
			lb: true,
			ub: true,
			incl: true,
		}
	}

	/// Returns the `Tine` representing a non-empty, non-point interval.
	fn from_nondegenerate_interval(mut interval: Interval<T>)
		-> (Self, Self)
	{
		interval.normalize();
		debug_assert!(!interval.is_empty());
		debug_assert!(!interval.is_degenerate());
		let left = Tine {
			point: interval.infimum(),
			lb: true,
			ub: false,
			incl: !interval.is_left_open(),
		};
		let right = Tine {
			point: interval.supremum(),
			lb: false,
			ub: true,
			incl: !interval.is_right_open(),
		};
		(left, right)
	}

	/// Combines two `Tine`s by 'or'ing their flags.
	pub fn merge(self, other: Self) -> Option<Self> {
		debug_assert!(self.point == other.point);
		debug_assert!(self.point.is_some()
			|| other.point.is_some() 
			|| (self.point.is_none() 
				&& other.point.is_none()
				&& self.lb == other.lb 
				&& self.ub == other.ub));

		let merged = Tine {
			lb: self.lb || other.lb,
			ub: self.ub || other.ub,
			incl: self.incl || other.incl,
			point: self.point,
		};

		if merged.lb && merged.ub && merged.incl {
			// New tine is merging 2 or more intervals.
			None
		} else {
			Some(merged)
		}
	}

	/// Converts a `Tine` into a lower `Bound`.
	pub fn into_bound_lower(lb: Self) -> Option<Bound<T>> {
		debug_assert!(lb.lb);
		if lb.point.is_none() {
			return None;
		}
		match lb.incl {
			true => Some(Bound::Include(lb.point.expect("bounded tine"))),
			false => Some(Bound::Exclude(lb.point.expect("bounded tine"))),
		}
	}

	/// Converts a `Tine` into an upper `Bound`.
	pub fn into_bound_upper(ub: Self) -> Option<Bound<T>> {
		debug_assert!(ub.ub);
		if ub.point.is_none() {
			return None;
		}
		match ub.incl {
			true => Some(Bound::Include(ub.point.expect("bounded tine"))),
			false => Some(Bound::Exclude(ub.point.expect("bounded tine"))),
		}
	}
}


impl<T> Eq for Tine<T> where T: PartialOrd + Ord + Clone {}

impl<T> PartialOrd for Tine<T> where T: PartialOrd + Ord + Clone {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

// Tine ordering is total.
impl<T> Ord for Tine<T> where T: PartialOrd + Ord + Clone {
	fn cmp(&self, other: &Self) -> Ordering {
		match (&self.point, &other.point) {
			// Tine points will compare directly.
			(&Some(ref a), &Some(ref b))
				=> a.cmp(&b),

			// Unbounded tines compare to bounded depending on the direction.
			(&None, &Some(_))
				=> if self.lb {Ordering::Less} else {Ordering::Greater},

			(&Some(_), &None)
				=> if other.lb {Ordering::Greater} else {Ordering::Less},

			// Unbounded tines compare to unbounded depending on the direction.
			_
				=> match (self.lb, self.ub, other.lb, other.ub) {
					(true, false, true, false) |
					(false, true, false, true) => Ordering::Equal,
					(true, false, false, true) => Ordering::Less,
					(false, true, true, false) => Ordering::Greater,
					_						   => panic!("invalid Tine"),
				}
		}
	}
}


impl<T> fmt::Display for Tine<T> where T: fmt::Debug {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}-{:?}", 
			match (self.lb, self.incl, self.ub) {
				(true, false, false)  => "(",  // Lower bound exclusive.
				(true, true, false)	  => "[",  // Lower bound inclusive.
				(true, true, true)	  => "[]", // Overlapping inclusive bounds.
				(true, false, true)	  => ")(", // Overlapping exclusive bounds.
				(false, false, true)  => ")",  // Upper bound exclusive.
				(false, true, true)	  => "]",  // Upper bound inclusive.
				(false, true, false)  => "|",  // Point bound.
				(false, false, false) => unreachable!(), // Invalid tine.
			},
			self.point)
	}
}



////////////////////////////////////////////////////////////////////////////////
// Unification traits
////////////////////////////////////////////////////////////////////////////////
/// Provides a method for getting the next lower point from the given point.
pub trait NextLower: Sized {
	/// Returns the next point less than the given point.
	fn next_lower(&self) -> Option<Self>;
}

/// Provides a method for getting the next upper point from the given point.
pub trait NextUpper: Sized {
	/// Returns the next point greater than the given point.
	fn next_upper(&self) -> Option<Self>;
}

// Default implementation to be overridden by normalizeable types.
impl<T> NextLower for T {
	default fn next_lower(&self) -> Option<Self> {None}
}

// Default implementation to be overridden by normalizeable types.
impl<T> NextUpper for T {
	default fn next_upper(&self) -> Option<Self> {None}
}

// Override of default for RightIterable types.
impl<T> NextLower for T where T: RightIterable {
	fn next_lower(&self) -> Option<Self> {
		self.pred()
	}
}

// Override of default for LeftIterable types.
impl<T> NextUpper for T where T: LeftIterable {
	fn next_upper(&self) -> Option<Self> {
		self.succ()
	}
}





#[cfg(test)]
mod tests {
	use ::interval::Interval;
	use super::*;


	#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
	struct Opaque(i32);

	#[test]
	fn tine_set_insert_disjoint() {
		let mut dm: TineSet<Opaque> = TineSet::new();

		let a = Interval::open(Opaque(0), Opaque(15));
		let b = Interval::open(Opaque(20), Opaque(25));
		let c = Interval::open(Opaque(30), Opaque(35));
		dm.union_interval(b.clone());
		dm.union_interval(a.clone());
		dm.union_interval(c.clone());

		let dm_res: Vec<Interval<Opaque>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![a, b, c]);
	}

	#[test]
	fn tine_set_insert_disjoint_normalized() {
		let mut dm: TineSet<i32> = TineSet::new();

		let a = Interval::open(0, 15);
		let b = Interval::open(20, 25);
		let c = Interval::open(30, 35);
		dm.union_interval(b.clone());
		dm.union_interval(a.clone());
		dm.union_interval(c.clone());

		let dm_res: Vec<Interval<i32>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![a, b, c]);
	}

	#[test]
	fn tine_set_insert_overlap() {
		let mut dm: TineSet<Opaque> = TineSet::new();

		let a = Interval::open(Opaque(0), Opaque(15));
		let b = Interval::open(Opaque(20), Opaque(25));
		let c = Interval::open(Opaque(14), Opaque(26));
		dm.union_interval(b.clone());
		dm.union_interval(a.clone());
		dm.union_interval(c.clone());

		let dm_res: Vec<Interval<Opaque>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::open(Opaque(0), Opaque(26))]);
	}

	#[test]
	fn tine_set_insert_overlap_normalized() {
		let mut dm: TineSet<i32> = TineSet::new();

		let a = Interval::open(0, 15);
		let b = Interval::open(20, 25);
		let c = Interval::open(14, 26);
		dm.union_interval(b.clone());
		dm.union_interval(a.clone());
		dm.union_interval(c.clone());

		let dm_res: Vec<Interval<i32>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::open(0, 26)]);
	}

	#[test]
	fn tine_set_insert_disjoint_close() {
		let mut dm: TineSet<Opaque> = TineSet::new();

		let a = Interval::open(Opaque(0), Opaque(15));
		let b = Interval::open(Opaque(15), Opaque(25));
		let c = Interval::open(Opaque(25), Opaque(30));
		dm.union_interval(c.clone());
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());

		let dm_res: Vec<Interval<Opaque>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![a, b, c]);
	}

	#[test]
	fn tine_set_insert_disjoint_close_normalized() {
		let mut dm: TineSet<i32> = TineSet::new();

		let a = Interval::open(0, 15);
		let b = Interval::open(15, 25);
		let c = Interval::open(25, 30);
		dm.union_interval(c.clone());
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());

		let dm_res: Vec<Interval<i32>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![a, b, c]);
	}

	#[test]
	fn tine_set_insert_overlap_close() {
		let mut dm: TineSet<Opaque> = TineSet::new();

		let a = Interval::open(Opaque(0), Opaque(15));
		let b = Interval::closed(Opaque(15), Opaque(25));
		let c = Interval::open(Opaque(25), Opaque(30));
		dm.union_interval(c.clone());
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());

		let dm_res: Vec<Interval<Opaque>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::open(Opaque(0), Opaque(30))]);
	}

	#[test]
	fn tine_set_insert_overlap_close_normalized() {
		let mut dm: TineSet<i32> = TineSet::new();

		let a = Interval::open(0, 15);
		let b = Interval::closed(15, 25);
		let c = Interval::open(25, 30);
		dm.union_interval(c.clone());
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());

		let dm_res: Vec<Interval<i32>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::open(0, 30)]);
	}

	#[test]
	fn tine_set_insert_disjoint_point() {
		let mut dm: TineSet<Opaque> = TineSet::new();

		let a = Interval::open(Opaque(0), Opaque(15));
		let b = Interval::open(Opaque(15), Opaque(25));
		let c = Interval::point(Opaque(15));
		dm.union_interval(c.clone());
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());

		let dm_res: Vec<Interval<Opaque>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::open(Opaque(0), Opaque(25))]);
	}

	#[test]
	fn tine_set_insert_disjoint_point_normalized() {
		let mut dm: TineSet<i32> = TineSet::new();

		let a = Interval::open(0, 15);
		let b = Interval::open(15, 25);
		let c = Interval::point(15);
		dm.union_interval(c.clone());
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());

		let dm_res: Vec<Interval<i32>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::open(0, 25)]);
	}

	#[test]
	fn tine_set_insert_overlap_exact() {
		let mut dm: TineSet<Opaque> = TineSet::new();

		let a = Interval::open(Opaque(0), Opaque(15));
		let b = Interval::open(Opaque(15), Opaque(25));
		let c = Interval::open(Opaque(0), Opaque(25));
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());
		dm.union_interval(c.clone());

		let dm_res: Vec<Interval<Opaque>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::open(Opaque(0), Opaque(25))]);
	}

	#[test]
	fn tine_set_insert_overlap_exact_normalized() {
		let mut dm: TineSet<i32> = TineSet::new();

		let a = Interval::open(0, 15);
		let b = Interval::open(15, 25);
		let c = Interval::open(0, 25);
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());
		dm.union_interval(c.clone());

		let dm_res: Vec<Interval<i32>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::open(0, 25)]);
	}

	#[test]
	fn tine_set_insert_overlap_widen() {
		let mut dm: TineSet<Opaque> = TineSet::new();

		let a = Interval::open(Opaque(0), Opaque(15));
		let b = Interval::open(Opaque(15), Opaque(25));
		let c = Interval::closed(Opaque(0), Opaque(25));
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());
		dm.union_interval(c.clone());

		let dm_res: Vec<Interval<Opaque>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::closed(Opaque(0), Opaque(25))]);
	}

	#[test]
	fn tine_set_insert_overlap_widen_normalized() {
		let mut dm: TineSet<i32> = TineSet::new();

		let a = Interval::open(0, 15);
		let b = Interval::open(15, 25);
		let c = Interval::closed(0, 25);
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());
		dm.union_interval(c.clone());

		let dm_res: Vec<Interval<i32>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::closed(0, 25)]);
	}

	#[test]
	fn tine_set_insert_overlap_point() {
		let mut dm: TineSet<Opaque> = TineSet::new();

		let a = Interval::open(Opaque(0), Opaque(30));
		let b = Interval::point(Opaque(15));
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());

		let dm_res: Vec<Interval<Opaque>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::open(Opaque(0), Opaque(30))]);
	}

	#[test]
	fn tine_set_insert_overlap_point_normalized() {
		let mut dm: TineSet<i32> = TineSet::new();

		let a = Interval::open(0, 30);
		let b = Interval::point(15);
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());

		let dm_res: Vec<Interval<i32>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::open(0, 30)]);
	}

	#[test]
	fn tine_set_insert_overlap_unbounded() {
		let mut dm: TineSet<Opaque> = TineSet::new();

		let a = Interval::from(Opaque(10));
		let b = Interval::open(Opaque(15), Opaque(30));
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());

		let dm_res: Vec<Interval<Opaque>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::from(Opaque(10))]);
	}

	#[test]
	fn tine_set_insert_overlap_unbounded_normalized() {
		let mut dm: TineSet<i32> = TineSet::new();

		let a = Interval::from(10);
		let b = Interval::open(15, 30);
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());

		let dm_res: Vec<Interval<i32>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::from(10)]);
	}

	#[test]
	fn tine_set_insert_units() {
		let mut dm: TineSet<Opaque> = TineSet::new();

		let a = Interval::point(Opaque(10));
		let b = Interval::point(Opaque(11));
		let c = Interval::point(Opaque(12));
		let d = Interval::point(Opaque(13));
		let e = Interval::point(Opaque(14));
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());
		dm.union_interval(c.clone());
		dm.union_interval(d.clone());
		dm.union_interval(e.clone());

		let dm_res: Vec<Interval<Opaque>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![
			Interval::point(Opaque(10)),
			Interval::point(Opaque(11)),
			Interval::point(Opaque(12)),
			Interval::point(Opaque(13)),
			Interval::point(Opaque(14)),
		]);
	}

	#[test]
	fn tine_set_insert_units_normalized() {
		let mut dm: TineSet<i32> = TineSet::new();

		let a = Interval::point(10);
		let b = Interval::point(11);
		let c = Interval::point(12);
		let d = Interval::point(13);
		let e = Interval::point(14);
		dm.union_interval(a.clone());
		dm.union_interval(b.clone());
		dm.union_interval(d.clone());
		dm.union_interval(e.clone());
		dm.union_interval(c.clone());

		let dm_res: Vec<Interval<i32>> = dm.into_iter().collect();
		assert_eq!(dm_res, vec![Interval::closed(10, 14)]);
	}
}