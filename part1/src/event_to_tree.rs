use crate::{Json, ParseError, ParseEvent, ParseEventKind};

/// Parses a stream of [ParseEvent]s to [Json].
pub fn event_to_tree<I: Iterator<Item = Result<ParseEvent, ParseError>>>(
    parser: &mut I,
    input: &str,
) -> Result<Json, ParseError> {
    let mut container_stack: Vec<Container> = vec![];
    let mut current_container: Option<Container> = None;
    let mut parsed_object: Option<Json> = None;

    for event in parser.by_ref() {
        let ParseEvent { kind, byte_offset } = match event {
            Ok(event) => event,
            Err(err) => return Err(err),
        };

        match kind {
            ParseEventKind::StartObject => {
                if let Some(container) = current_container.take() {
                    container_stack.push(container);
                }
                current_container = Some(Container::new_map());
            }

            ParseEventKind::EndObject => {
                let map = current_container.take().unwrap().into_map().finish();
                match container_stack.pop() {
                    Some(mut container) => {
                        container.add_object(map);
                        current_container = Some(container)
                    }
                    None => {
                        parsed_object = Some(map);
                        break;
                    }
                }
            }

            ParseEventKind::StartArray => {
                if let Some(container) = current_container.take() {
                    container_stack.push(container);
                }
                current_container = Some(Container::new_array());
            }

            ParseEventKind::EndArray => {
                let array = current_container.take().unwrap().into_array();
                match container_stack.pop() {
                    Some(mut container) => {
                        container.add_object(Json::Array(array));
                        current_container = Some(container)
                    }
                    None => {
                        parsed_object = Some(Json::Array(array));
                        break;
                    }
                }
            }

            ParseEventKind::Int(int) => {
                let object = Json::Int(int);
                match current_container.as_mut() {
                    Some(container) => container.add_object(object),
                    None => {
                        parsed_object = Some(object);
                        break;
                    }
                }
            }

            ParseEventKind::Str { size_in_bytes } => {
                let string = input[byte_offset..byte_offset + size_in_bytes].to_string();
                let object = Json::String(string);
                match current_container.as_mut() {
                    Some(container) => container.add_object(object),
                    None => {
                        parsed_object = Some(object);
                        break;
                    }
                }
            }

            ParseEventKind::Bool(bool) => {
                let object = Json::Bool(bool);
                match current_container.as_mut() {
                    Some(container) => container.add_object(object),
                    None => {
                        parsed_object = Some(object);
                        break;
                    }
                }
            }

            ParseEventKind::Null => {
                let object = Json::Null;
                match current_container.as_mut() {
                    Some(container) => container.add_object(object),
                    None => {
                        parsed_object = Some(object);
                        break;
                    }
                }
            }

            ParseEventKind::Comment { .. } => {}
        }
    }

    Ok(parsed_object.unwrap())
}

pub(crate) enum Container {
    Array(Vec<Json>),
    Map(MapInProgress),
}

pub(crate) struct MapInProgress {
    built: Vec<(String, Json)>,
    next: Option<String>,
}

impl Container {
    pub(crate) fn new_map() -> Container {
        Container::Map(MapInProgress {
            built: vec![],
            next: None,
        })
    }

    pub(crate) fn new_array() -> Container {
        Container::Array(vec![])
    }

    pub(crate) fn into_map(self) -> MapInProgress {
        match self {
            Container::Array(_) => panic!(),
            Container::Map(map) => map,
        }
    }

    pub(crate) fn into_array(self) -> Vec<Json> {
        match self {
            Container::Array(array) => array,
            Container::Map(_) => panic!(),
        }
    }

    pub(crate) fn add_object(&mut self, object: Json) {
        match self {
            Container::Array(array) => array.push(object),
            Container::Map(map) => map.add(object),
        }
    }
}

impl MapInProgress {
    pub(crate) fn add(&mut self, object: Json) {
        match self.next.take() {
            Some(key) => {
                self.built.push((key, object));
            }
            None => {
                self.next = Some(object.into_string());
            }
        }
    }

    pub(crate) fn finish(self) -> Json {
        let MapInProgress { built, next } = self;
        assert!(next.is_none());
        Json::Object(built)
    }
}

#[test]
fn event_to_tree_tests() {
    for (str, ast) in crate::test_common::ast_tests() {
        let mut parser = crate::parse_events(&str);
        let ast_ = event_to_tree(&mut parser, &str).unwrap();
        assert_eq!(ast_, ast);
    }
}
