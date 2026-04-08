use dialoguer::{Input, Select, theme::ColorfulTheme};
use jsonschema::ValidationError;
use serde_json::{Value, json};

#[derive(Debug)]
pub enum InputError {
    DialoguerError(dialoguer::Error),
    ValidationErrors(Vec<ValidationError<'static>>),
}

fn to_case(s: String, case: &str) -> String {
    match case {
        "camelCase" => {
            let mut result = String::new();
            let mut capitalize_next = false;

            for c in s.chars() {
                if c.is_whitespace() || c == '_' || c == '-' {
                    capitalize_next = true;
                } else if capitalize_next {
                    result.push(c.to_ascii_uppercase());
                    capitalize_next = false;
                } else {
                    result.push(c.to_ascii_lowercase());
                }
            }

            result
        }
        "PascalCase" => {
            let mut result = String::new();
            let mut capitalize_next = true;

            for c in s.chars() {
                if c.is_whitespace() || c == '_' || c == '-' {
                    capitalize_next = true;
                } else if capitalize_next {
                    result.push(c.to_ascii_uppercase());
                    capitalize_next = false;
                } else {
                    result.push(c.to_ascii_lowercase());
                }
            }

            result
        }
        "snake_case" => s.replace(' ', "_").replace('-', "_").to_ascii_lowercase(),
        "kebab-case" => s.replace(' ', "-").replace('_', "-").to_ascii_lowercase(),
        "UPPER_CASE" => s.replace(' ', "_").replace('-', "_").to_ascii_uppercase(),
        _ => s.to_string(),
    }
}

pub fn handle_inputs(schema_json: &Value) -> Result<Value, InputError> {
    // Compile the JSON schema
    let compiled_schema = match jsonschema::validator_for(&schema_json) {
        Ok(schema) => schema,
        Err(err) => return Err(InputError::ValidationErrors(vec![err])),
    };

    let mut instance = json!({});

    let casing_options = [
        "camelCase",
        "PascalCase",
        "snake_case",
        "kebab-case",
        "UPPER_CASE",
    ];

    // Get properties from schema and prompt user for each
    if let Some(obj) = schema_json.as_object() {
        if let Some(properties) = obj.get("properties") {
            for (key, property) in properties.as_object().unwrap() {
                let prop_type = property
                    .get("type")
                    .and_then(|d| d.as_str())
                    .unwrap_or("string");

                let default_prompt = format!("Enter value for {}:", key);
                let prompt = property
                    .get("x-prompt")
                    .and_then(|d| d.as_str())
                    .unwrap_or(default_prompt.as_str());

                let theme = &ColorfulTheme::default();

                match prop_type {
                    "string" => {
                        let value = handle_string_property(property, prompt, theme)?;

                        instance[key] = Value::String(value.clone());

                        let casing = property
                            .get("x-casing")
                            .and_then(|d| d.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .filter(|s| casing_options.contains(s))
                                    .collect::<Vec<&str>>()
                            })
                            .unwrap_or_else(|| vec![]);

                        for case in casing {
                            let cased_value = to_case(value.clone(), case);
                            let cased_key = format!("{}_{}", key, case);

                            instance[cased_key] = Value::String(cased_value);
                        }
                    }
                    "integer" => {
                        let value = handle_integer_property(property, prompt, theme)?;

                        instance[key] =
                            Value::Number(serde_json::Number::from_i128(value.into()).unwrap());
                    }
                    "number" => {
                        let value = handle_number_property(property, prompt, theme)?;

                        instance[key] = Value::Number(serde_json::Number::from_f64(value).unwrap());
                    }
                    "boolean" => {
                        let value = handle_boolean_property(property, prompt, theme)?;

                        instance[key] = Value::Bool(value);
                        continue;
                    }
                    _ => {}
                }
            }
        }
    }

    // Validate the instance against the schema
    match compiled_schema.validate(&instance) {
        Ok(_) => {}
        Err(_) => {
            return Err(InputError::ValidationErrors(
                compiled_schema
                    .iter_errors(&instance)
                    .map(|e| e.to_owned())
                    .collect(),
            ));
        }
    }

    Ok(instance)
}

fn handle_enum_property(
    property: &Value,
    prop_desc: &str,
    theme: &ColorfulTheme,
) -> Result<String, InputError> {
    let options = property
        .get("enum")
        .and_then(|d| {
            d.as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>())
        })
        .unwrap_or_else(|| vec![]);

    let prop_default = property
        .get("default")
        .and_then(|d| d.as_str())
        .unwrap_or("");

    let mut select = Select::with_theme(theme)
        .with_prompt(prop_desc)
        .items(&options);

    if !prop_default.is_empty() {
        let default_index = options.iter().position(|&r| r == prop_default).unwrap_or(0);

        select = select.default(default_index);
    }

    let selection = match select.interact() {
        Ok(index) => index,
        Err(err) => return Err(InputError::DialoguerError(err)),
    };

    let value = options[selection].to_string();

    Ok(value)
}

fn handle_string_property(
    property: &Value,
    prop_desc: &str,
    theme: &ColorfulTheme,
) -> Result<String, InputError> {
    if property.get("enum").is_some() {
        return handle_enum_property(property, prop_desc, theme);
    }

    let prop_default = property
        .get("default")
        .and_then(|d| d.as_str())
        .unwrap_or("");

    let mut input = Input::<String>::with_theme(theme).with_prompt(prop_desc);
    if !prop_default.is_empty() {
        input = input.default(prop_default.to_string());
    }

    let value: String = input.interact_text().unwrap_or(prop_default.to_string());

    Ok(value)
}

fn handle_integer_property(
    property: &Value,
    prop_desc: &str,
    theme: &ColorfulTheme,
) -> Result<i64, InputError> {
    let prop_default = property
        .get("default")
        .and_then(|d| d.as_i64())
        .unwrap_or(0);

    let mut input = Input::<i64>::with_theme(theme).with_prompt(prop_desc);
    if prop_default != 0 {
        input = input.default(prop_default);
    }

    let value: i64 = input.interact_text().unwrap_or(prop_default);

    Ok(value)
}

fn handle_number_property(
    property: &Value,
    prop_desc: &str,
    theme: &ColorfulTheme,
) -> Result<f64, InputError> {
    let prop_default = property
        .get("default")
        .and_then(|d| d.as_f64())
        .unwrap_or(0.0);

    let mut input = Input::<f64>::with_theme(theme).with_prompt(prop_desc);
    if prop_default != 0.0 {
        input = input.default(prop_default);
    }

    let value: f64 = input.interact_text().unwrap_or(prop_default);

    Ok(value)
}

fn handle_boolean_property(
    property: &Value,
    prop_desc: &str,
    theme: &ColorfulTheme,
) -> Result<bool, InputError> {
    let prop_default = property
        .get("default")
        .and_then(|d| d.as_bool())
        .unwrap_or(false);

    let value = Input::<bool>::with_theme(theme)
        .with_prompt(prop_desc)
        .default(prop_default)
        .interact_text()
        .unwrap_or(prop_default);

    Ok(value)
}
