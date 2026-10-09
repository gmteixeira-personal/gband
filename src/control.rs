use gband_protocol::{Answer, Entry, Key, Process, Value};
use serde_json::{Map, Number, Value as Json};

pub struct Report {
    pub stdout: Vec<u8>,
    pub stderr: Option<String>,
    pub success: bool,
}

impl Report {
    fn printed(stdout: Vec<u8>, success: bool) -> Self {
        Self {
            stdout,
            stderr: None,
            success,
        }
    }

    fn failed(reason: &str) -> Self {
        Self {
            stdout: Vec::new(),
            stderr: Some(one_line(reason)),
            success: false,
        }
    }
}

pub fn to_json(value: &Value) -> Json {
    match value {
        Value::Nil => Json::Null,
        Value::Bool(value) => Json::Bool(*value),
        Value::Int(value) => Json::Number((*value).into()),
        Value::Float(value) => Number::from_f64(*value).map_or(Json::Null, Json::Number),
        Value::Bytes(bytes) => Json::String(String::from_utf8_lossy(bytes).into_owned()),
        Value::Table(entries) => table_to_json(entries),
    }
}

fn table_to_json(entries: &[(Key, Value)]) -> Json {
    let mut sequence: Vec<(i64, &Value)> = entries
        .iter()
        .filter_map(|(key, value)| match key {
            Key::Int(index) => Some((*index, value)),
            Key::Bytes(_) => None,
        })
        .collect();
    sequence.sort_by_key(|(index, _)| *index);
    let is_array = !entries.is_empty()
        && sequence.len() == entries.len()
        && sequence
            .iter()
            .zip(1..)
            .all(|((index, _), expected)| *index == expected);
    if is_array {
        return Json::Array(
            sequence
                .into_iter()
                .map(|(_, value)| to_json(value))
                .collect(),
        );
    }
    let object: Map<String, Json> = entries
        .iter()
        .map(|(key, value)| {
            let key = match key {
                Key::Int(index) => index.to_string(),
                Key::Bytes(bytes) => String::from_utf8_lossy(bytes).into_owned(),
            };
            (key, to_json(value))
        })
        .collect();
    Json::Object(object)
}

pub fn from_json(json: &Json) -> Value {
    match json {
        Json::Null => Value::Nil,
        Json::Bool(value) => Value::Bool(*value),
        Json::Number(number) => match number.as_i64() {
            Some(integer) => Value::Int(integer),
            None => Value::Float(number.as_f64().unwrap_or(f64::NAN)),
        },
        Json::String(text) => Value::string(text.clone()),
        Json::Array(items) => Value::Table(
            items
                .iter()
                .zip(1..)
                .map(|(item, index)| (Key::Int(index), from_json(item)))
                .collect(),
        ),
        Json::Object(fields) => Value::Table(
            fields
                .iter()
                .map(|(key, value)| (Key::string(key.as_str()), from_json(value)))
                .collect(),
        ),
    }
}

fn lua_number(number: f64) -> String {
    if number.is_nan() {
        return if number.is_sign_negative() {
            "-nan"
        } else {
            "nan"
        }
        .to_owned();
    }
    if number.is_infinite() {
        return if number < 0.0 { "-inf" } else { "inf" }.to_owned();
    }
    let scientific = format!("{number:.13e}");
    let (mantissa, exponent) = scientific
        .split_once('e')
        .expect("scientific notation has an exponent");
    let exponent: i32 = exponent.parse().expect("the exponent is a number");
    let written = if (-4..14).contains(&exponent) {
        let decimals = usize::try_from(13 - exponent).unwrap_or(0);
        trim_fraction(&format!("{number:.decimals$}"))
    } else {
        let sign = if exponent < 0 { '-' } else { '+' };
        format!(
            "{}e{sign}{:02}",
            trim_fraction(mantissa),
            exponent.unsigned_abs()
        )
    };
    if written.chars().all(|c| c == '-' || c.is_ascii_digit()) {
        format!("{written}.0")
    } else {
        written
    }
}

