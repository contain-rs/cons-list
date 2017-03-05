// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! An immutable singly-linked list, as seen in basically every functional language.

#![cfg_attr(test, feature(test))]
#[cfg(test)]
extern crate test;



use std::cmp::Ordering;
use std::iter;
use std::rc::Rc;
use std::hash::{Hash, Hasher};

struct Node<T> {
    elem: T,
    next: Option<Rc<Node<T>>>,
}

impl<T> Node<T> {
    fn new(elem: T) -> Node<T> {
        Node {
            elem: elem,
            next: None,
        }
    }
}

/// An iterator over the items of an ConsList
#[derive(Clone)]
pub struct Iter<'a, T: 'a> {
    head: Option<&'a Node<T>>,
    nelem: usize,
}

/// An immutable singly-linked list, as seen in basically every functional language
pub struct ConsList<T> {
    front: Option<Rc<Node<T>>>,
    length: usize,
}

impl<T> ConsList<T> {
    /// Constructs a new, empty `ConsList`
    pub fn new() -> ConsList<T> {
        ConsList {
            front: None,
            length: 0,
        }
    }

    /// Returns a copy of the list, with `elem` appended to the front
    pub fn append(&self, elem: T) -> ConsList<T> {
        let mut new_node = Node::new(elem);
        new_node.next = self.front.clone();

        ConsList {
            front: Some(Rc::new(new_node)),
            length: self.len() + 1,
        }
    }

    /// Returns a reference to the first element in the list
    pub fn head(&self) -> Option<&T> {
        self.front.as_ref().map(|node| &node.elem)
    }

    /// Returns a copy of the list, with the first element removed
    pub fn tail(&self) -> ConsList<T> {
        self.tailn(1)
    }

    /// Returns a copy of the list, with the first `n` elements removed
    pub fn tailn(&self, n: usize) -> ConsList<T> {
        if self.len() <= n {
            ConsList::new()
        } else {
            let len = self.len() - n;
            let mut head = self.front.as_ref();
            for _ in 0..n {
                head = head.unwrap().next.as_ref();
            }
            ConsList {
                front: Some(head.unwrap().clone()),
                length: len,
            }
        }
    }

    /// Returns the last element in the list
    pub fn last(&self) -> Option<&T> {
        self.iter().last()
    }

    /// Returns a copy of the list, with only the last `n` elements remaining
    pub fn lastn(&self, n: usize) -> ConsList<T> {
        if n >= self.length {
            self.clone()
        } else {
            self.tailn(self.length - n)
        }

    }

    /// Returns an iterator over references to the elements of the list in order
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            head: self.front.as_ref().map(|x| &**x),
            nelem: self.len(),
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        return self.len() == 0;
    }
}

impl<T> Drop for ConsList<T> {
    fn drop(&mut self) {
        // don't want to blow the stack with destructors,
        // but also don't want to walk the whole list.
        // So walk the list until we find a non-uniquely owned item
        let mut head = self.front.take();
        loop {
            let temp = head;
            match temp {
                Some(node) => {
                    match Rc::try_unwrap(node) {
                        Ok(mut node) => {
                            head = node.next.take();
                        }
                        _ => return,
                    }
                }
                _ => return,
            }
        }
    }
}


impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        match self.head.take() {
            None => None,
            Some(head) => {
                self.nelem -= 1;
                self.head = head.next.as_ref().map(|next| &**next);
                Some(&head.elem)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.nelem, Some(self.nelem))
    }
}

impl<T> iter::FromIterator<T> for ConsList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> ConsList<T> {
        let mut list = ConsList::new();
        for elem in iter {
            list = list.append(elem);
        }
        list
    }
}

impl<T: PartialEq> PartialEq for ConsList<T> {
    fn eq(&self, other: &ConsList<T>) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(x, y)| x == y)
    }

    fn ne(&self, other: &ConsList<T>) -> bool {
        self.len() != other.len() || self.iter().zip(other.iter()).all(|(x, y)| x != y)
    }
}

impl<T: PartialOrd> PartialOrd for ConsList<T> {
    fn partial_cmp(&self, other: &ConsList<T>) -> Option<Ordering> {
        let mut a = self.iter();
        let mut b = other.iter();
        loop {
            match (a.next(), b.next()) {
                (None, None) => return Some(std::cmp::Ordering::Equal),
                (None, _) => return Some(std::cmp::Ordering::Less),
                (_, None) => return Some(std::cmp::Ordering::Greater),
                (Some(x), Some(y)) => {
                    match x.partial_cmp(&y) {
                        Some(std::cmp::Ordering::Equal) => (),
                        non_eq => return non_eq,
                    }
                }
            }
        }
    }
}

impl<T> Clone for ConsList<T> {
    fn clone(&self) -> ConsList<T> {
        ConsList {
            front: self.front.clone(),
            length: self.length,
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ConsList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        try!(write!(f, "["));

        for (i, e) in self.iter().enumerate() {
            if i != 0 {
                try!(write!(f, ", "));
            }
            try!(write!(f, "{:?}", *e));
        }

        write!(f, "]")
    }
}

impl<A: Hash> Hash for ConsList<A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for elt in self.iter() {
            elt.hash(state);
        }
    }
}

