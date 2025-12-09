use std::fmt::Write;

// Add this as a new function in your file
pub fn pjson_to_rust_code(value: &serde_json::Value) -> String {
    let mut output = String::new();
    let indent = "    ".to_string();
    write_element(value, "", &mut output, &indent);
    writeln!(output, "\n\n//generated code end").unwrap();
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
    write!(output, "\n{}div()", indent).unwrap();
    write_common_attrs(obj, output, inner_indent);
    *inner_indent = format!("{}    ", indent);
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

    write!(output, "label::Label::new({})", label_content).unwrap();

    write_common_attrs(obj, output, inner_indent);
    *inner_indent = format!("{}    ", indent);
}

fn write_text_input(
    obj: &serde_json::Map<String, serde_json::Value>,
    _field_name: &str,
    output: &mut String,
    indent: &str,
    inner_indent: &mut String,
) {
    if let Some(bind) = obj.get("bind").and_then(|v| v.as_str()) {
        write!(output, "input::TextInput::new(&self.{})", bind).unwrap();
    } else {
        write!(output, "input::TextInput::new(/* missing bind */)").unwrap();
    }
    write_common_attrs(obj, output, inner_indent);
    *inner_indent = format!("{}    ", indent);
}

fn write_button(
    obj: &serde_json::Map<String, serde_json::Value>,
    _field_name: &str,
    output: &mut String,
    indent: &str,
    inner_indent: &mut String,
) {
    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("");
    write!(output, "button::Button::new(\"{}\")", id).unwrap();

    if let Some(label) = obj.get("label").and_then(|v| v.as_str()) {
        write!(output, ".label(\"{}\")", label.escape_default()).unwrap();
    }
    write_common_attrs(obj, output, inner_indent);
    if let Some(on_click) = obj.get("on_click").and_then(|v| v.as_str()) {
        write!(output, r#".on_click(Self::{}(cx))"#, on_click).unwrap();
    }
    *inner_indent = format!("{}    ", indent);
}

fn write_fn_call(
    obj: &serde_json::Map<String, serde_json::Value>,
    _field_name: &str,
    output: &mut String,
    indent: &str,
    inner_indent: &mut String,
) {
    if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
        write!(output, "(self.{}())(self, cx)", name).unwrap();
    } else {
        writeln!(output, "{}// Missing 'name' for fn call", indent).unwrap();
    }
    *inner_indent = format!("{}    ", indent);
    write_common_attrs(obj, output, inner_indent);
    *inner_indent = format!("{}    ", indent);
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
                write!(output, "\n{}.child(", indent).unwrap();
                let child_indent = format!("{}    ", indent);
                write_element(child, etype, output, &child_indent);
                if etype == "div" {
                    write!(output, "\n{}    )", indent).unwrap();
                } else {
                    write!(output, ")").unwrap();
                }
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
    _indent: &str,
) {
    if let Some(class) = obj.get("class").and_then(|v| v.as_str()) {
        write!(output, r#".class("{}", sd)"#, class).unwrap();
    }

    if let Some(style) = obj.get("style") {
        write!(output, ".apply_style_rule_json(json!({}))", style).unwrap();
    }
}
