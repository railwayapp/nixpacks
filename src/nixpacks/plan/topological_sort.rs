use anyhow::{bail, Result};
use std::collections::{BTreeMap, BTreeSet};

pub trait TopItem {
    fn get_name(&self) -> String;
    fn get_dependencies(&self) -> &[String];
}

pub fn topological_sort<T>(items: Vec<T>) -> Result<Vec<T>>
where
    T: Clone + TopItem,
{
    let mut items = items;
    items.sort_by_cached_key(TopItem::get_name);

    let mut lookup = BTreeMap::<String, T>::new();
    for item in items.clone() {
        if lookup.contains_key(&item.get_name()) {
            bail!("Multiple items with the same name: {}", item.get_name());
        }

        lookup.insert(item.get_name(), item);
    }

    // Reference of name -> dependent [items]
    let mut adj_list = items.iter().fold(
        BTreeMap::<String, BTreeSet<String>>::new(),
        |mut acc, item| {
            let n = item.get_name();
            if !acc.contains_key(&n) {
                acc.insert(n.clone(), BTreeSet::new());
            }

            item.get_dependencies().iter().for_each(|dep| {
                acc.get_mut(&n).unwrap().insert(dep.clone());
            });

            acc
        },
    );

    // The number of dependencies for each item
    let mut indegree = items
        .into_iter()
        .map(|item| {
            (
                item.get_name(),
                item.get_dependencies()
                    .iter()
                    .filter(|dep| lookup.contains_key(&(*dep).clone()))
                    .count(),
            )
        })
        .collect::<BTreeMap<String, usize>>();

    let mut result: Vec<T> = Vec::new();

    while !indegree.is_empty() {
        // Get the items with no dependencies
        let (no_deps, new_indegree): (BTreeMap<String, usize>, BTreeMap<String, usize>) = indegree
            .into_iter()
            .partition(|(_name, indgree)| *indgree == 0);

        // Circular dependency
        if no_deps.is_empty() {
            bail!("Circular dependency detected");
        }

        indegree = new_indegree;

        // Add the items to the result
        result.append(
            &mut no_deps
                .keys()
                .map(|name| lookup.get(name).unwrap().clone())
                .collect::<Vec<_>>(),
        );

        // Update the indegree of the dependent items
        for (name, _) in no_deps {
            adj_list.remove(&name);

            for (dep_name, dependents) in &mut adj_list {
                if dependents.remove(&name) {
                    indegree.entry(dep_name.clone()).and_modify(|e| *e -= 1);
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::nixpacks::plan::topological_sort;

    use super::TopItem;

    #[derive(Clone, Debug)]
    struct TestItem {
        name: String,
        dependencies: Vec<String>,
    }

    impl TestItem {
        pub fn new<S: Into<String>>(name: S, dependencies: Vec<String>) -> Self {
            Self {
                name: name.into(),
                dependencies,
            }
        }
    }

    impl TopItem for TestItem {
        fn get_name(&self) -> String {
            self.name.clone()
        }

        fn get_dependencies(&self) -> &[String] {
            &self.dependencies
        }
    }

    #[test]
    fn test_sorts_graph() {
        let items = vec![
            TestItem::new("a", vec![]),
            TestItem::new("b", vec!["a".to_string()]),
            TestItem::new("c", vec!["b".to_string()]),
            TestItem::new("d", vec!["b".to_string(), "c".to_string()]),
        ];

        assert_eq!(
            topological_sort(items)
                .unwrap()
                .iter()
                .map(topological_sort::TopItem::get_name)
                .collect::<Vec<_>>(),
            vec!["a", "b", "c", "d"]
        );
    }

    #[test]
    fn test_circular_dep() {
        let items = vec![
            TestItem::new("a", vec!["b".to_string()]),
            TestItem::new("b", vec!["a".to_string()]),
        ];

        assert!(topological_sort(items).is_err());
    }
}