impl<'a, T> IntoIterator for &'a ConsList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use std::hash;

    use super::ConsList;

    #[test]
    fn test_basic() {
        let mut m = ConsList::new();
        assert_eq!(m.head(), None);
        assert_eq!(m.tail().head(), None);
        m = m.append(Box::new(1));
        assert_eq!(**m.head().unwrap(), 1);
        m = m.tail().append(Box::new(2)).append(Box::new(3));
        assert_eq!(m.len(), 2);
        assert_eq!(**m.head().unwrap(), 3);
        m = m.tail();
        assert_eq!(**m.head().unwrap(), 2);
        m = m.tail();
        assert_eq!(m.len(), 0);
        assert_eq!(m.head(), None);
        m = m.append(Box::new(7)).append(Box::new(5)).append(Box::new(3)).append(Box::new(1));
        assert_eq!(**m.head().unwrap(), 1);
    }

    #[test]
    fn test_tailn() {
        let m = list_from(&[0, 1, 2, 3, 4, 5]);
        assert_eq!(m.tailn(0), m);
        assert_eq!(m.tailn(3), m.tail().tail().tail());
    }

    #[test]
    fn test_last() {
        let mut m = list_from(&[0, 1, 2, 3, 4, 5]);
        assert_eq!(m.last().unwrap(), &5);

        m = ConsList::new();
        assert_eq!(m.last(), None);
    }

    #[test]
    fn test_lastn() {
        let m = list_from(&[0, 1, 2, 3, 4, 5]);
        assert_eq!(m.lastn(0).head(), None);
        assert_eq!(m.lastn(8), m);
        assert_eq!(m.lastn(4), m.tail().tail());
    }

    #[cfg(test)]
    fn generate_test() -> ConsList<i32> {
        list_from(&[0, 1, 2, 3, 4, 5, 6])
    }

    #[cfg(test)]
    fn list_from<T: Clone>(v: &[T]) -> ConsList<T> {
        v.iter().rev().map(|x| (*x).clone()).collect()
    }

    #[test]
    fn test_iterator() {
        let m = generate_test();
        for (i, elt) in m.iter().enumerate() {
            assert_eq!(i as i32, *elt);
        }
        let mut n = ConsList::new();
        assert_eq!(n.iter().next(), None);
        n = n.append(4);
        let mut it = n.iter();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_iterator_clone() {
        let mut n = ConsList::new();
        n = n.append(1).append(2).append(3);
        let mut it = n.iter();
        it.next();
        let mut jt = it.clone();
        assert_eq!(it.next(), jt.next());
        assert_eq!(it.next(), jt.next());
    }

    #[test]
    fn test_eq() {
        let mut n: ConsList<u8> = list_from(&[]);
        let mut m = list_from(&[]);
        assert!(n == m);
        n = n.append(1);
        assert!(n != m);
        m = m.append(1);
        assert!(n == m);

        let n = list_from(&[2, 3, 4]);
        let m = list_from(&[1, 2, 3]);
        assert!(n != m);
    }

    #[test]
    fn test_hash() {
        let mut x = ConsList::new();
        let mut y = ConsList::new();

        let mut h = hash::SipHasher::new();

        assert!(hash::Hash::hash(&x, &mut h) == hash::Hash::hash(&y, &mut h));

        x = x.append(1).append(2).append(3);
        y = y.append(1).append(4).tail().append(2).append(3);

        assert!(hash::Hash::hash(&x, &mut h) == hash::Hash::hash(&y, &mut h));
    }

    #[test]
    fn test_ord() {
        let n = list_from(&[]);
        let m = list_from(&[1, 2, 3]);
        assert!(n < m);
        assert!(m > n);
        assert!(n <= n);
        assert!(n >= n);
    }

    #[test]
    fn test_ord_nan() {
        let nan = 0.0f64 / 0.0;
        let n = list_from(&[nan]);
        let m = list_from(&[nan]);
        assert!(!(n < m));
        assert!(!(n > m));
        assert!(!(n <= m));
        assert!(!(n >= m));

        let n = list_from(&[nan]);
        let one = list_from(&[1.0f64]);
        assert!(!(n < one));
        assert!(!(n > one));
        assert!(!(n <= one));
        assert!(!(n >= one));

        let u = list_from(&[1.0f64, 2.0, nan]);
        let v = list_from(&[1.0f64, 2.0, 3.0]);
        assert!(!(u < v));
        assert!(!(u > v));
        assert!(!(u <= v));
        assert!(!(u >= v));

        let s = list_from(&[1.0f64, 2.0, 4.0, 2.0]);
        let t = list_from(&[1.0f64, 2.0, 3.0, 2.0]);
        assert!(!(s < t));
        assert!(s > one);
        assert!(!(s <= one));
        assert!(s >= one);
    }

    #[test]
    fn test_debug() {
        let list: ConsList<i32> = (0..10).rev().collect();
        assert_eq!(format!("{:?}", list), "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]");

        let list: ConsList<&str> = vec!["just", "one", "test", "more"]
            .iter()
            .rev()
            .map(|&s| s)
            .collect();
        assert_eq!(format!("{:?}", list), r#"["just", "one", "test", "more"]"#);
    }
}

#[cfg(test)]
mod bench {
    use test::Bencher;
    use test;

    use super::ConsList;

    #[bench]
    fn bench_collect_into(b: &mut test::Bencher) {
        let v = &[0i32; 64];
        b.iter(|| { let _: ConsList<i32> = v.iter().map(|x| *x).collect(); })
    }

    #[bench]
    fn bench_append(b: &mut test::Bencher) {
        let mut m: ConsList<i32> = ConsList::new();
        b.iter(|| { m = m.append(0); })
    }

    #[bench]
    fn bench_append_tail(b: &mut test::Bencher) {
        let mut m: ConsList<i32> = ConsList::new();
        b.iter(|| { m = m.append(0).tail(); })
    }

    #[bench]
    fn bench_iter(b: &mut test::Bencher) {
        let v = &[0; 128];
        let m: ConsList<i32> = v.iter().map(|&x| x).collect();
        b.iter(|| {
                   assert!(m.iter().count() == 128);
               })
    }
}

