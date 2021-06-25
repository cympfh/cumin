use crate::parser::value::*;

#[derive(Debug, Clone)]
pub struct Entries {
    data: Vec<(String, Value)>,
}

impl Entries {
    pub fn new(data: Vec<(String, Value)>) -> Self {
        Self { data }
    }
    pub fn iter(&self) -> std::slice::Iter<(String, Value)> {
        self.data.iter()
    }
}

impl PartialEq for Entries {
    fn eq(&self, other: &Self) -> bool {
        let mut xs = self.data.clone();
        xs.sort_by_key(|item| item.0.clone());
        let mut ys = other.data.clone();
        ys.sort_by_key(|item| item.0.clone());
        xs == ys
    }
}
