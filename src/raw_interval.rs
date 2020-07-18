// Copyright 2018 Skylor R. Schermer.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
////////////////////////////////////////////////////////////////////////////////
//! Non-normalizing interval type.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::bound::Bound;
use crate::utility::Few;

// Standard library imports.
use std::cmp::Ordering;


////////////////////////////////////////////////////////////////////////////////
// RawInterval<T>
////////////////////////////////////////////////////////////////////////////////
/// A contiguous interval of the type T. Used to implement the internal state of
/// `Interval`.
/// 
/// [`Interval`]: interval/struct.Interval.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawInterval<T> {
    /// An interval containing no points.
    Empty,
    /// An interval containing only the given point.
    Point(T),
    /// An interval containing all points between two given points, excluding
    /// them both.
    Open(T, T),
    /// An interval containing all points between two given points, including
    /// the greater of the two.
    LeftOpen(T, T),
    /// An interval containing all points between two given points, including
    /// the lesser of the two.
    RightOpen(T, T),
    /// An interval containing all points between two given points, including
    /// them both.
    Closed(T, T),
    /// An interval containing all points less than the given point.
    UpTo(T),
    /// An interval containing all points greater than the given point.
    UpFrom(T),
    /// An interval containing the given point and all points less than it.
    To(T),
    /// An interval containing the given point and all points greater than it.
    From(T),
    /// An interval containing all points.
    Full,
}

impl<T> RawInterval<T> where T: Ord + Clone {
    ////////////////////////////////////////////////////////////////////////////
    // Constructors
    ////////////////////////////////////////////////////////////////////////////
    
    /// Constructs a new interval from the given [`Bound`]s. If the right bound
    /// point is less than the left bound point, an [`Empty`] interval will be 
    /// returned.
    /// 
    /// [`Bound`]: bound/enum.Bound.html
    /// [`Empty`]: #variant.Empty
    pub fn new(lower: Bound<T>, upper: Bound<T>) -> Self {
        use Bound::*;
        use RawInterval::*;
        match (lower, upper) {
            (Include(l), Include(u)) => RawInterval::closed(l, u),
            (Include(l), Exclude(u)) => RawInterval::right_open(l, u),
            (Include(l), Infinite)   => From(l),
            (Exclude(l), Include(u)) => RawInterval::left_open(l, u),
            (Exclude(l), Exclude(u)) => RawInterval::open(l, u),
            (Exclude(l), Infinite)   => UpFrom(l),
            (Infinite,   Include(u)) => To(u),
            (Infinite,   Exclude(u)) => UpTo(u),
            (Infinite,   Infinite)   => Full,
        }
    }
    
    /// Constructs a new [`Open`] interval from the given points. If the upper
    /// point is less than the lower point, an [`Empty`] `RawInterval` will be
    /// returned.
    ///
    /// [`Open`]: #variant.Open
    /// [`Empty`]: #variant.Empty
    pub fn open(lower: T, upper: T) -> Self {
        use RawInterval::*;
        match T::cmp(&lower, &upper) {
            Ordering::Less => Open(lower, upper),
            _              => Empty,
        }
    }
    
    /// Constructs a new [`LeftOpen`] interval from the given points. If the
    /// upper bound point is less than the lower bound point, an [`Empty`]
    /// `RawInterval` will be returned.
    ///
    /// [`LeftOpen`]: #variant.LeftOpen
    /// [`Empty`]: #variant.Empty
    pub fn left_open(lower: T, upper: T) -> Self {
        use RawInterval::*;
        match T::cmp(&lower, &upper) {
            Ordering::Less    => LeftOpen(lower, upper),
            Ordering::Equal   => Point(upper),
            Ordering::Greater => Empty,
        }
    }
    
