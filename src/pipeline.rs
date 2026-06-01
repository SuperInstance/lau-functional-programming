//! Functional patterns for agent composition pipelines.
//! Demonstrates how functional programming concepts apply to building
//! composable agent systems.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An agent that transforms input to output.
pub trait Agent<I, O> {
    fn process(&self, input: I) -> O;
    fn name(&self) -> &str;
}

/// A simple function-based agent.
pub struct FuncAgent<I, O> {
    pub name: String,
    pub func: Option<Box<dyn Fn(I) -> O>>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I: 'static, O: 'static> FuncAgent<I, O> {
    pub fn new(name: &str, f: impl Fn(I) -> O + 'static) -> Self {
        FuncAgent {
            name: name.to_string(),
            func: Some(Box::new(f)),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<I, O> Agent<I, O> for FuncAgent<I, O> {
    fn process(&self, input: I) -> O {
        (self.func.as_ref().unwrap())(input)
    }
    fn name(&self) -> &str { &self.name }
}

/// A pipeline of agents composed in sequence.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pipeline<A, B> {
    pub stages: Vec<PipelineStage>,
    _phantom: std::marker::PhantomData<(A, B)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PipelineStage {
    Map(String),
    Filter(String),
    FlatMap(String),
    Reduce(String),
    Branch(String),
}

/// Result of a pipeline execution.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PipelineResult<T> {
    Success(T),
    Error(String),
    Skipped,
}

impl<T> PipelineResult<T> {
    pub fn is_success(&self) -> bool {
        matches!(self, PipelineResult::Success(_))
    }

    pub fn unwrap(self) -> T {
        match self {
            PipelineResult::Success(t) => t,
            _ => panic!("unwrap on non-success"),
        }
    }
}

/// Compose two agents sequentially: g ∘ f
pub fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |a| g(f(a))
}

/// Compose multiple functions into a pipeline.
pub fn compose_all<A>(funcs: Vec<Box<dyn Fn(A) -> A>>) -> impl Fn(A) -> A {
    move |mut input: A| {
        for f in &funcs {
            input = f(input);
        }
        input
    }
}

/// Kleisli composition: compose two monadic functions.
pub fn kleisli<A, B, C, M>(
    f: impl Fn(A) -> M,
    g: impl Fn(B) -> M,
) -> impl Fn(A) -> M
where
    M: crate::monad::Monad<B>,
{
    move |_a| unimplemented!("Kleisli composition requires bind")
}

/// Map over a collection.
pub fn pipeline_map<T, U>(items: Vec<T>, f: impl Fn(T) -> U) -> Vec<U> {
    items.into_iter().map(f).collect()
}

/// Filter a collection.
pub fn pipeline_filter<T>(items: Vec<T>, pred: impl Fn(&T) -> bool) -> Vec<T> {
    items.into_iter().filter(pred).collect()
}

/// FlatMap (bind for Vec).
pub fn pipeline_flatmap<T, U>(items: Vec<T>, f: impl Fn(T) -> Vec<U>) -> Vec<U> {
    items.into_iter().flat_map(f).collect()
}

/// Reduce/fold a collection.
pub fn pipeline_reduce<T, A>(items: Vec<T>, init: A, f: impl Fn(A, T) -> A) -> A {
    items.into_iter().fold(init, f)
}

/// Parallel fan-out: send input to multiple agents, collect results.
pub fn fan_out<A: Clone, B>(input: A, agents: Vec<Box<dyn Fn(A) -> B>>) -> Vec<B> {
    agents.into_iter().map(|agent| agent(input.clone())).collect()
}

/// Fan-in: merge multiple inputs into one.
pub fn fan_in<A, B>(inputs: Vec<A>, merge: impl Fn(Vec<A>) -> B) -> B {
    merge(inputs)
}

/// A processing node in a DAG.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub agent_name: String,
    pub inputs: Vec<String>,
}

/// A processing DAG (directed acyclic graph).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dag {
    pub nodes: Vec<Node>,
}

impl Dag {
    pub fn new() -> Self {
        Dag { nodes: Vec::new() }
    }

    pub fn add_node(&mut self, id: &str, agent_name: &str, inputs: Vec<&str>) {
        self.nodes.push(Node {
            id: id.to_string(),
            agent_name: agent_name.to_string(),
            inputs: inputs.iter().map(|s| s.to_string()).collect(),
        });
    }

