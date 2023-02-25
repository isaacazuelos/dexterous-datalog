/// A binding is a map from usize to usize, but using the index into a `vec` as
/// the key, so the keys must be dense and inserted in order.

#[derive(Debug, Default, PartialEq, PartialOrd, Ord, Eq)]
pub(crate) struct Binding(Vec<usize>);

impl From<Vec<usize>> for Binding {
    fn from(value: Vec<usize>) -> Self {
        Binding(value)
    }
}

impl Binding {
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.0.iter().cloned().enumerate()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, value: usize) -> usize {
        for (k, v) in self.iter() {
            if value == v {
                return k;
            }
        }

        let k = self.0.len();
        self.0.push(value);
        k
    }
}

impl std::ops::Index<usize> for Binding {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
