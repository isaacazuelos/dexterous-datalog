use std::collections::BTreeSet;

use crate::error::Error;
use crate::parser::{
    Atom, Const, Fact, Program, Query as QuerySyntax, Relation as RelationSyntax,
    Rule as RuleSyntax, Statement, Term as TermSyntax, Var,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Tuple(Vec<usize>);

type Relation = BTreeSet<Tuple>;

pub struct DataSet {
    last_len: usize,

    relation_names: NamePool,
    constant_names: NamePool,
    variable_names: NamePool,

    rules: Vec<Rule>,

    relations: Vec<Relation>,
}

impl Default for DataSet {
    fn default() -> Self {
        DataSet {
            last_len: 0,
            relation_names: NamePool::default(),
            constant_names: NamePool::default(),
            variable_names: NamePool::default(),
            rules: Vec::new(),

            relations: Vec::new(),
        }
    }
}

impl DataSet {
    pub fn add_program(&mut self, program: &Program) {
        for statement in program {
            match statement {
                Statement::Fact(fact) => self.add_fact(fact),
                Statement::Rule(rule) => self.add_rule(rule),
            }
        }
    }

    pub fn add_rule(&mut self, rule: &RuleSyntax) {
        let RuleSyntax(head, clauses) = rule;

        let rule = Rule::new(head, clauses, self);
        self.rules.push(rule);
    }

    pub fn step(&mut self) {
        for i in 0..self.rules.len() {
            let rule = &self.rules[i];
            let new = rule.next(&self);
            self.relations[rule.relation()].extend(new);
        }
    }

    fn len(&self) -> usize {
        self.relations.iter().map(|r| r.len()).sum()
    }

    pub fn run(&mut self) {
        while self.last_len != self.len() {
            self.last_len = self.len();
            self.step();
        }
    }

    pub fn run_query(&mut self, query: &QuerySyntax) -> Result<(), Error> {
        let QuerySyntax(_sub_goals) = query;
        println!("I didn't quite get to this. :(");
        Ok(())
    }
}

impl DataSet {
    fn add_fact(&mut self, fact: &Fact) {
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

    fn declare_relation(&mut self, name: &str) -> usize {
        let rel = self.relation_names.add_name(name);
        if rel == self.relations.len() {
            self.relations.push(Default::default());
        }
        rel
    }

    fn constants_count(&self) -> usize {
        self.constant_names.names.len()
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

#[derive(Debug, Default)]
struct NamePool {
    names: Vec<String>,
}

impl NamePool {
    fn add_name(&mut self, name: &str) -> usize {
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

#[derive(Debug)]
enum Term {
    Constant(usize),
    Variable(usize),
}

#[derive(Debug)]
pub struct Goal {
    relation: usize,
    terms: Vec<Term>,
}

impl Goal {
    fn new(atom: &Atom, variables: &mut Binding, data: &mut DataSet) -> Goal {
        let Atom(RelationSyntax(name), terms) = atom;

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

    fn is_satisfied_by(&self, binding: &Binding, data: &DataSet) -> bool {
        let relation = &data.relations[self.relation];
        let tuple = self.make_tuple(binding);

        relation.contains(&tuple)
    }

    fn make_tuple(&self, binding: &Binding) -> Tuple {
        // binding: local -> const name
        // variables: local -> var name

        let elements = self
            .terms
            .iter()
            .map(|term| match term {
                Term::Constant(c) => *c,
                Term::Variable(v) => {
                    // v is a local, and we want a constant
                    binding[*v]
                }
            })
            .collect::<Vec<_>>();

        Tuple(elements)
    }
}

#[derive(Debug)]
pub struct Rule {
    // a mapping of all the variables in this rule and it's sub-goals into data
    // set's variable name pool.
    variables: Binding,
    goal: Goal,
    sub_goals: Vec<Goal>,
}

impl Rule {
    pub fn new(head: &Atom, clauses: &[Atom], data: &mut DataSet) -> Self {
        let mut variables = Binding::default();

        let goal = Goal::new(head, &mut variables, data);

        let sub_goals = clauses
            .iter()
            .map(|sub| Goal::new(sub, &mut variables, data))
            .collect::<Vec<Goal>>();

        Rule {
            variables,
            goal,
            sub_goals,
        }
    }

    /// Produces a list of bindings that satisfy this rule.
    pub fn bindings<'d>(&'d self, data: &'d DataSet) -> impl Iterator<Item = Binding> + 'd {
        Counter::new(self.variables.len(), data.constants_count())
            .map(Binding)
            .filter(|binding| {
                self.sub_goals
                    .iter()
                    .all(|sg| sg.is_satisfied_by(&binding, data))
            })
    }

    pub fn next(&self, data: &DataSet) -> Relation {
        Relation::from_iter(
            self.bindings(data)
                .map(|binding| self.goal.make_tuple(&binding)),
        )
    }

    fn relation(&self) -> usize {
        self.goal.relation
    }
}

pub struct _Query {}

impl _Query {
    // fn _search(&self, _data: &DataSet) -> Vec<Binding> {
    //     todo!()
    // }
}

#[derive(Debug, Default, PartialEq, PartialOrd, Ord, Eq)]
struct VarBindings(Binding);

#[derive(Debug, Default, PartialEq, PartialOrd, Ord, Eq)]
struct ConstBindings(Binding);

#[derive(Debug, Default, PartialEq, PartialOrd, Ord, Eq)]
pub struct Binding(Vec<usize>);

impl Binding {
    fn iter(&self) -> impl Iterator<Item = &usize> {
        self.0.iter()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn insert(&mut self, value: usize) -> usize {
        for (i, n) in self.iter().enumerate() {
            if value == *n {
                return i;
            }
        }

        let i = self.0.len();
        self.0.push(value);
        i
    }
}

impl std::ops::Index<usize> for Binding {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

pub struct Counter {
    tup_len: usize,
    max: usize,
    end: usize,
    cursor: usize,
}

impl Counter {
    pub fn new(tup_len: usize, max: usize) -> Counter {
        Counter {
            tup_len,
            max,
            end: tup_len.pow(max as u32),
            cursor: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cursor == self.end
    }
}

impl Iterator for Counter {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            let mut buf = Vec::new();

            for i in 0..self.tup_len {
                let n = (self.cursor / self.max.pow(i as u32)) % self.max;
                buf.push(n);
            }

            self.cursor += 1;

            Some(buf)
        }
    }
}
