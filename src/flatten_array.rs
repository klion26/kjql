use serde_json::Value;

pub fn flatten_array(array: &[Value]) -> Vec<Value> {
    array
        .iter()
        .fold(Vec::with_capacity(array.len()), |mut acc, value| {
            if value.is_array() {
                value.as_array().unwrap().iter().for_each(|i| {
                    if i.is_array() {
                        i.as_array().unwrap().iter().for_each(|j| {
                            println!("{:?}", j);
                            acc.push(j.clone());
                        })
                    } else {
                        println!("asdasd");
                        acc.push(i.clone())
                    }
                });
                println!("---=== {:?}", acc);
                acc
            } else {
                println!("Nope");
                acc.push(value.clone());
                acc
            }
        })
}
