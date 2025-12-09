use std::fmt::Write;

// Add this as a new function in your file
pub fn pjson_to_rust_code(value: &serde_json::Value) -> String {
    let mut output = String::new();
    let indent = "    ".to_string();
    write_element(value, "", &mut output, &indent);
    output
}

#[test]
fn test_pjson_to_rust_code() {
    let content = include_bytes!(r"../examples/layout_demo/layout.pjson");

    let json = pjson::PJsonReader::from_pjson(content);
    let json = String::from_utf8_lossy(&json).to_string();
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let rust_code = pjson_to_rust_code(&json_value);

    println!("{}", rust_code)
}

fn write_element(value: &serde_json::Value, field_name: &str, output: &mut String, indent: &str) {
    if let Some(obj) = value.as_object() {
        let etype = obj.get("type").and_then(|v| v.as_str()).unwrap_or("div");
        let mut inner_indent = indent.to_string();

        match etype {
            "div" => write_div(obj, field_name, output, indent, &mut inner_indent),
            "label" => write_label(obj, field_name, output, indent, &mut inner_indent),
            "text_input" => write_text_input(obj, field_name, output, indent, &mut inner_indent),
            "button" => write_button(obj, field_name, output, indent, &mut inner_indent),
            "fn" => write_fn_call(obj, field_name, output, indent, &mut inner_indent),
            _ => {
                writeln!(output, "{}// Unsupported element type: {}", indent, etype).unwrap();
                writeln!(output, "{}div() // fallback", indent).unwrap();
            }
        }

        write_children(obj, output, &inner_indent);
        writeln!(output, "{}", indent).unwrap(); // newline after element
    } else {
        writeln!(
            output,
            "{}// Invalid element format: expected object",
            indent
        )
        .unwrap();
    }
}

fn write_div(
    obj: &serde_json::Map<String, serde_json::Value>,
    _field_name: &str,
    output: &mut String,
    indent: &str,
    inner_indent: &mut String,
) {
    write!(output, "{}div()", indent).unwrap();

    write_common_attrs(obj, output, inner_indent);
}

fn write_label(
    obj: &serde_json::Map<String, serde_json::Value>,
    _field_name: &str,
    output: &mut String,
    indent: &str,
    inner_indent: &mut String,
) {
    let label_content = if let Some(bind) = obj.get("bind").and_then(|v| v.as_str()) {
        format!("&self.{}", bind)
    } else if let Some(text) = obj.get("label").and_then(|v| v.as_str()) {
        format!("\"{}\"", text.escape_default())
    } else {
        "\"\"".to_string()
    };

    write!(output, "{}label::Label::new({})", indent, label_content).unwrap();

    write_common_attrs(obj, output, inner_indent);
}

fn write_text_input(
    obj: &serde_json::Map<String, serde_json::Value>,
    _field_name: &str,
    output: &mut String,
    indent: &str,
    inner_indent: &mut String,
) {
    if let Some(bind) = obj.get("bind").and_then(|v| v.as_str()) {
        write!(output, "{}input::TextInput::new(&self.{})", indent, bind).unwrap();
    } else {
        write!(
            output,
            "{}input::TextInput::new(/* missing bind */)",
            indent
        )
        .unwrap();
    }
    write_common_attrs(obj, output, inner_indent);
}

fn write_button(
    obj: &serde_json::Map<String, serde_json::Value>,
    _field_name: &str,
    output: &mut String,
    indent: &str,
    inner_indent: &mut String,
) {
    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("");
    write!(output, "{}button::Button::new(\"{}\")", indent, id).unwrap();

    if let Some(label) = obj.get("label").and_then(|v| v.as_str()) {
        write!(output, ".label(\"{}\")", label.escape_default()).unwrap();
    }
    write_common_attrs(obj, output, inner_indent);
    if let Some(on_click) = obj.get("on_click").and_then(|v| v.as_str()) {
        write!(output, r#".on_click(self.{}())"#, on_click).unwrap();
    }
}

fn write_fn_call(
    obj: &serde_json::Map<String, serde_json::Value>,
    _field_name: &str,
    output: &mut String,
    indent: &str,
    inner_indent: &mut String,
) {
    if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
        write!(output, "{}(self.{}())(self, cx)", indent, name).unwrap();
    } else {
        write!(output, "{}// Missing 'name' for fn call", indent).unwrap();
    }
    *inner_indent = format!("{}    ", indent);
    write_common_attrs(obj, output, inner_indent);
}

fn write_children(
    obj: &serde_json::Map<String, serde_json::Value>,
    output: &mut String,
    indent: &str,
) {
    if let Some(children) = obj.get("children").and_then(|v| v.as_array()) {
        for child in children {
            if let Some(obj) = child.as_object() {
                let etype = obj.get("type").and_then(|v| v.as_str()).unwrap_or("div");
                write!(output, "\n{}    .child(", indent).unwrap();

                write_element(child, etype, output, "");
                write!(output, ")").unwrap();
            } else if let Some(text) = child.as_str() {
                write!(
                    output,
                    "\n{}    .child(label::Label::new(\"{}\"))",
                    indent,
                    text.escape_default()
                )
                .unwrap();
            }
        }
    }
}

fn write_common_attrs(
    obj: &serde_json::Map<String, serde_json::Value>,
    output: &mut String,
    indent: &str,
) {
    if let Some(class) = obj.get("class").and_then(|v| v.as_str()) {
        write!(output, r#".class("{}", ld)"#, class).unwrap();
    }

    if let Some(style) = obj.get("style") {
        writeln!(
            output,
            "{}    .apply_style_rule(&/* style rule */ serde_json::from_value(json!({})).unwrap())",
            indent, style
        )
        .unwrap();
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
