use serde_json::Value;
use structs::{matches, Criterion, Item, StackItem, Step};

pub struct Iter<'a, 'b> {
    criteria: &'b [Criterion],
    ci: usize,
    current: Option<StackItem<'a>>,
    root: StackItem<'a>,
    stack: Vec<StackItem<'a>>,
}

pub struct Found<'a> {
    pub value: &'a Value,
    pub path: Vec<Step<'a>>,
}

impl<'a, 'b> Iterator for Iter<'a, 'b> {
    type Item = Found<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut current) = self.current.take() {
            if let Some(criterion) = self.criteria.get(self.ci) {
                if matches(&mut current, criterion, &self.root) {
                    // if there are no further criteria
                    if self.criteria.len() == self.ci + 1 {
                        let val = current.item.value;
                        // Hack to prevent overflow
                        if self.ci > 0 {
                            self.ci -= 1;
                        }
                        let mut path = self.stack.iter()
                            .map(|item| item.step.clone())
                            .collect::<Vec<_>>();
                        path.push(current.step);
                        self.current = self.stack.pop();
                        return Some(Found {value: val, path: path});
                    } else {
                        self.current = current.next();
                        self.ci += 1;
                        self.stack.push(current);

                        if self.current.is_none() {
                            self.ci -= 1;
                            self.stack.pop();

                            // Hack to prevent overflow
                            if self.ci > 0 {
                                self.ci -= 1;
                            }
                            self.current = self.stack.pop();
                        }
                    }
                } else if !self.stack.is_empty() {
                    // the step and criterion do not match
                    match self.stack.last_mut().unwrap().next() {
                        Some(new_cur) => self.current = Some(new_cur),
                        None => {
                            self.ci -= 1;
                            self.current = self.stack.pop();
                        }
                    }
                }
            } else {
                // This must be unreachable, because we look forward for empty criteria in
                //    if self.criteria.len() == self.ci + 1 {
                unreachable!();
            }
        }
        None
    }
}

impl<'a, 'b> Iter<'a, 'b> {
    pub fn new(root: &'a Value, criteria: &'b [Criterion]) -> Self {
        let root_item = Item::new(root);
        let step = Step::Root;
        let root = StackItem::new(Item::new(root), Step::Root);
        let current = Some(StackItem::new(root_item, step));

        Self {
            criteria,
            current,
            root,
            stack: vec![],
            ci: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_simple_json() {
        let json = r#"
            {
                "dog": {
                    "name": "Rex"
                }
            }
        "#;

        let root: Value = serde_json::from_str(&json).unwrap();
        let criteria = vec![
            Criterion::Root,
            Criterion::NamedChild("dog".to_owned()),
            Criterion::NamedChild("name".to_owned()),
        ];

        let found: Vec<&Value> = Iter::new(&root, &criteria)
            .map(|v| v.value)
            .collect();
        assert_eq!(found, vec!["Rex"]);
    }

    #[test]
    fn test_complex_json() {
        let json = r#"
            {
                "pets": [
                    {
                        "type":"cat",
                        "name":"Tom"
                    },
                    {
                        "type":"dog",
                        "name":"Rex"
                    }
                ],
                "user": {
                    "name":"Sergey",
                    "age":27
                }
            }
        "#;

        let root: Value = serde_json::from_str(&json).unwrap();

        // $.user.age
        let criteria = vec![
            Criterion::Root,
            Criterion::NamedChild("user".to_owned()),
            Criterion::NamedChild("age".to_owned()),
        ];
        let found: Vec<&Value> = Iter::new(&root, &criteria)
            .map(|v| v.value)
            .collect();
        assert_eq!(found, vec![27]);

        // $.pets.*.type
        let criteria = vec![
            Criterion::Root,
            Criterion::NamedChild("pets".to_owned()),
            Criterion::AnyChild,
            Criterion::NamedChild("type".to_owned()),
        ];
        let found: Vec<&Value> = Iter::new(&root, &criteria)
            .map(|v| v.value)
            .collect();
        assert_eq!(found, vec!["cat", "dog"]);

        // $.pets.*.name
        let criteria = vec![
            Criterion::Root,
            Criterion::NamedChild("pets".to_owned()),
            Criterion::AnyChild,
            Criterion::NamedChild("name".to_owned()),
        ];
        let found: Vec<&Value> = Iter::new(&root, &criteria)
            .map(|v| v.value)
            .collect();
        assert_eq!(found, vec!["Tom", "Rex"]);

        // $.user.*
        let criteria = vec![
            Criterion::Root,
            Criterion::NamedChild("user".to_owned()),
            Criterion::AnyChild,
        ];
        let found: Vec<&Value> = Iter::new(&root, &criteria)
            .map(|v| v.value)
            .collect();
        assert_eq!(found, vec![&Value::from(27), &Value::from("Sergey")]);
    }

    #[test]
    fn test_any_child() {
        let json = r#"
            {
                "pet": {
                    "type": "dog",
                    "name": "Rex"
                },
                "car": {
                    "type": "passenger",
                    "name": "Zorro"
                }
            }
        "#;

        let root: Value = serde_json::from_str(&json).unwrap();
        let criteria = vec![
            Criterion::Root,
            Criterion::NamedChild("pet".to_owned()),
            Criterion::AnyChild,
        ];

        let found: Vec<&Value> = Iter::new(&root, &criteria)
            .map(|v| v.value)
            .collect();
        assert_eq!(found, vec!["Rex", "dog"]);
    }

    #[test]
    fn test_indexed_child() {
        let json = r#"
            ["Foo", "Bar", "Baz"]
        "#;

        let root: Value = serde_json::from_str(&json).unwrap();
        let criteria = vec![Criterion::Root, Criterion::IndexedChild(1)];

        let found: Vec<&Value> = Iter::new(&root, &criteria)
            .map(|v| v.value)
            .collect();
        assert_eq!(found, vec!["Bar"]);
    }
}
