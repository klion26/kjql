#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::walker;
    use serde_json::{json, Value};
    const ARRAY_DATA: &str = r#"[1, 2, 3]"#;
    const DATA: &str = r#"{
       "array": [1, 2, 3],
       "nested": {
           "a": "one",
           "b": "two",
           "c": "three"
       },
       "number": 1337,
       "text": "some text",
       ".property..": "This is valid JSON!",
       "\"": "This is valid JSON as well",
       " ": "Yup, this too 🐼!",
       "": "Yup, again 🐨!",
       "mix": [{"first": 1}],
       "range": [1, 2, 3, 4, 5, 6, 7],
       "filter": [
            { "color": "red" },
            { "color": "green" },
            { "color": "blue" }
       ]
       }
    "#;

    #[test]
    fn get_test() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("text");
        assert_eq!(Ok(json["text"].clone()), walker(&json, selector));
    }

    #[test]
    fn get_number() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("number");
        assert_eq!(Ok(json["number"].clone()), walker(&json, selector));
    }

    #[test]
    fn get_array() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector: Option<&str> = Some("array");
        assert_eq!(Ok(json["array"].clone()), walker(&json, selector));
    }

    #[test]
    fn get_array_item() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("array.0");
        assert_eq!(Ok(json["array"][0].clone()), walker(&json, selector));
    }

    #[test]
    fn get_out_of_bound_item_in_array() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("array.3");
        assert_eq!(
            Err(String::from(
                "Index ( 3 ) is out of bound, node ( array ) has a length of 3"
            )),
            walker(&json, selector)
        );
    }

    #[test]
    fn get_out_of_bound_item_in_root_array() {
        let json_array: Value = serde_json::from_str(ARRAY_DATA).unwrap();
        let array_selector = Some("3");
        assert_eq!(
            Err(String::from(
                "Index ( 3 ) is out of bound, root elment has a length of 3"
            )),
            walker(&json_array, array_selector)
        );
    }

    #[test]
    fn get_negative_index_in_array() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("array.-1");
        assert_eq!(
            Err(String::from("Invalid negative array index")),
            walker(&json, selector)
        );
    }

    #[test]
    fn get_negative_index_in_root_array() {
        let json_array: Value = serde_json::from_str(ARRAY_DATA).unwrap();
        let array_selector = Some("-1");
        assert_eq!(
            Err(String::from("Invalid negative array index")),
            walker(&json_array, array_selector)
        );
    }
    #[test]
    fn get_index_in_non_array() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("text.1");
        assert_eq!(
            Err(String::from("Node ( text ) is not an array")),
            walker(&json, selector)
        );
        let root_selector = Some("1");
        assert_eq!(
            Err(String::from("Root element is not an array")),
            walker(&json, root_selector)
        );
    }

    #[test]
    fn get_non_existing_root_node() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("foo");
        assert_eq!(
            Err(String::from("Node ( foo ) is not the root element")),
            walker(&json, selector)
        );
    }

    #[test]
    fn get_non_existing_child_node() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("nested.d");
        assert_eq!(
            Err(String::from(
                "Node ( d ) not found on parent node ( nested )"
            )),
            walker(&json, selector)
        );
    }

    #[test]
    fn get_existing_child_node() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("nested.a");
        assert_eq!(Ok(json["nested"]["a"].clone()), walker(&json, selector));
    }

    #[ignore]
    #[test]
    fn get_unterminated_selector() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("nested.");
        assert_eq!(
            Err(String::from("Unterminated selector found")),
            walker(&json, selector)
        );
    }

    #[test]
    fn get_weird_json() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let dot_selector = Some(r#"".property..""#);
        let quote_selector = Some(r#"""""#);
        let space_selector = Some(r#"" ""#);
        let empty_selector = Some(r#""""#);
        assert_eq!(
            Ok(json[".property.."].clone()),
            walker(&json, dot_selector)
        );
        assert_eq!(Ok(json[r#"""#].clone()), walker(&json, quote_selector));
        assert_eq!(Ok(json[r#" "#].clone()), walker(&json, space_selector));
        assert_eq!(Ok(json[r#""#].clone()), walker(&json, empty_selector));
    }

    #[test]
    fn get_mix_json() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let mix_selector = Some("mix.0.first");
        assert_eq!(
            Ok(json["mix"][0]["first"].clone()),
            walker(&json, mix_selector)
        )
    }

    #[test]
    fn get_range() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let range_selector = Some("range.2:5");
        assert_eq!(Ok(json!([3, 4, 5, 6])), walker(&json, range_selector));
    }

    #[test]
    fn get_one_item_range() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("range.2:2");
        assert_eq!(Ok(json!([3])), walker(&json, selector));
    }

    #[test]
    fn get_reversed_range() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("range.5:2");
        assert_eq!(Ok(json!([6, 5, 4, 3])), walker(&json, selector));
    }

    #[test]
    fn get_original_from_reversed_range() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("range.5:2.3:0");

        assert_eq!(Ok(json!([3, 4, 5, 6])), walker(&json, selector));
    }

    #[test]
    fn get_out_of_bound_range() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("range.6:7");

        assert_eq!(
            Err(String::from(
                "Range ( 6 : 7 ) is out of bound, node ( range ) has a length \
                 of 7"
            )),
            walker(&json, selector)
        );
    }

    #[test]
    fn get_multi_selection() {
        let json: Value = serde_json::from_str(DATA).unwrap();

        let selector = Some("array,number");
        assert_eq!(
            Ok(json!([json["array"], json["number"]])),
            walker(&json, selector)
        );
    }

    #[test]
    fn get_filter() {
        let json: Value = serde_json::from_str(DATA).unwrap();

        let selector = Some("filter|color");
        assert_eq!(Ok(json!(["red", "green", "blue"])), walker(&json, selector))
    }

    #[test]
    fn get_filter_with_range() {
        let json: Value = serde_json::from_str(DATA).unwrap();

        let selector = Some("filter.1:2|color");
        assert_eq!(Ok(json!(["green", "blue"])), walker(&json, selector));
    }

    #[test]
    fn get_filter_with_multi_selection() {
        let json: Value = serde_json::from_str(DATA).unwrap();

        let selector = Some("filter, filter.1:2|color");
        assert_eq!(
            Ok(json!([["red", "green", "blue"], ["green", "blue"]])),
            walker(&json, selector)
        )
    }

    #[test]
    fn get_wrong_filter() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("filter|colors");
        assert_eq!(
            Err(String::from("Node ( colors ) is not the root element")),
            walker(&json, selector)
        )
    }

    #[test]
    fn get_wrong_filter_with_range() {
        let json: Value = serde_json::from_str(DATA).unwrap();
        let selector = Some("filter.1:2|colors");
        assert_eq!(
            Err(String::from("Node ( colors ) is not the root element")),
            walker(&json, selector)
        )
    }
}
