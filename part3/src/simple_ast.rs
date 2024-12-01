/// A simple AST without comments and source locations.
#[derive(Debug, PartialEq, Eq)]
pub enum Json {
    Int(u64),
    String(String),
    Bool(bool),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
    Null,
}

impl Json {
    pub(crate) fn into_string(self) -> String {
        match self {
            Json::String(str) => str,
            _ => panic!(),
        }
    }
}
