#[derive(Debug, Default)]
pub(crate) struct NamePool {
    pub(crate) names: Vec<String>,
}

impl NamePool {
    pub(crate) fn len(&self) -> usize {
        self.names.len()
    }

    pub(crate) fn add_name(&mut self, name: &str) -> usize {
        for (i, n) in self.names.iter().enumerate() {
            if name == n {
                return i;
            }
        }

        let i = self.names.len();
        self.names.push(name.into());
        i
    }
}

impl std::ops::Index<usize> for NamePool {
    type Output = str;
    fn index(&self, index: usize) -> &Self::Output {
        &self.names[index]
    }
}
