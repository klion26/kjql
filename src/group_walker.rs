use serde_json::{json, Value};

use crate::{
    apply_filter::apply_filter,
    flatten_json_array::flatten_json_array,
    get_selection::get_selections,
    truncate::truncate_json,
    types::{Group, MayArray, Selection},
};

// walks through a group
pub fn group_walker(
    Group {
        filters,
        filter_lenses,
        root,
        selectors,
        spread,
        truncate,
    }: &Group,
    json: &Value,
) -> Selection {
    // empty group, return early
    if selectors.is_empty() && root.is_none() {
        return Err(String::from("Empty group"));
    }

    // check for empty selection, in this case we assume that the user expects
    // to get back the complete raw JSON back for this group.
    match get_selections(selectors, json) {
        Ok(ref items) => {
            // check for an empty selection, in this case we assume that the
            // user expects to get back the complete raw JSON for
            // this group.
            let output_json = if items.is_empty() {
                json.clone()
            } else {
                json!(items.last().unwrap())
            };

            let is_spreading = spread.is_some();

            let output = match apply_filter(filters, filter_lenses, &output_json) {
                Ok(filtered) => match filtered {
                    MayArray::Array(array) => Ok(if is_spreading {
                        json!(flatten_json_array(&json!(array)))
                    } else {
                        json!(array)
                    }),
                    MayArray::NonArray(single_value) => {
                        if is_spreading {
                            Err(String::from("Only array can be flattened."))
                        } else {
                            // we know that we are holding a single value
                            // wrapped inside a MaybeArray::NoArray enum.
                            // we need to pick the first item of the vector.
                            Ok(json!(single_value[0]))
                        }
                    }
                },
                Err(error) => Err(error),
            };
            match truncate {
                Some(_) => match output {
                    Ok(value) => Ok(truncate_json(value)),
                    Err(error) => Err(error),
                },
                None => output,
            }
        }
        Err(error) => Err(error),
    }
}
