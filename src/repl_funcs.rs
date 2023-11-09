use crate::error::Result;

use repl_rs::Convert;
use repl_rs::Value;
use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub struct Context {
    list: VecDeque<String>,
}

// Append name to list
pub fn append(args: HashMap<String, Value>, context: &mut Context) -> Result<Option<String>> {
    let name: String = args["name"].convert()?;
    context.list.push_back(name);
    let list: Vec<String> = context.list.clone().into();

    Ok(Some(list.join(", ")))
}

// Prepend name to list
pub fn prepend(args: HashMap<String, Value>, context: &mut Context) -> Result<Option<String>> {
    let name: String = args["name"].convert()?;
    context.list.push_front(name);
    let list: Vec<String> = context.list.clone().into();

    Ok(Some(list.join(", ")))
}

// Add two numbers.
pub fn add<T>(args: HashMap<String, Value>, _context: &mut T) -> Result<Option<String>> {
    let first: i32 = args["first"].convert()?;
    let second: i32 = args["second"].convert()?;

    Ok(Some((first + second).to_string()))
}

// Write "Hello"
pub fn hello<T>(args: HashMap<String, Value>, _context: &mut T) -> Result<Option<String>> {
    Ok(Some(format!("Hello, {}", args["who"])))
}
