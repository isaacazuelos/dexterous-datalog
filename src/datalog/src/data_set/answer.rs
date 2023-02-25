use std::collections::BTreeSet as Set;

use crate::{binding::Binding, DataSet};

/// An [`Answer`] is a set of pairs of strings, which correspond to a variable
/// name, which, when bound to a constant, produces an answer to some query.
pub struct Answer(Set<(String, String)>);

impl Answer {
    pub(super) fn new(binding: &Binding, variables: &Binding, data: &DataSet) -> Answer {
        Answer(
            binding
                .iter()
                .map(|(v, c)| {
                    let var_name_index = variables[v];
                    let var_name = &data.variable_names[var_name_index];
                    let constant_name = &data.constant_names[c];
                    (var_name.into(), constant_name.into())
                })
                .collect(),
        )
    }
}

impl std::fmt::Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{",)?;
        let mut iter = self.0.iter();
        if let Some((v, c)) = iter.next() {
            write!(f, "{v} = {c}")?;
        }
        for (v, c) in iter {
            write!(f, ", {v} = {c}")?;
        }
        write!(f, "}}")
    }
}
