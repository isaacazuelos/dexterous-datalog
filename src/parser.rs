//! The worlds dumbest parser for datalog!

use std::fmt;

use chumsky::prelude::*;

use crate::Filter;

pub type Program = Vec<Statement>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Fact(Fact),
    Rule(Rule),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Repl {
    Program(Program),
    Query(Query),
}

// Things like `parent(padme, luke).`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact(pub Relation, pub Vec<Const>);

// ancestor(X, Y) :- parent(X, Z), ancestor(Z, Y).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule(pub Atom, pub Vec<Atom>);

// ancestor(X, Y)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Atom(pub Relation, pub Vec<Term>);

// Things like `?- father(X, luke)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query(pub Vec<Atom>);

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

pub fn repl(filter: Filter) -> impl Parser<char, Repl, Error = Simple<char>> {
    program(filter)
        .map(|p| Repl::Program(p))
        .or(query(filter).map(|q| Repl::Query(q)))
}

pub fn program(filter: Filter) -> impl Parser<char, Program, Error = Simple<char>> {
    statement(filter)
        .separated_by(just('.').padded())
        .allow_trailing()
        .then_ignore(end())
}

pub fn query(filter: Filter) -> impl Parser<char, Query, Error = Simple<char>> {
    just("?-")
        .padded()
        .then(query_no_prompt(filter))
        .map(|(_, q)| q)
}

pub fn query_no_prompt(filter: Filter) -> impl Parser<char, Query, Error = Simple<char>> {
    atom(filter)
        .separated_by(just(',').padded())
        .map(|atoms| Query(atoms))
        .then_ignore(end().or(just(".").ignored().then_ignore(end())))
}

// A name looks like a constant if there's at least one letter, and all letters
// are lowercase.
fn is_constant_name(name: &str) -> bool {
    name.chars().any(|c| c.is_ascii_alphabetic())
        && name
            .chars()
            .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_lowercase())
}

fn name(filter: Filter) -> impl Parser<char, String, Error = Simple<char>> {
    text::ident().padded().map(move |name: String| {
        let left: String = name.chars().filter(|c| filter.is_allowed(*c)).collect();

        if left.is_empty() {
            "no".into()
        } else {
            left
        }
    })
}

fn term(filter: Filter) -> impl Parser<char, Term, Error = Simple<char>> {
    name(filter).map(|n| {
        if is_constant_name(&n) {
            Term::Const(Const(n))
        } else {
            Term::Var(Var(n))
        }
    })
}

fn constant(filter: Filter) -> impl Parser<char, Const, Error = Simple<char>> {
    name(filter).validate(|n, span, emit| {
        if !is_constant_name(&n) {
            emit(Simple::custom(
                span,
                format!("expected a constant but found variable `{n}`"),
            ))
        }
        Const(n)
    })
}

fn relation(filter: Filter) -> impl Parser<char, Relation, Error = Simple<char>> {
    name(filter).validate(|n, span, emit| {
        if !is_constant_name(&n) {
            emit(Simple::custom(
                span,
                format!("expected a relation but found variable `{n}`"),
            ))
        }
        Relation(n)
    })
}

fn fact(filter: Filter) -> impl Parser<char, Fact, Error = Simple<char>> {
    relation(filter)
        .then(
            constant(filter)
                .separated_by(just(',').padded())
                .allow_trailing()
                .delimited_by(just('(').padded(), just(')').padded()),
        )
        .map(|(relation, terms)| Fact(relation, terms))
}

fn atom(filter: Filter) -> impl Parser<char, Atom, Error = Simple<char>> {
    relation(filter)
        .then(
            term(filter)
                .separated_by(just(',').padded())
                .allow_trailing()
                .delimited_by(just('(').padded(), just(')').padded()),
        )
        .map(|(rel, terms)| Atom(rel, terms))
}

fn rule(filter: Filter) -> impl Parser<char, Rule, Error = Simple<char>> {
    atom(filter)
        .then(just(":-").padded())
        .then(
            atom(filter)
                .separated_by(just(',').padded())
                .allow_trailing(),
        )
        .map(|((head, _), body)| Rule(head, body))
}

fn statement(filter: Filter) -> impl Parser<char, Statement, Error = Simple<char>> {
    rule(filter)
        .map(|r| Statement::Rule(r))
        .or(fact(filter).map(|f| Statement::Fact(f)))
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
        let syntax = program(Filter::Off).parse(input).unwrap();
        assert!(syntax.is_empty())
    }

    #[test]
    fn parse_fact() {
        let input = " fact ( a, b, c ) ";
        let syntax = fact(Filter::Off).parse(input).unwrap();
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

        let syntax = rule(Filter::Off).parse(input).unwrap();
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
    fn parse_query() {
        let input = "ancestor(X, Y) :- parent(X, Z), ancestor(Z, Y)";

        let syntax = rule(Filter::Off).parse(input).unwrap();
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
