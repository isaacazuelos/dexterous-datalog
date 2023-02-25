use crate::{binding::Binding, counter::Counter, data_set::goal::Goal, parser::Atom, DataSet};

use super::Set;

#[derive(Debug)]
pub(super) struct Query {
    variables: Binding,
    sub_goals: Vec<Goal>,
}

impl Query {
    pub(super) fn new(clauses: &[Atom], data: &mut DataSet) -> Query {
        let mut variables = Binding::default();

        let sub_goals = clauses
            .iter()
            .map(|sub| Goal::new(sub, &mut variables, data))
            .collect::<Vec<Goal>>();

        Query {
            variables,
            sub_goals,
        }
    }

    pub(super) fn bindings<'d>(&'d self, data: &'d DataSet) -> Set<Binding> {
        let mut set = Set::default();

        for var_binding in
            Counter::new(self.variables.len(), data.constants_count()).map(Binding::from)
        {
            if satisfies_all(&var_binding, &self.sub_goals, data) {
                set.insert(var_binding);
            }
        }

        set
    }

    pub(super) fn variables(&self) -> &Binding {
        &self.variables
    }
}

pub(crate) fn satisfies_all(vars: &Binding, goals: &[Goal], data: &DataSet) -> bool {
    for goal in goals {
        if !goal.is_satisfied_by(vars, data) {
            return false;
        }
    }

    true
}
