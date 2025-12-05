use std::fmt::Display;

pub struct Env {
    fns: Vec<FnMap>,
}

/// Function namespace -> Call
pub struct FnMap {}

pub struct Namespace {
    path: Vec<String>,
}

impl Namespace {
    pub fn in_namespace(&self, other: &Self) -> bool {
        true
    }
}
