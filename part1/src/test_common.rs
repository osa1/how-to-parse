use crate::{Json, ParseEventKind};

#[rustfmt::skip]
pub(crate) fn ast_tests() -> Vec<(String, Json)> {
    vec![
        // Simple cases
        (s("123"), Json::Int(123)),
        (s(r#""hi""#), Json::String("hi".to_string())),
        (s("true"), Json::Bool(true)),
        (s("false"), Json::Bool(false)),
        (s("[]"), Json::Array(vec![])),
        (s("{}"), Json::Object(vec![])),
        (s("null"), Json::Null),

        // Simple cases + whitespace
        (s(" 123 "), Json::Int(123)),
        (s(r#" "hi" "#), Json::String("hi".to_string())),
        (s(" true "), Json::Bool(true)),
        (s(" false "), Json::Bool(false)),
        (s(" [ ] "), Json::Array(vec![])),
        (s(" { } "), Json::Object(vec![])),
        (s(" null "), Json::Null),

        // Simple cases, with comments before JSON
        (add_comment_before("123"), Json::Int(123)),
        (add_comment_before(r#""hi""#), Json::String(s("hi"))),
        (add_comment_before("true"), Json::Bool(true)),
        (add_comment_before("false"), Json::Bool(false)),
        (add_comment_before("[]"), Json::Array(vec![])),
        (add_comment_before("{}"), Json::Object(vec![])),
        (add_comment_before("null"), Json::Null),

        // Simple cases, with comments after JSON
        (add_comment_after("123"), Json::Int(123)),
        (add_comment_after(r#""hi""#), Json::String(s("hi"))),
        (add_comment_after("true"), Json::Bool(true)),
        (add_comment_after("false"), Json::Bool(false)),
        (add_comment_after("[]"), Json::Array(vec![])),
        (add_comment_after("{}"), Json::Object(vec![])),
        (add_comment_after("null"), Json::Null),

        // Arrays
        (s(r#"[ 1 ]"#), Json::Array(vec![Json::Int(1)])),
        (
            s(r#"[ true, false, "hi", null, 456, {}, [] ]"#),
            Json::Array(vec![
                Json::Bool(true),
                Json::Bool(false),
                Json::String(s("hi")),
                Json::Null,
                Json::Int(456),
                Json::Object(vec![]),
                Json::Array(vec![]),
            ])
        ),

        // Object
        (s(r#"{ "a" : 1 }"#), Json::Object(vec![(s("a"), Json::Int(1))])),
        (
            s(r#"{ "a": true, "b": false, "c": "hi", "d": null, "e": 456, "f": {}, "g": [] }"#),
            Json::Object(vec![
                (s("a"), Json::Bool(true)),
                (s("b"), Json::Bool(false)),
                (s("c"), Json::String(s("hi"))),
                (s("d"), Json::Null),
                (s("e"), Json::Int(456)),
                (s("f"), Json::Object(vec![])),
                (s("g"), Json::Array(vec![])),
            ])
        ),
    ]
}

#[rustfmt::skip]
pub(crate) fn event_tests() -> Vec<(String, Vec<ParseEventKind>)> {
    use ParseEventKind::*;

    vec![
        // Simple cases
        (s("123"), vec![Int(123)]),
        (s(r#""hi""#), vec![Str { size_in_bytes: 2 }]),
        (s("true"), vec![Bool(true)]),
        (s("false"), vec![Bool(false)]),
        (s("[]"), vec![StartArray, EndArray]),
        (s("{}"), vec![StartObject, EndObject]),
        (s("null"), vec![Null]),

        // Simple cases + whitespace
        (s(" 123 "), vec![Int(123)]),
        (s(r#" "hi" "#), vec![Str { size_in_bytes: 2 }]),
        (s(" true "), vec![Bool(true)]),
        (s(" false "), vec![Bool(false)]),
        (s(" [ ] "), vec![StartArray, EndArray]),
        (s(" { } "), vec![StartObject, EndObject]),
        (s(" null "), vec![Null]),

        // Simple cases, with comments before JSON
        (add_comment_before("123"), vec![COMMENT, Int(123)]),
        (add_comment_before(r#""hi""#), vec![COMMENT, Str { size_in_bytes: 2 }]),
        (add_comment_before("true"), vec![COMMENT, Bool(true)]),
        (add_comment_before("false"), vec![COMMENT, Bool(false)]),
        (add_comment_before("[]"), vec![COMMENT, StartArray, EndArray]),
        (add_comment_before("{}"), vec![COMMENT, StartObject, EndObject]),
        (add_comment_before("null"), vec![COMMENT, Null]),

        // Simple cases, with comments after JSON
        (add_comment_after("123"), vec![Int(123), COMMENT]),
        (add_comment_after(r#""hi""#), vec![Str { size_in_bytes: 2 }, COMMENT]),
        (add_comment_after("true"), vec![Bool(true), COMMENT]),
        (add_comment_after("false"), vec![Bool(false), COMMENT]),
        (add_comment_after("[]"), vec![StartArray, EndArray, COMMENT]),
        (add_comment_after("{}"), vec![StartObject, EndObject, COMMENT]),
        (add_comment_after("null"), vec![Null, COMMENT]),

        // Comments inside container: right after starting token
        (s(r#"[ // hi
            ]"#), vec![StartArray, COMMENT, EndArray]),
        (s(r#"{ // hi
            }"#), vec![StartObject, COMMENT, EndObject]),

        // Comments inside container: before a comma
        (s(r#"[1 // hi
            ,2
            ]"#), vec![StartArray, Int(1), COMMENT, Int(2), EndArray]),
        (s(r#"{"a":1 // hi
            ,"b":2
            }"#), vec![StartObject, Str { size_in_bytes: 1 }, Int(1), COMMENT,
                       Str { size_in_bytes: 1 }, Int(2), EndObject]),

        // Comments inside container: after a comma
        (s(r#"[1, // hi
            2
            ]"#), vec![StartArray, Int(1), COMMENT, Int(2), EndArray]),
        (s(r#"{"a":1, // hi
            "b":2
            }"#), vec![StartObject, Str { size_in_bytes: 1 }, Int(1), COMMENT,
                       Str { size_in_bytes: 1 }, Int(2), EndObject]),

        // Comments inside container: before a colon
        (s(r#"{"a" // hi
            :1}"#), vec![StartObject, Str { size_in_bytes: 1 }, COMMENT, Int(1), EndObject]),

        // Comments inside container: after a colon
        (s(r#"{"a": // hi
            1}"#), vec![StartObject, Str { size_in_bytes: 1 }, COMMENT, Int(1), EndObject]),

        // Arrays
        (s(r#"[ 1 ]"#), vec![StartArray, Int(1), EndArray]),
        (
            s(r#"[ true, false, "hi", null, 456, {}, [] ]"#),
            vec![
                StartArray, Bool(true), Bool(false), Str { size_in_bytes: 2 },
                Null, Int(456), StartObject, EndObject, StartArray, EndArray, EndArray,
            ]
        ),

        // Object
        (s(r#"{ "a" : 1 }"#), vec![StartObject, Str { size_in_bytes: 1 }, Int(1), EndObject]),
        (
            s(r#"{ "a": true, "b": false, "c": "hi", "d": null, "e": 456, "f": {}, "g": [] }"#),
            vec![
                StartObject, Str { size_in_bytes: 1 }, Bool(true), Str { size_in_bytes: 1 }, Bool(false),
                Str { size_in_bytes: 1 }, Str { size_in_bytes: 2 }, Str { size_in_bytes: 1 }, Null,
                Str { size_in_bytes: 1 }, Int(456), Str { size_in_bytes: 1 }, StartObject, EndObject,
                Str { size_in_bytes: 1 }, StartArray, EndArray, EndObject,
            ]
        ),
    ]
}

fn add_comment_before(input: &str) -> String {
    format!("// hi\n{}", input)
}

fn add_comment_after(input: &str) -> String {
    format!("{}\n// hi\n", input)
}

const COMMENT: ParseEventKind = ParseEventKind::Comment { size_in_bytes: 6 };

fn s(s: &str) -> String {
    s.to_string()
}
