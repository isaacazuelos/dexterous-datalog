use crate::{binding::Binding, counter::Counter, data_set::goal::Goal, parser::Atom, DataSet};

use super::{query::satisfies_all, Set, Tuple};

#[derive(Debug)]
pub(super) struct Rule {
    goal: Goal,
    sub_goals: Vec<Goal>,
    variables: Binding,
}

impl Rule {
    pub(super) fn new(head: &Atom, clauses: &[Atom], data: &mut DataSet) -> Self {
        let mut variables = Binding::default();

        let goal = Goal::new(head, &mut variables, data);

        let sub_goals = clauses
            .iter()
            .map(|sub| Goal::new(sub, &mut variables, data))
            .collect::<Vec<Goal>>();

        Rule {
            goal,
            sub_goals,
            variables,
        }
    }

    pub(super) fn step<'d>(&'d self, data: &'d DataSet) -> Set<Tuple> {
        let mut set = Set::default();

        for var_binding in
            Counter::new(self.variables.len(), data.constants_count()).map(Binding::from)
        {
            if satisfies_all(&var_binding, &self.sub_goals, data) {
                set.insert(self.goal.make_tuple(&var_binding));
            }
        }

        set
    }

    pub(super) fn relation(&self) -> usize {
        self.goal.relation
    }
}

#[cfg(test)]
mod tests {
    use crate::{BlockList, Program};

    use super::*;

    #[test]
    fn variables() {
        let input = " p(a). p(b). ";
        let program = Program::parse(input, BlockList::OFF).unwrap();
        let mut data = DataSet::default();
        data.program(&program);

        let crate::parser::Rule(head, clauses) =
            crate::parser::Rule::parse(" q(X) :- p(X). ", BlockList::OFF).unwrap();

        let rule = Rule::new(&head, &clauses, &mut data);
        assert_eq!(rule.variables.iter().collect::<Vec<_>>(), vec![(0, 0)]);
    }

    #[test]
    fn step() {
        let input = " p(a). p(b). q(c) ";
        let program = Program::parse(input, BlockList::OFF).unwrap();
        let mut data = DataSet::default();
        data.program(&program);

        let crate::parser::Rule(head, clauses) =
            crate::parser::Rule::parse(" q(X) :- p(X). ", BlockList::OFF).unwrap();

        let rule = Rule::new(&head, &clauses, &mut data);

        assert_eq!(
            rule.step(&data),
            Set::from_iter(vec![vec![0].into(), vec![1].into()])
        );
    }
}
