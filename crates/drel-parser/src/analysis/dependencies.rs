//! Build dependency graphs from dREL AST
//!
//! This module provides functions for building dependency graphs that
//! show which data items depend on which other items.

use super::references::extract_references;
use crate::ast::{Span, Stmt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A dependency graph showing relationships between data items
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Map from item name to the items it depends on
    dependencies: HashMap<String, HashSet<String>>,
    /// Map from item name to the items that depend on it
    dependents: HashMap<String, HashSet<String>>,
    /// Track where each dependency was declared: (from, to) -> all spans where it appears
    dependency_spans: HashMap<(String, String), Vec<Span>>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a dependency: `item` depends on `dependency`
    pub fn add_dependency(&mut self, item: impl Into<String>, dependency: impl Into<String>) {
        let item = item.into();
        let dependency = dependency.into();

        self.dependencies
            .entry(item.clone())
            .or_default()
            .insert(dependency.clone());

        self.dependents.entry(dependency).or_default().insert(item);
    }

    /// Add a dependency with source location tracking
    pub fn add_dependency_with_span(
        &mut self,
        item: impl Into<String>,
        dependency: impl Into<String>,
        span: Span,
    ) {
        let item = item.into();
        let dependency = dependency.into();

        self.dependencies
            .entry(item.clone())
            .or_default()
            .insert(dependency.clone());

        self.dependents
            .entry(dependency.clone())
            .or_default()
            .insert(item.clone());

        self.dependency_spans
            .entry((item, dependency))
            .or_default()
            .push(span);
    }

    /// Get all items that `item` depends on
    pub fn get_dependencies(&self, item: &str) -> Option<&HashSet<String>> {
        self.dependencies.get(item)
    }

    /// Get all items that depend on `item`
    pub fn get_dependents(&self, item: &str) -> Option<&HashSet<String>> {
        self.dependents.get(item)
    }

    /// Get spans where a specific dependency is declared
    pub fn get_dependency_spans(&self, from: &str, to: &str) -> Option<&Vec<Span>> {
        self.dependency_spans
            .get(&(from.to_string(), to.to_string()))
    }

    /// Get all items in the graph
    pub fn all_items(&self) -> HashSet<&str> {
        let mut items = HashSet::new();
        for key in self.dependencies.keys() {
            items.insert(key.as_str());
        }
        for key in self.dependents.keys() {
            items.insert(key.as_str());
        }
        items
    }

    /// Check if the graph has any cycles
    ///
    /// Returns Some(cycle) if a cycle is found, None otherwise
    pub fn find_cycle(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for item in self.dependencies.keys() {
            if !visited.contains(item) {
                if let Some(cycle) = self.dfs_cycle(item, &mut visited, &mut rec_stack, &mut path) {
                    return Some(cycle);
                }
            }
        }

        None
    }

    /// Find a cycle and return it with span information
    ///
    /// Returns Some((cycle, spans)) where spans are the locations where
    /// each dependency in the cycle is declared.
    pub fn find_cycle_with_spans(&self) -> Option<(Vec<String>, Vec<Span>)> {
        let cycle = self.find_cycle()?;

        // Collect spans for each edge in the cycle
        let mut spans = Vec::new();
        for i in 0..cycle.len() - 1 {
            let from = &cycle[i];
            let to = &cycle[i + 1];
            if let Some(edge_spans) = self.get_dependency_spans(from, to) {
                if let Some(first_span) = edge_spans.first() {
                    spans.push(*first_span);
                }
            }
        }

        Some((cycle, spans))
    }

    fn dfs_cycle(
        &self,
        item: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(item.to_string());
        rec_stack.insert(item.to_string());
        path.push(item.to_string());

        if let Some(deps) = self.dependencies.get(item) {
            for dep in deps {
                if !visited.contains(dep) {
                    if let Some(cycle) = self.dfs_cycle(dep, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(dep) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|x| x == dep).unwrap();
                    let mut cycle: Vec<String> = path[cycle_start..].to_vec();
                    cycle.push(dep.clone());
                    return Some(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(item);
        None
    }

    /// Get a topological sort of the items (if no cycles exist)
    ///
    /// Returns items in order such that dependencies come before dependents
    pub fn topological_sort(&self) -> Result<Vec<String>, Vec<String>> {
        if let Some(cycle) = self.find_cycle() {
            return Err(cycle);
        }

        let mut result = Vec::new();
        let mut visited = HashSet::new();

        for item in self.dependencies.keys() {
            self.topo_visit(item, &mut visited, &mut result);
        }

        // Also include items that are only dependents (no dependencies of their own)
        for item in self.dependents.keys() {
            if !visited.contains(item) {
                result.push(item.clone());
            }
        }

        Ok(result)
    }

    fn topo_visit(&self, item: &str, visited: &mut HashSet<String>, result: &mut Vec<String>) {
        if visited.contains(item) {
            return;
        }

        visited.insert(item.to_string());

        if let Some(deps) = self.dependencies.get(item) {
            for dep in deps {
                self.topo_visit(dep, visited, result);
            }
        }

        result.push(item.to_string());
    }

    /// Merge another dependency graph into this one
    pub fn merge(&mut self, other: &DependencyGraph) {
        for (item, deps) in &other.dependencies {
            for dep in deps {
                self.add_dependency(item.clone(), dep.clone());
            }
        }
        // Also merge spans
        for ((from, to), spans) in &other.dependency_spans {
            for span in spans {
                self.dependency_spans
                    .entry((from.clone(), to.clone()))
                    .or_default()
                    .push(*span);
            }
        }
    }
}

/// Build a dependency graph from a dREL method
///
/// The `target_item` is the item being defined by this method, and the
/// method body determines what it depends on.
///
/// # Example
///
/// ```rust,ignore
/// use drel_parser::{parse, analysis::build_dependency_graph};
///
/// let stmts = parse("_crystal.density = _cell.atomic_mass / _cell.volume")?;
/// let graph = build_dependency_graph("_crystal.density", &stmts);
///
/// // _crystal.density depends on _cell.atomic_mass and _cell.volume
/// let deps = graph.get_dependencies("_crystal.density").unwrap();
/// assert!(deps.contains("_cell.atomic_mass"));
/// assert!(deps.contains("_cell.volume"));
/// ```
pub fn build_dependency_graph(target_item: &str, stmts: &[Stmt]) -> DependencyGraph {
    let mut graph = DependencyGraph::new();
    let refs = extract_references(stmts);

    for item_ref in refs {
        // Only add data name references as dependencies
        if item_ref.is_data_name() {
            let ref_name = item_ref.full_name();
            // Don't add self-reference as dependency
            if ref_name != target_item {
                graph.add_dependency_with_span(target_item, ref_name, item_ref.span);
            }
        }
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_build_graph() {
        let stmts = parse("_crystal.density = _cell.atomic_mass / _cell.volume").unwrap();
        let graph = build_dependency_graph("_crystal.density", &stmts);

        let deps = graph.get_dependencies("_crystal.density").unwrap();
        assert!(deps.contains("_cell.atomic_mass"));
        assert!(deps.contains("_cell.volume"));
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "b");
        graph.add_dependency("b", "c");
        graph.add_dependency("c", "a"); // Creates cycle: a -> b -> c -> a

        let cycle = graph.find_cycle();
        assert!(cycle.is_some());
    }

    #[test]
    fn test_no_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "b");
        graph.add_dependency("b", "c");
        graph.add_dependency("a", "c");

        assert!(graph.find_cycle().is_none());
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "b");
        graph.add_dependency("b", "c");

        let sorted = graph.topological_sort().unwrap();
        // c should come before b, b before a
        let c_pos = sorted.iter().position(|x| x == "c").unwrap();
        let b_pos = sorted.iter().position(|x| x == "b").unwrap();
        let a_pos = sorted.iter().position(|x| x == "a").unwrap();
        assert!(c_pos < b_pos);
        assert!(b_pos < a_pos);
    }

    #[test]
    fn test_dependency_spans() {
        let stmts = parse("_crystal.density = _cell.atomic_mass / _cell.volume").unwrap();
        let graph = build_dependency_graph("_crystal.density", &stmts);

        // Check that we have spans for the dependencies
        let spans = graph.get_dependency_spans("_crystal.density", "_cell.atomic_mass");
        assert!(spans.is_some());
        let spans = spans.unwrap();
        assert!(!spans.is_empty());
        // The span should point to a valid location
        assert!(spans[0].start_line > 0 || spans[0].start_col > 0);
    }
}
