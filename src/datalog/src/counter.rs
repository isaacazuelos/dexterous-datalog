//! Our candidate tuple generator.

/// A counter counts up [`Vec`]s to produce all `Vec<usize>` values of a given
/// length, where each element is between 0 and some max.
///
/// # Note
///
/// The total number of tuples generated must be below [`usize::MAX`].
pub struct Counter {
    /// The length of each Vec that's generated.
    length: usize,

    /// The max value that any element can be.
    max: usize,

    /// The current cursor tells us how many [`Vec`]s we've generated.
    ///
    /// If you think of the cursor as a number in base-`max`, each 'digit'
    /// tells us the value of that element of the next `Vec`.
    cursor: usize,

    /// The last cursor value we'll see.
    end: usize,
}

impl Counter {
    /// Create a new counter where each element will be a `Vec<usize>` of the
    /// given length, and each `usize` will be in `0..max`.
    pub fn new(length: usize, max: usize) -> Counter {
        Counter {
            length,
            max,
            cursor: 0,
            // e.g. length 3, max 10 should be 10^4, since 0 - 9,999 is 10,000 digits
            end: max.pow(length as u32 + 1),
        }
    }

    pub fn is_empty(&self) -> bool {
        (self.cursor == self.end) || self.length == 0
    }
}

impl Iterator for Counter {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            let mut buf = Vec::new();

            for i in 0..self.length {
                let n = (self.cursor / self.max.pow(i as u32)) % self.max;
                buf.push(n);
            }

            self.cursor += 1;

            Some(buf)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_len() {
        let counter = Counter::new(0, 100);
        assert!(counter.is_empty());
    }

    #[test]
    fn empty_max() {
        let counter = Counter::new(1, 0);
        assert!(counter.is_empty());
    }

    #[test]
    fn counter() {
        use std::collections::BTreeSet as Set;

        let mut counter = Counter::new(1, 2);
        let result = Set::from_iter(&mut counter);
        assert!(counter.is_empty());

        let expected = Set::from_iter(vec![vec![0], vec![1]]);

        assert_eq!(result, expected);
    }

    #[test]
    fn counter_longer() {
        use std::collections::BTreeSet as Set;

        let mut counter = Counter::new(3, 4);
        let result = Set::from_iter(&mut counter);
        assert!(counter.is_empty());

        let mut expected = Set::default();
        for x in 0..4 {
            for y in 0..4 {
                for z in 0..4 {
                    expected.insert(vec![x, y, z]);
                }
            }
        }

        assert_eq!(result, expected);
        assert!(!result.contains(&vec![0, 0, 4]));
        assert!(result.contains(&vec![3, 3, 3]));
    }
}
