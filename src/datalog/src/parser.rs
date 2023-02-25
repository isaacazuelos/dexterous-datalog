//! A parser for datalog based on the grammar on [Wikipedia][wiki].
//!
//! [wiki]: https://en.wikipedia.org/wiki/datalog#Syntax

use std::fmt;

use chumsky::prelude::*;

use crate::Error;

#[derive(Clone, Copy)]
pub struct BlockList {
    blocked: &'static str,
}

impl BlockList {
    /// A block list which doesn't block anything.
    pub const OFF: BlockList = BlockList { blocked: "" };

    /// Create a new [`BlockList`] which blocks a given set of [`char`]s.
    pub fn from_disallowed(blocked: &'static str) -> BlockList {
        BlockList { blocked }
    }

    /// Does this allow a given character?
    fn is_allowed(&self, c: char) -> bool {
        !self.blocked.contains(c.to_ascii_lowercase())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Repl {
    Program(Program),
    Query(Query),
}

impl Repl {
    pub fn parse(input: &str, blocked: BlockList) -> Result<Self, Error> {
        Self::parser(blocked).parse(input).map_err(Error::from)
    }

    fn parser(blocked: BlockList) -> impl Parser<char, Self, Error = Simple<char>> {
        Program::parser(blocked)
            .map(Repl::Program)
            .or(Query::parser(blocked).map(Repl::Query))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program(Vec<Statement>);

impl Program {
    pub fn parse(input: &str, blocked: BlockList) -> Result<Self, Error> {
        Self::parser(blocked).parse(input).map_err(Error::from)
    }

    fn parser(blocked: BlockList) -> impl Parser<char, Self, Error = Simple<char>> {
        statement(blocked)
            .separated_by(just('.').padded())
            .allow_trailing()
            .then_ignore(end())
            .map(Program)
    }

    pub(crate) fn statements(&self) -> &[Statement] {
        &self.0
    }
}

// Things like `father(X, luke)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query(pub Vec<Atom>);

impl Query {
    pub fn parse(input: &str, blocked: BlockList) -> Result<Self, Error> {
        Self::parser(blocked).parse(input).map_err(Error::from)
    }

    fn parser(blocked: BlockList) -> impl Parser<char, Self, Error = Simple<char>> {
        atom(blocked)
            .separated_by(just(',').padded())
            .map(Query)
            .then_ignore(end().or(just(".").ignored().then_ignore(end())))
    }
}

// ancestor(X, Y) :- parent(X, Z), ancestor(Z, Y).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Rule(pub Atom, pub Vec<Atom>);

impl Rule {
    #[cfg(test)]
    pub(crate) fn parse(input: &str, blocked: BlockList) -> Result<Self, Error> {
        Self::parser(blocked).parse(input).map_err(Error::from)
    }

    pub(crate) fn parser(blocked: BlockList) -> impl Parser<char, Rule, Error = Simple<char>> {
        atom(blocked)
            .then(just(":-").padded())
            .then(
                atom(blocked)
                    .separated_by(just(',').padded())
                    .allow_trailing(),
            )
            .map(|((head, _), body)| Rule(head, body))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Statement {
    Fact(Fact),
    Rule(Rule),
}

// Things like `parent(padme, luke).`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact(pub Relation, pub Vec<Const>);

// ancestor(X, Y)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Atom(pub Relation, pub Vec<Term>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Term {
    Const(Const),
    Var(Var),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Relation(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Const(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Var(pub String);

// A name looks like a constant if there's at least one letter, and all letters
// are lowercase.
fn is_constant_name(name: &str) -> bool {
    name.chars().any(|c| c.is_ascii_alphabetic())
        && name
            .chars()
            .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_lowercase())
}

fn name(blocked: BlockList) -> impl Parser<char, String, Error = Simple<char>> {
    text::ident().padded().map(move |name: String| {
        let left: String = name.chars().filter(|c| blocked.is_allowed(*c)).collect();

        if left.is_empty() {
            "no".into()
        } else {
            left
        }
    })
}

fn term(blocked: BlockList) -> impl Parser<char, Term, Error = Simple<char>> {
    name(blocked).map(|n| {
        if is_constant_name(&n) {
            Term::Const(Const(n))
        } else {
            Term::Var(Var(n))
        }
    })
}

fn constant(blocked: BlockList) -> impl Parser<char, Const, Error = Simple<char>> {
    name(blocked).validate(|n, span, emit| {
        if !is_constant_name(&n) {
            emit(Simple::custom(
                span,
                format!("expected a constant but found variable `{n}`"),
            ))
        }
        Const(n)
    })
}

fn relation(blocked: BlockList) -> impl Parser<char, Relation, Error = Simple<char>> {
    name(blocked).validate(|n, span, emit| {
        if !is_constant_name(&n) {
            emit(Simple::custom(
                span,
                format!("expected a relation but found variable `{n}`"),
            ))
        }
        Relation(n)
    })
}

fn fact(blocked: BlockList) -> impl Parser<char, Fact, Error = Simple<char>> {
    relation(blocked)
        .then(
            constant(blocked)
                .separated_by(just(',').padded())
                .allow_trailing()
                .delimited_by(just('(').padded(), just(')').padded()),
        )
        .map(|(relation, terms)| Fact(relation, terms))
}

fn atom(blocked: BlockList) -> impl Parser<char, Atom, Error = Simple<char>> {
    relation(blocked)
        .then(
            term(blocked)
                .separated_by(just(',').padded())
                .allow_trailing()
                .delimited_by(just('(').padded(), just(')').padded()),
        )
        .map(|(rel, terms)| Atom(rel, terms))
}

fn statement(blocked: BlockList) -> impl Parser<char, Statement, Error = Simple<char>> {
    Rule::parser(blocked)
        .map(Statement::Rule)
        .or(fact(blocked).map(Statement::Fact))
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Rule(head, body) = self;
        write!(f, "{} :-", head)?;
        for clause in &body[..body.len() - 1] {
            write!(f, " {},", clause)?;
        }
        write!(f, " {}.", body.last().unwrap())
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Query(body) = self;
        write!(f, "?- ")?;
        for term in &body[..body.len() - 1] {
            write!(f, "{}, ", term)?;
        }
        write!(f, "{}.", body.last().unwrap())
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Atom(Relation(name), body) = self;
        write!(f, "{}(", name)?;
        for term in &body[..body.len() - 1] {
            write!(f, "{}, ", term)?;
        }
        write!(f, "{})", body.last().unwrap())
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Const(Const(s)) => write!(f, "{s}"),
            Term::Var(Var(s)) => write!(f, "{s}"),
        }
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn is_constant() {
        assert!(is_constant_name("name"));
        assert!(!is_constant_name("Name"));
        assert!(!is_constant_name("_"));
        assert!(!is_constant_name("_9"));
    }

    #[test]
    fn empty() {
        let input = "";
        let syntax = Program::parse(input, BlockList::OFF).unwrap();
        assert_eq!(syntax, Program(vec![]));
    }

    #[test]
    fn parse_query() {
        let input = "ancestor(X, Y) :- parent(X, Z), ancestor(Z, Y)";

        let syntax = Rule::parse(input, BlockList::OFF).unwrap();
        assert_eq!(
            syntax,
            Rule(
                Atom(
                    Relation("ancestor".into()),
                    vec![Term::Var(Var("X".into())), Term::Var(Var("Y".into()))]
                ),
                vec![
                    Atom(
                        Relation("parent".into()),
                        vec![Term::Var(Var("X".into())), Term::Var(Var("Z".into()))]
                    ),
                    Atom(
                        Relation("ancestor".into()),
                        vec![Term::Var(Var("Z".into())), Term::Var(Var("Y".into()))]
                    ),
                ]
            ),
        )
    }

    #[test]
    fn parse_fact() {
        let input = " fact ( a, b, c ) ";
        let syntax = fact(BlockList::OFF).parse(input).unwrap();
        assert_eq!(
            syntax,
            Fact(
                Relation("fact".into()),
                vec![Const("a".into()), Const("b".into()), Const("c".into()),]
            )
        )
    }

    #[test]
    fn parse_rule() {
        let input = "ancestor(X, Y) :- parent(X, Z), ancestor(Z, Y)";

        let syntax = Rule::parse(input, BlockList::OFF).unwrap();
        assert_eq!(
            syntax,
            Rule(
                Atom(
                    Relation("ancestor".into()),
                    vec![Term::Var(Var("X".into())), Term::Var(Var("Y".into()))]
                ),
                vec![
                    Atom(
                        Relation("parent".into()),
                        vec![Term::Var(Var("X".into())), Term::Var(Var("Z".into()))]
                    ),
                    Atom(
                        Relation("ancestor".into()),
                        vec![Term::Var(Var("Z".into())), Term::Var(Var("Y".into()))]
                    ),
                ]
            ),
        )
    }
}
