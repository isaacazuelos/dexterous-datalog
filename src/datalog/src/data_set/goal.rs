use crate::{
    binding::Binding,
    data_set::Term,
    parser::{Atom, Const, Relation, Term as TermSyntax, Var},
    DataSet,
};

use super::Tuple;

#[derive(Debug)]
pub(crate) struct Goal {
    pub(super) relation: usize,
    pub(super) terms: Vec<Term>,
}

impl Goal {
    pub(super) fn new(atom: &Atom, variables: &mut Binding, data: &mut DataSet) -> Goal {
        let Atom(Relation(name), terms) = atom;

        let relation = data.declare_relation(name);

        let terms = terms
            .iter()
            .map(|t| match t {
                TermSyntax::Const(Const(c)) => Term::Constant(data.constant_names.add_name(c)),
                TermSyntax::Var(Var(var)) => {
                    let var_name_index = data.variable_names.add_name(var);
                    let v = variables.insert(var_name_index);
                    Term::Variable(v)
                }
            })
            .collect();

        Goal { relation, terms }
    }

    pub(super) fn is_satisfied_by(&self, binding: &Binding, data: &DataSet) -> bool {
        let tuple = self.make_tuple(binding);
        data.relations[self.relation].contains(&tuple)
    }

    pub(super) fn make_tuple(&self, binding: &Binding) -> Tuple {
        let elements = self
            .terms
            .iter()
            .map(|term| match term {
                Term::Constant(c) => *c,
                Term::Variable(v) => binding[*v],
            })
            .collect::<Vec<_>>();

        Tuple::from(elements)
    }
}
