use std::collections::BTreeSet;

use crate::{
    name_pool::NamePool,
    parser::{
        Const, Fact, Program, Query as QuerySyntax, Relation as RelationSyntax, Rule as RuleSyntax,
        Statement,
    },
};

mod answer;
mod goal;
mod query;
mod rule;

pub use self::answer::Answer;
use self::{query::Query, rule::Rule};

pub(self) type Set<T> = BTreeSet<T>;

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub(self) struct Tuple(Vec<usize>);

impl From<Vec<usize>> for Tuple {
    fn from(value: Vec<usize>) -> Self {
        Tuple(value)
    }
}

#[derive(Debug)]
pub(crate) enum Term {
    Constant(usize),
    Variable(usize),
}

#[derive(Default, Debug)]
pub struct DataSet {
    last_len: usize,
    rules: Vec<Rule>,

    /// The names of all the relations in this data set.
    pub(self) relation_names: NamePool,
    /// The names of all the constant values in this data set.
    pub(self) constant_names: NamePool,

    /// The names of variables seen in queries.
    pub(self) variable_names: NamePool,

    /// A relation is a set of tuples which satisfy some predicate. The index
    /// corresponds to relation_names.
    pub(self) relations: Vec<Set<Tuple>>,
}

/// Public interface for working with the data set.
impl DataSet {
    /// The number of facts currently known.
    pub fn len(&self) -> usize {
        self.relations.iter().map(|r| r.len()).sum()
    }

    /// Is this data set empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// This is true if there may be rules which are not fully expanded out yet.
    pub fn is_dirty(&self) -> bool {
        self.last_len != self.len()
    }

    /// Applies the known rules until there are no more facts to discover.
    pub fn run(&mut self) {
        while self.is_dirty() {
            self.last_len = self.len();
            self.step();
        }
    }
}

/// Syntax-based public methods
impl DataSet {
    /// Add the facts and rules from a [`Program`] into this data set.
    pub fn program(&mut self, program: &Program) {
        for statement in program.statements() {
            match statement {
                Statement::Fact(fact) => self.fact(fact),
                Statement::Rule(rule) => self.rule(rule),
            }
        }
    }

    /// Run a [`Query`][`crate::parser::Query`] against this data set.
    ///
    /// Note that this doesn't call [`Dataset::run`].
    pub fn query(&mut self, query: &QuerySyntax) -> Vec<Answer> {
        let QuerySyntax(sub_goals) = query;
        let q = Query::new(sub_goals, self);

        self.search(q)
    }
}

/// Helpers
impl DataSet {
    /// Makes sure some relation name exists in the data set, adding an empty
    /// relation for it and adding the name if it doesn't.
    fn declare_relation(&mut self, name: &str) -> usize {
        let rel = self.relation_names.add_name(name);
        if rel == self.relations.len() {
            self.relations.push(Default::default());
        }
        rel
    }

    /// Takes a step in the fact-expanding loop, used by [`DataSet::run`].
    fn step(&mut self) {
        for i in 0..self.rules.len() {
            let rule = &self.rules[i];
            let new_facts = rule.step(self);
            self.relations[rule.relation()].extend(new_facts);
        }
    }

    /// The number of constant names in this data set.
    pub(self) fn constants_count(&self) -> usize {
        self.constant_names.len()
    }

    fn search(&self, query: Query) -> Vec<Answer> {
        query
            .bindings(self)
            .iter()
            .map(|binding| Answer::new(binding, query.variables(), self))
            .collect()
    }
}

/// Syntax helpers
impl DataSet {
    fn rule(&mut self, rule: &RuleSyntax) {
        let RuleSyntax(head, clauses) = rule;

        let rule = Rule::new(head, clauses, self);
        self.rules.push(rule);
    }

    fn fact(&mut self, fact: &Fact) {
        let Fact(RelationSyntax(name), constants) = fact;

        let tuple = Tuple(
            constants
                .iter()
                .map(|Const(c)| self.constant_names.add_name(c))
                .collect(),
        );

        let rel = self.declare_relation(name);

        self.relations[rel].insert(tuple);
    }
}

impl std::fmt::Display for DataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (rel, relation) in self.relations.iter().enumerate() {
            for tuple in relation.iter() {
                write!(f, "{}(", &self.relation_names[rel])?;
                let mut iter = tuple.0.iter().map(|c| &self.constant_names[*c]);
                if let Some(first) = iter.next() {
                    write!(f, "{first}")?;
                }
                for elt in iter {
                    write!(f, ", {elt}")?;
                }
                writeln!(f, ").")?;
            }
        }

        Ok(())
    }
}