fn trim_fraction(text: &str) -> String {
    if !text.contains('.') {
        return text.to_owned();
    }
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn text(value: &Value) -> Vec<u8> {
    match value {
        Value::Nil => b"nil".to_vec(),
        Value::Bool(value) => value.to_string().into_bytes(),
        Value::Int(value) => value.to_string().into_bytes(),
        Value::Float(value) => lua_number(*value).into_bytes(),
        Value::Bytes(bytes) => bytes.clone(),
        Value::Table(_) => to_json(value).to_string().into_bytes(),
    }
}

fn one_line(message: &str) -> String {
    message.replace('\n', "\\n")
}

pub fn process_name(process: Process) -> String {
    match process {
        Process::Server => "server".to_owned(),
        Process::Client(number) => format!("client {number}"),
    }
}

fn process_json(process: Process, answered: bool) -> Map<String, Json> {
    let mut object = Map::new();
    match process {
        Process::Server => {
            object.insert("process".to_owned(), Json::from("server"));
        }
        Process::Client(number) => {
            object.insert("process".to_owned(), Json::from("client"));
            object.insert("client".to_owned(), Json::from(number));
        }
    }
    object.insert("answered".to_owned(), Json::Bool(answered));
    object
}

fn json_document(json: &Json) -> Vec<u8> {
    let mut document = json.to_string().into_bytes();
    document.push(b'\n');
    document
}

fn no_answer(process: Process) -> String {
    format!("{}\t-\tno answer\n", process_name(process))
}

pub fn reload(entries: &[Entry], json: bool) -> Report {
    let mut success = true;
    let mut lines = String::new();
    let mut objects = Vec::new();
    for entry in entries {
        let Some(answer) = &entry.answer else {
            success = false;
            lines.push_str(&no_answer(entry.process));
            objects.push(Json::Object(process_json(entry.process, false)));
            continue;
        };
        let (load, error) = match answer {
            Answer::Loaded { load, error } => (*load, error.clone()),
            other => (0, Some(format!("unexpected answer {other:?}"))),
        };
        success &= error.is_none();
        let name = process_name(entry.process);
        match &error {
            None => lines.push_str(&format!("{name}\t{load}\tok\n")),
            Some(error) => lines.push_str(&format!("{name}\t{load}\terror\t{}\n", one_line(error))),
        }
        let mut object = process_json(entry.process, true);
        object.insert("load".to_owned(), Json::from(load));
        object.insert("error".to_owned(), error.map_or(Json::Null, Json::from));
        objects.push(Json::Object(object));
    }
    let stdout = if json {
        json_document(&Json::Array(objects))
    } else {
        lines.into_bytes()
    };
    Report::printed(stdout, success)
}

pub fn errors(entries: &[Entry], json: bool) -> Report {
    let mut success = true;
    let mut lines = String::new();
    let mut objects = Vec::new();
    for entry in entries {
        let Some(answer) = &entry.answer else {
            success = false;
            lines.push_str(&no_answer(entry.process));
            objects.push(Json::Object(process_json(entry.process, false)));
            continue;
        };
        let (load, errors) = match answer {
            Answer::Errors { load, errors } => (*load, errors.clone()),
            other => (0, vec![format!("unexpected answer {other:?}")]),
        };
        success &= errors.is_empty();
        let name = process_name(entry.process);
        for error in &errors {
            lines.push_str(&format!("{name}\t{load}\t{}\n", one_line(error)));
        }
        let mut object = process_json(entry.process, true);
        object.insert("load".to_owned(), Json::from(load));
        object.insert(
            "errors".to_owned(),
            Json::Array(errors.into_iter().map(Json::from).collect()),
        );
        objects.push(Json::Object(object));
    }
    let stdout = if json {
        json_document(&Json::Array(objects))
    } else {
        lines.into_bytes()
    };
    Report::printed(stdout, success)
}

fn values(entries: &[Entry], missing: &str) -> Result<Vec<Value>, String> {
    let entry = entries.first().ok_or_else(|| missing.to_owned())?;
    match &entry.answer {
        None => Err(format!("{} did not answer", process_name(entry.process))),
        Some(Answer::Values(values)) => values.clone(),
        Some(other) => Err(format!("unexpected answer {other:?}")),
    }
}

pub fn eval(entries: &[Entry], missing: &str, json: bool) -> Report {
    let values = match values(entries, missing) {
        Ok(values) => values,
        Err(reason) => return Report::failed(&reason),
    };
    let stdout = if json {
        json_document(&Json::Array(values.iter().map(to_json).collect()))
    } else {
        values
            .iter()
            .flat_map(|value| {
                let mut line = text(value);
                line.push(b'\n');
                line
            })
            .collect()
    };
    Report::printed(stdout, true)
}

pub fn cmd(entries: &[Entry], missing: &str, json: bool) -> Report {
    let result = match values(entries, missing) {
        Ok(values) => values.into_iter().next().unwrap_or(Value::Nil),
        Err(reason) => return Report::failed(&reason),
    };
    let stdout = if json {
        json_document(&to_json(&result))
    } else if result == Value::Nil {
        Vec::new()
    } else {
        let mut line = text(&result);
        line.push(b'\n');
        line
    };
    Report::printed(stdout, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(entries: Vec<(Key, Value)>) -> Value {
        Value::Table(entries)
    }

    fn sequence(items: Vec<Value>) -> Value {
        table(
            items
                .into_iter()
                .zip(1..)
                .map(|(item, index)| (Key::Int(index), item))
                .collect(),
        )
    }

    fn written(value: &Value) -> String {
        to_json(value).to_string()
    }

    #[test]
    fn plain_values_as_json() {
        assert_eq!(written(&Value::Nil), "null");
        assert_eq!(written(&Value::Bool(true)), "true");
        assert_eq!(written(&Value::Int(-3)), "-3");
        assert_eq!(written(&Value::Float(1.5)), "1.5");
        assert_eq!(written(&Value::Float(f64::INFINITY)), "null");
        assert_eq!(written(&Value::Float(f64::NAN)), "null");
        assert_eq!(written(&Value::string("a\"b")), "\"a\\\"b\"");
        assert_eq!(
            written(&Value::Bytes(vec![b'a', 0xff, b'b'])),
            "\"a\u{fffd}b\""
        );
    }

    #[test]
    fn array_and_object() {
        let array = sequence(vec![Value::string("a"), Value::string("b")]);
        let mixed = table(vec![
            (Key::Int(1), Value::string("a")),
            (Key::string("x"), Value::Int(2)),
        ]);
        let json = Json::Array(vec![to_json(&array), to_json(&mixed)]);
        assert_eq!(json.to_string(), r#"[["a","b"],{"1":"a","x":2}]"#);
    }

    #[test]
    fn table_shapes() {
        assert_eq!(written(&table(Vec::new())), "{}");
        let gap = table(vec![
            (Key::Int(1), Value::Int(1)),
            (Key::Int(3), Value::Int(3)),
        ]);
        assert_eq!(written(&gap), r#"{"1":1,"3":3}"#);
        let unordered = table(vec![
            (Key::Int(2), Value::Int(20)),
            (Key::Int(1), Value::Int(10)),
        ]);
        assert_eq!(written(&unordered), "[10,20]");
        let from_zero = table(vec![(Key::Int(0), Value::Int(0))]);
        assert_eq!(written(&from_zero), r#"{"0":0}"#);
    }

    #[test]
    fn arguments_read_from_json() {
        let json: Json = serde_json::from_str(r#"{"n":1,"list":[1,2],"f":1.0,"e":1e2,"big":18446744073709551615,"s":"x","z":null,"b":false}"#).unwrap();
        let Value::Table(fields) = from_json(&json) else {
            panic!("not a table");
        };
        let field = |name: &str| {
            fields
                .iter()
                .find(|(key, _)| *key == Key::string(name))
                .map(|(_, value)| value.clone())
                .unwrap()
        };
        assert_eq!(field("n"), Value::Int(1));
        assert_eq!(field("list"), sequence(vec![Value::Int(1), Value::Int(2)]));
        assert_eq!(field("f"), Value::Float(1.0));
        assert_eq!(field("e"), Value::Float(100.0));
        assert_eq!(field("big"), Value::Float(18_446_744_073_709_551_615.0));
        assert_eq!(field("s"), Value::string("x"));
        assert_eq!(field("z"), Value::Nil);
        assert_eq!(field("b"), Value::Bool(false));
    }

    #[test]
    fn numbers_as_lua_writes_them() {
        for (number, expected) in [
            (42.0, "42.0"),
            (-2.0, "-2.0"),
            (0.1, "0.1"),
            (1.5, "1.5"),
            (1e100, "1e+100"),
            (1e15, "1e+15"),
            (123_456_789_012_345.0, "1.2345678901234e+14"),
            (1e-5, "1e-05"),
            (0.0001, "0.0001"),
            (1.0 / 3.0, "0.33333333333333"),
            (f64::INFINITY, "inf"),
            (f64::NEG_INFINITY, "-inf"),
        ] {
            assert_eq!(lua_number(number), expected, "{number}");
        }
    }

    fn entry(process: Process, answer: Option<Answer>) -> Entry {
        Entry { process, answer }
    }

    fn loaded(load: u64, error: Option<&str>) -> Option<Answer> {
        Some(Answer::Loaded {
            load,
            error: error.map(str::to_owned),
        })
    }

    fn stdout(report: &Report) -> &str {
        std::str::from_utf8(&report.stdout).unwrap()
    }

    #[test]
    fn reload_lines() {
        let report = reload(
            &[
                entry(Process::Server, loaded(3, None)),
                entry(Process::Client(1), loaded(6, None)),
            ],
            false,
        );
        assert_eq!(stdout(&report), "server\t3\tok\nclient 1\t6\tok\n");
        assert!(report.success);
        let report = reload(
            &[
                entry(Process::Server, loaded(3, None)),
                entry(Process::Client(2), loaded(4, Some("init.lua:1: bad\nmore"))),
                entry(Process::Client(3), None),
            ],
            false,
        );
        assert_eq!(
            stdout(&report),
            "server\t3\tok\nclient 2\t4\terror\tinit.lua:1: bad\\nmore\nclient 3\t-\tno answer\n"
        );
        assert!(!report.success);
    }

    #[test]
    fn reload_as_json() {
        let report = reload(
            &[
                entry(Process::Server, loaded(3, None)),
                entry(Process::Client(1), loaded(6, Some("a\nb"))),
                entry(Process::Client(2), None),
            ],
            true,
        );
        let json: Json = serde_json::from_slice(&report.stdout).unwrap();
        let expected: Json = serde_json::from_str(
            r#"[{"process":"server","answered":true,"load":3,"error":null},
                {"process":"client","client":1,"answered":true,"load":6,"error":"a\nb"},
                {"process":"client","client":2,"answered":false}]"#,
        )
        .unwrap();
        assert_eq!(json, expected);
        assert!(report.stdout.ends_with(b"]\n"));
        assert!(!report.success);
    }

    #[test]
    fn errors_lines_and_json() {
        let entries = [
            entry(
                Process::Server,
                Some(Answer::Errors {
                    load: 2,
                    errors: vec!["boom".to_owned()],
                }),
            ),
            entry(
                Process::Client(1),
                Some(Answer::Errors {
                    load: 5,
                    errors: Vec::new(),
                }),
            ),
        ];
        let report = errors(&entries, false);
        assert_eq!(stdout(&report), "server\t2\tboom\n");
        assert!(!report.success);
        let json: Json = serde_json::from_slice(&errors(&entries, true).stdout).unwrap();
        let expected: Json = serde_json::from_str(
            r#"[{"process":"server","answered":true,"load":2,"errors":["boom"]},
                {"process":"client","client":1,"answered":true,"load":5,"errors":[]}]"#,
        )
        .unwrap();
        assert_eq!(json, expected);
        let clean = [entry(
            Process::Client(1),
            Some(Answer::Errors {
                load: 1,
                errors: Vec::new(),
            }),
        )];
        let report = errors(&clean, false);
        assert!(report.stdout.is_empty());
        assert!(report.success);
    }

    fn answered(values: Result<Vec<Value>, String>) -> [Entry; 1] {
        [entry(Process::Client(1), Some(Answer::Values(values)))]
    }

    #[test]
    fn eval_output() {
        let report = eval(
            &answered(Ok(vec![
                Value::string("client"),
                Value::Int(2),
                Value::Nil,
                Value::Bool(false),
                Value::Float(42.0),
                sequence(vec![Value::Int(1)]),
            ])),
            "",
            false,
        );
        assert_eq!(stdout(&report), "client\n2\nnil\nfalse\n42.0\n[1]\n");
        let report = eval(
            &answered(Ok(vec![
                sequence(vec![Value::Int(1), Value::Int(2)]),
                table(vec![(Key::string("a"), Value::Bool(true))]),
            ])),
            "",
            true,
        );
        assert_eq!(stdout(&report), "[[1,2],{\"a\":true}]\n");
        assert!(eval(&answered(Ok(Vec::new())), "", false).stdout.is_empty());
    }

    #[test]
    fn eval_failures() {
        let report = eval(&answered(Err("chunk:1: boom\nstack".to_owned())), "", false);
        assert_eq!(report.stderr.as_deref(), Some("chunk:1: boom\\nstack"));
        assert!(!report.success);
        let report = eval(&[], "session work has no client", false);
        assert_eq!(report.stderr.as_deref(), Some("session work has no client"));
        let report = eval(&[entry(Process::Client(2), None)], "", false);
        assert_eq!(report.stderr.as_deref(), Some("client 2 did not answer"));
    }

    #[test]
    fn cmd_output() {
        let report = cmd(&answered(Ok(vec![Value::string("hi you")])), "", false);
        assert_eq!(stdout(&report), "hi you\n");
        let report = cmd(&answered(Ok(vec![Value::Nil])), "", false);
        assert!(report.stdout.is_empty());
        assert!(report.success);
        let report = cmd(&answered(Ok(vec![Value::Nil])), "", true);
        assert_eq!(stdout(&report), "null\n");
        let report = cmd(
            &answered(Err("unknown command `absent`".to_owned())),
            "",
            false,
        );
        assert!(report.stderr.unwrap().contains("absent"));
    }
}
