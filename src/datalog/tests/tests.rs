use datalog::{self, BlockList, DataSet, Program, Query};

fn star_wars_data() -> DataSet {
    let input = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../samples/star-wars.dl"
    ));

    let program = Program::parse(input, BlockList::OFF).expect("sample code parses");

    let mut data = DataSet::default();
    data.program(&program);
    data
}

#[test]
fn single_clause_rule() {
    let input = " p(a). p(b). q(X) :- p(X). ";
    let program = Program::parse(input, BlockList::OFF).unwrap();
    let mut data = DataSet::default();
    data.program(&program);

    assert_eq!(data.len(), 2);
    data.run();
    assert_eq!(data.len(), 4, "{data}");
}

#[test]
fn spoiler() {
    let mut data = star_wars_data();
    data.run();

    let query = Query::parse("father(X, luke).", BlockList::OFF).unwrap();
    let answers = data.query(&query);
    assert!(answers.iter().any(|a| a.to_string() == "{X = vader}"))
}