    /// Constructs a new [`RightOpen`] interval from the given points. If the
    /// upper bound point is less than the lower bound point, an [`Empty`]
    /// `RawInterval` will be returned.
    ///
    /// [`RightOpen`]: #variant.RightOpen
    /// [`Empty`]: #variant.Empty
    pub fn right_open(lower: T, upper: T) -> Self {
        use RawInterval::*;
        match T::cmp(&lower, &upper) {
            Ordering::Less    => RightOpen(lower, upper),
            Ordering::Equal   => Point(lower),
            Ordering::Greater => Empty,
        }
    }
    
    /// Constructs a new [`Closed`] interval from the given points. If the
    /// upper bound point is less than the lower bound point, an [`Empty`]
    /// `RawInterval` will be returned.
    ///
    /// [`Closed`]: #variant.Closed
    /// [`Empty`]: #variant.Empty
    pub fn closed(lower: T, upper: T) -> Self {
        use RawInterval::*;
        match T::cmp(&lower, &upper) {
            Ordering::Less    => Closed(lower, upper),
            Ordering::Equal   => Point(lower),
            Ordering::Greater => Empty,
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    // Bound accessors
    ////////////////////////////////////////////////////////////////////////////

    /// Returns the lower bound of the interval, or `None` if the interval is
    /// empty.
    pub fn lower_bound(&self) -> Option<Bound<T>> {
        use Bound::*;
        use RawInterval::*;
        Some(match *self {
            Empty               => return None,
            Point(ref p)        => Include(p.clone()),
            Open(ref l, _)      => Exclude(l.clone()),
            LeftOpen(ref l, _)  => Exclude(l.clone()),
            RightOpen(ref l, _) => Include(l.clone()),
            Closed(ref l, _)    => Include(l.clone()),
            UpTo(_)             => Infinite,
            UpFrom(ref p)       => Exclude(p.clone()),
            To(_)               => Infinite,
            From(ref p)         => Include(p.clone()),
            Full                => Infinite,
        })
    }
    
    /// Returns the upper bound of the interval, or `None` if the interval is
    /// empty.
    pub fn upper_bound(&self) -> Option<Bound<T>> {
        use Bound::*;
        use RawInterval::*;
        Some(match *self {
            Empty               => return None,
            Point(ref p)        => Include(p.clone()),
            Open(_, ref r)      => Exclude(r.clone()),
            LeftOpen(_, ref r)  => Include(r.clone()),
            RightOpen(_, ref r) => Exclude(r.clone()),
            Closed(_, ref r)    => Include(r.clone()),
            UpTo(ref p)         => Exclude(p.clone()),
            UpFrom(_)           => Infinite,
            To(ref p)           => Include(p.clone()),
            From(_)             => Infinite,
            Full                => Infinite,
        })
    }

    /// Returns the greatest lower bound of the interval.
    pub fn infimum(&self) -> Option<T> {
        use Bound::*;
        match self.lower_bound() {
            Some(Include(ref b)) => Some(b.clone()),
            Some(Exclude(ref b)) => Some(b.clone()),
            _ => None,
        }
    }
    
    /// Returns the least upper bound of the interval.
    pub fn supremum(&self) -> Option<T> {
        use Bound::*;
        match self.upper_bound() {
            Some(Include(ref b)) => Some(b.clone()),
            Some(Exclude(ref b)) => Some(b.clone()),
            _ => None,
        }
    }

    // Query operations
    ////////////////////////////////////////////////////////////////////////////
    
    /// Returns `true` if the interval is [`Empty`].
    ///
    /// [`Empty`]: #variant.Empty
    pub fn is_empty(&self) -> bool {
        use RawInterval::*;
        match *self {
            Empty => true,
            _     => false,
        }
    }

    /// Returns `true` if the interval is [`Full`].
    ///
    /// [`Full`]: #variant.Full
    pub fn is_full(&self) -> bool {
        use RawInterval::*;
        match *self {
            Full => true,
            _     => false,
        }
    }

    /// Returns `true` if the interval contains the given point.
    pub fn contains(&self, point: &T) -> bool {
        use RawInterval::*;
        match *self {
            Empty                   => false,
            Point(ref p)            => point == p,
            Open(ref l, ref r)      => point > l && point < r,
            LeftOpen(ref l, ref r)  => point > l && point <= r,
            RightOpen(ref l, ref r) => point >= l && point < r,
            Closed(ref l, ref r)    => point >= l && point <= r,
            UpTo(ref p)             => point < p,
            UpFrom(ref p)           => point > p,
            To(ref p)               => point <= p,
            From(ref p)             => point >= p,
            Full                    => true,
        }
    }

    // Set comparisons
    ////////////////////////////////////////////////////////////////////////////
    
    /// Returns `true` if the interval overlaps the given interval.
    pub fn intersects(&self, other: &Self) -> bool {
        !self.intersect(other).is_empty()
    }

    /// Returns `true` if the given intervals share any boundary points.
    pub fn adjacent(&self, other: &Self) -> bool {
        let a = match (self.lower_bound(), other.upper_bound()) {
            (Some(lb), Some(ub)) => lb.union_adjacent(&ub),
            _ => false,

        };
        let b = match (self.upper_bound(), other.lower_bound()) {
            (Some(ub), Some(lb)) => lb.union_adjacent(&ub),
            _ => false,
        };
        a || b
    }

    // Set operations
    ////////////////////////////////////////////////////////////////////////////

    /// Returns a `Vec` of `RawInterval`s containing all of the points not in
    /// the interval.
    pub fn complement(&self) -> impl Iterator<Item=Self> {
        use RawInterval::*;
        match *self {
            Empty                   => Few::One(Full),
            Point(ref p)            => Few::Two(UpTo(p.clone()), UpFrom(p.clone())),
            Open(ref l, ref r)      => Few::Two(To(l.clone()), From(r.clone())),
            LeftOpen(ref l, ref r)  => Few::Two(To(l.clone()), UpFrom(r.clone())),
            RightOpen(ref l, ref r) => Few::Two(UpTo(l.clone()), From(r.clone())),
            Closed(ref l, ref r)    => Few::Two(UpTo(l.clone()), UpFrom(r.clone())),
            UpTo(ref p)             => Few::One(From(p.clone())),
            UpFrom(ref p)           => Few::One(To(p.clone())),
            To(ref p)               => Few::One(UpFrom(p.clone())),
            From(ref p)             => Few::One(UpTo(p.clone())),
            Full                    => Few::Zero,
        }
    }

    /// Returns the largest interval whose points are all contained entirely
    /// within this interval and the given interval.
    pub fn intersect(&self, other: &Self) -> Self {
        let lb = match (self.lower_bound(), other.lower_bound()) {
            (Some(a), Some(b)) => a.greatest_intersect(&b),
            _                  => return RawInterval::Empty, // Either Empty.
        };

        let ub = match (self.upper_bound(), other.upper_bound()) {
            (Some(a), Some(b)) => a.least_intersect(&b),
            _                  => return RawInterval::Empty, // Either Empty.
        };

        if lb.as_ref() == ub.as_ref() && 
            ((lb.is_inclusive() && ub.is_exclusive()) ||
             (lb.is_exclusive() && ub.is_inclusive()))
        {
            RawInterval::Empty
        } else {
            RawInterval::new(lb, ub)
        }
    }
    
    /// Returns a `Vec` of `RawInterval`s containing all of the points 
    /// contained within this interval and the given interval., vec![a, b]);
    pub fn union(&self, other: &Self) -> impl Iterator<Item=Self> {
        match (self.is_empty(), other.is_empty()) {
            (true,  true)  => Few::Zero,
            (true,  false) => Few::One(other.clone()),
            (false, true)  => Few::One(self.clone()),
            (false, false) => {
                // if self.lb > other.ub || other.lb < self.ub
                if self.intersects(other) || self.adjacent(other) {
                    Few::One(self.enclose(other))
                } else {
                    Few::Two(self.clone(), other.clone())
                }
            },
        }
    }
    
    /// Returns a `Vec` of `RawInterval`s containing all of the points
    /// contained within this interval that are not in the given interval.
    pub fn minus(&self, other: &Self) -> impl Iterator<Item=Self> {
        other.complement()
            .map(|i| self.intersect(&i))
            .filter(|i| !i.is_empty())
            .collect::<Vec<_>>()
            .into_iter()
    }
    
    /// Returns the smallest interval that contains all of the points contained
    /// within this interval and the given interval.
    pub fn enclose(&self, other: &Self) -> Self {
        let lb = match (self.lower_bound(), other.lower_bound()) {
            (Some(a), Some(b)) => a.least_union(&b),
            (Some(a), None)    => a,
            (None,    Some(b)) => b,
            (None,    None)    => return RawInterval::Empty, // Both Empty.
        };

        let ub = match (self.upper_bound(), other.upper_bound()) {
            (Some(a), Some(b)) => a.greatest_union(&b),
            (Some(a), None)    => a,
            (None,    Some(b)) => b,
            (None,    None)    => return RawInterval::Empty, // Both Empty.
        };

        RawInterval::new(lb, ub)
    }

    /// Returns the smallest closed interval that contains all of the points
    /// contained within the interval.
    pub fn closure(&self) -> Self {
        use RawInterval::*;
        match self {
            &Open(ref l, ref r)      => Closed(l.clone(), r.clone()),
            &LeftOpen(ref l, ref r)  => Closed(l.clone(), r.clone()),
            &RightOpen(ref l, ref r) => Closed(l.clone(), r.clone()),
            &UpTo(ref r)             => To(r.clone()),
            &UpFrom(ref l)           => From(l.clone()),
            _                        => self.clone(),
        }
    }

    // Bulk set operations
    ////////////////////////////////////////////////////////////////////////////

    /// Returns the interval enclosing all of the given intervals.
    pub fn enclose_all<I>(intervals: I) -> Self
        where I: Iterator<Item=Self>
    {
        intervals.fold(RawInterval::Full, |acc, i| acc.enclose(&i))
    }

    /// Returns the intersection of all of the given intervals.
    pub fn intersect_all<I>(intervals: I) -> Self
        where I: Iterator<Item=Self>
    {
        intervals.fold(RawInterval::Full, |acc, i| acc.intersect(&i))
    }

    /// Returns the union of all of the given intervals.
    pub fn union_all<I>(intervals: I) -> impl Iterator<Item=Self>
        where I: Iterator<Item=Self>
    {
        // TODO: Consider using selection/disjunction map. It may be faster.
        let mut it = intervals.filter(|i| !i.is_empty());
   
        // Get first interval.
        if let Some(start) = it.next() {
            // Fold over remaining intervals.
            it.fold(vec![start], |mut prev, next| {
                // Early exit for full interval.
                if next == RawInterval::Full {
                    return vec![RawInterval::Full];
                }
                let mut append = true;
                for item in prev.iter_mut() {
                    if item.intersects(&next) || item.adjacent(&next) {
                        *item = item.enclose(&next);
                        append = false;
                        break;
                    }
                }
                if append {prev.push(next);}
                prev
            })
        } else {
           Vec::new()
        }.into_iter()
    }
}

// Display using interval notation.
impl<T> std::fmt::Display for RawInterval<T> where T: std::fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RawInterval::*;
        match *self {
            Empty                   => write!(f, "Ø"),
            Point(ref p)            => write!(f, "{}", p),
            Open(ref l, ref r)      => write!(f, "({}, {})", l, r),
            LeftOpen(ref l, ref r)  => write!(f, "({}, {}]", l, r),
            RightOpen(ref l, ref r) => write!(f, "[{}, {})", l, r),
            Closed(ref l, ref r)    => write!(f, "[{}, {}]", l, r),
            UpTo(ref p)             => write!(f, "(-∞, {})", p),
            UpFrom(ref p)           => write!(f, "({}, ∞)", p),
            To(ref p)               => write!(f, "(-∞, {})", p),
            From(ref p)             => write!(f, "({}, ∞)", p),
            Full                    => write!(f, "(-∞, ∞)"),
        }
    }
}