    /// Topologically sort the DAG.
    pub fn topological_order(&self) -> Result<Vec<String>, String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        let mut ids: HashMap<String, &Node> = HashMap::new();

        for node in &self.nodes {
            in_degree.insert(node.id.clone(), 0);
            adj.insert(node.id.clone(), Vec::new());
            ids.insert(node.id.clone(), node);
        }

        for node in &self.nodes {
            for input in &node.inputs {
                if ids.contains_key(input) {
                    adj.get_mut(input).unwrap().push(node.id.clone());
                    *in_degree.get_mut(&node.id).unwrap() += 1;
                }
            }
        }

        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut result = Vec::new();
        while let Some(id) = queue.pop() {
            result.push(id.clone());
            if let Some(neighbors) = adj.get(&id) {
                for neighbor in neighbors {
                    let d = in_degree.get_mut(neighbor).unwrap();
                    *d -= 1;
                    if *d == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            return Err("cycle detected in DAG".into());
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose() {
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;
        let h = compose(f, g);
        assert_eq!(h(3), 8); // (3+1)*2 = 8
    }

    #[test]
    fn test_pipeline_map() {
        let items = vec![1, 2, 3];
        let result = pipeline_map(items, |x| x * 2);
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn test_pipeline_filter() {
        let items = vec![1, 2, 3, 4, 5];
        let result = pipeline_filter(items, |x| x % 2 == 0);
        assert_eq!(result, vec![2, 4]);
    }

    #[test]
    fn test_pipeline_flatmap() {
        let items = vec![1, 2, 3];
        let result = pipeline_flatmap(items, |x| vec![x, x * 10]);
        assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
    }

    #[test]
    fn test_pipeline_reduce() {
        let items = vec![1, 2, 3, 4, 5];
        let sum = pipeline_reduce(items, 0, |acc, x| acc + x);
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_fan_out() {
        let input = 42;
        let agents: Vec<Box<dyn Fn(i32) -> String>> = vec![
            Box::new(|x| format!("a: {}", x)),
            Box::new(|x| format!("b: {}", x)),
        ];
        let results = fan_out(input, agents);
        assert_eq!(results, vec!["a: 42", "b: 42"]);
    }

    #[test]
    fn test_fan_in() {
        let inputs = vec![1, 2, 3];
        let result = fan_in(inputs, |v| v.iter().sum::<i32>());
        assert_eq!(result, 6);
    }

    #[test]
    fn test_pipeline_result() {
        let r = PipelineResult::Success(42);
        assert!(r.is_success());
        assert_eq!(r.unwrap(), 42);
    }

    #[test]
    fn test_pipeline_result_skipped() {
        let r: PipelineResult<i32> = PipelineResult::Skipped;
        assert!(!r.is_success());
    }

    #[test]
    fn test_dag_topological_order() {
        let mut dag = Dag::new();
        dag.add_node("a", "extract", vec![]);
        dag.add_node("b", "transform", vec!["a"]);
        dag.add_node("c", "load", vec!["b"]);
        let order = dag.topological_order().unwrap();
        assert_eq!(order.len(), 3);
        let a_pos = order.iter().position(|x| x == "a").unwrap();
        let b_pos = order.iter().position(|x| x == "b").unwrap();
        let c_pos = order.iter().position(|x| x == "c").unwrap();
        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_dag_cycle_detection() {
        let mut dag = Dag::new();
        dag.add_node("a", "agent", vec!["b"]);
        dag.add_node("b", "agent", vec!["a"]);
        assert!(dag.topological_order().is_err());
    }

    #[test]
    fn test_dag_parallel() {
        let mut dag = Dag::new();
        dag.add_node("a", "extract", vec![]);
        dag.add_node("b", "extract", vec![]);
        dag.add_node("c", "merge", vec!["a", "b"]);
        let order = dag.topological_order().unwrap();
        assert_eq!(order.len(), 3);
        let c_pos = order.iter().position(|x| x == "c").unwrap();
        let a_pos = order.iter().position(|x| x == "a").unwrap();
        let b_pos = order.iter().position(|x| x == "b").unwrap();
        assert!(c_pos > a_pos);
        assert!(c_pos > b_pos);
    }

    #[test]
    fn test_compose_chain() {
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;
        let h = |x: i32| x - 3;
        let composed = compose(compose(f, g), h);
        assert_eq!(composed(5), 9); // ((5+1)*2)-3 = 9
    }
}
