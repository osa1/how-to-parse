use crate::event_to_tree::Container;
use crate::{EventListener, Json, ParseError};

/// An [EventListener] that builds [Json].
pub struct AstBuilderListener<'a> {
    input: &'a str,
    container_stack: Vec<Container>,
    current_container: Option<Container>,
    parsed_object: Option<Json>,
    error: Option<ParseError>,
}

impl<'a> AstBuilderListener<'a> {
    pub fn new(input: &'a str) -> AstBuilderListener<'a> {
        AstBuilderListener {
            input,
            container_stack: vec![],
            current_container: None,
            parsed_object: None,
            error: None,
        }
    }
}

impl<'a> EventListener for AstBuilderListener<'a> {
    fn handle_start_object(&mut self, _byte_offset: usize) -> bool {
        if let Some(container) = self.current_container.take() {
            self.container_stack.push(container);
        }
        self.current_container = Some(Container::new_map());
        true
    }

    fn handle_end_object(&mut self, _byte_offset: usize) -> bool {
        let map = self.current_container.take().unwrap().into_map().finish();
        match self.container_stack.pop() {
            Some(mut container) => {
                container.add_object(map);
                self.current_container = Some(container)
            }
            None => {
                self.parsed_object = Some(map);
            }
        }
        true
    }

    fn handle_start_array(&mut self, _byte_offset: usize) -> bool {
        if let Some(container) = self.current_container.take() {
            self.container_stack.push(container);
        }
        self.current_container = Some(Container::new_array());
        true
    }

    fn handle_end_array(&mut self, _byte_offset: usize) -> bool {
        let array = self.current_container.take().unwrap().into_array();
        match self.container_stack.pop() {
            Some(mut container) => {
                container.add_object(Json::Array(array));
                self.current_container = Some(container)
            }
            None => {
                self.parsed_object = Some(Json::Array(array));
            }
        }
        true
    }

    fn handle_int(&mut self, _byte_offset: usize, i: u64) -> bool {
        let object = Json::Int(i);
        match self.current_container.as_mut() {
            Some(container) => container.add_object(object),
            None => {
                self.parsed_object = Some(object);
            }
        }
        true
    }

    fn handle_str(&mut self, byte_offset: usize, size_in_bytes: usize) -> bool {
        let string = self.input[byte_offset..byte_offset + size_in_bytes].to_string();
        let object = Json::String(string);
        match self.current_container.as_mut() {
            Some(container) => container.add_object(object),
            None => {
                self.parsed_object = Some(object);
            }
        }
        true
    }

    fn handle_bool(&mut self, _byte_offset: usize, b: bool) -> bool {
        let object = Json::Bool(b);
        match self.current_container.as_mut() {
            Some(container) => container.add_object(object),
            None => {
                self.parsed_object = Some(object);
            }
        }
        true
    }

    fn handle_null(&mut self, _byte_offset: usize) -> bool {
        let object = Json::Null;
        match self.current_container.as_mut() {
            Some(container) => container.add_object(object),
            None => {
                self.parsed_object = Some(object);
            }
        }
        true
    }

    fn handle_error(&mut self, error: crate::ParseError) {
        self.error = Some(error);
    }
}

#[cfg(test)]
fn parse(input: &str) -> Result<Json, ParseError> {
    let mut listener = AstBuilderListener::new(input);
    crate::event_push_parser::parse(input, &mut listener);
    if let Some(err) = listener.error {
        return Err(err);
    }
    if let Some(value) = listener.parsed_object {
        return Ok(value);
    }
    panic!()
}

#[test]
fn test_push_parser() {
    for (str, ast) in crate::test_common::ast_tests() {
        println!("Parsing {:?}", str);
        assert_eq!(parse(&str).unwrap(), ast);
    }
}
