# JSONPath for Rust

The library is in hard development stage.


## Example

```rust
extern crate jsonpath;
extern crate serde_json;

use jsonpath::Selector;
use serde_json::Value;

fn main() {
    let jsondoc = r#"
        {
            "favorites": {
               "books": [
                   {
                       "title": "Der schwarze Obelist",
                       "author": "Erich Maria Remarque"
                   },
                   {
                       "title": "Le mur",
                       "author": "Jean-Paul Sartre"
                   }
               ]
            }
        }
    "#;

    // Parse JSON document
    let json: Value = serde_json::from_str(jsondoc).unwrap();

    // Create a JSONPath selector
    let selector = Selector::new("$.favorites..title").unwrap();

    // Apply the selector to the JSON and convert Vec<&Value> into Vec<&str>
    let titles: Vec<&str> = selector.find_all(&json)
        .iter()
        .map(|t| t.as_str().unwrap())
        .collect();
    assert_eq!(titles, vec!["Der schwarze Obelist", "Le mur"]);
}
```

## Contributors

- [greyblake](https://github.com/greyblake) Potapov Sergey - creator, maintainer.
