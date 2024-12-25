// convert an array selector to a readable string
pub fn display_array_selector(capitalized: bool) -> String {
    String::from(if capitalized { "Array" } else { "array" })
}

// convert a range to a readable string.
pub fn display_range_selector(
    (start, end): (Option<usize>, Option<usize>),
    capitalized: bool,
) -> String {
    let position_to_string = |position: Option<usize>| -> String {
        position.map_or_else(|| String::from(""), |value| value.to_string())
    };
    let (start, end) = (position_to_string(start), position_to_string(end));
    [
        if capitalized { "Range [" } else { "range [" },
        start.to_string().as_str(),
        ":",
        end.to_string().as_str(),
        "]",
    ]
    .join("")
}
// convert a range to a readable string.
pub fn display_default_selector(value: &str, capitalized: bool) -> String {
    [
        if capitalized {
            r#"Node ""#
        } else {
            r#"node ""#
        },
        value,
        r#"""#,
    ]
    .join("")
}

pub fn display_index_selector(indexes: &[usize], capitalized: bool) -> String {
    if indexes.len() == 1 {
        [
            if capitalized { "Index [" } else { "index [" },
            indexes[0].to_string().as_str(),
            "]",
        ]
        .join("")
    } else {
        [
            if capitalized { "Index [" } else { "index [" },
            indexes
                .iter()
                .map(|index| index.to_string())
                .collect::<Vec<String>>()
                .join("")
                .as_str(),
            "]",
        ]
        .join("")
    }
}

pub fn display_object_selector(
    properties: &[String],
    capitalized: bool,
) -> String {
    if properties.len() == 1 {
        [
            if capitalized {
                "Property {"
            } else {
                "property {"
            },
            properties[0].to_string().as_str(),
            "}",
        ]
        .join("")
    } else {
        [
            if capitalized {
                "Property {"
            } else {
                "property {"
            },
            properties.join(",").as_str(),
            "}",
        ]
        .join("")
    }
}
