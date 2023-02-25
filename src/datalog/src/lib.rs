//! The datalog core.

mod binding;
mod counter;
mod data_set;
mod error;
mod name_pool;
mod parser;

pub use crate::{
    data_set::{Answer, DataSet},
    error::Error,
    parser::{BlockList, Program, Query, Repl},
};
