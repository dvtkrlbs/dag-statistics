use std::collections::{HashMap, HashSet};
use std::io::{BufReader, BufRead};
use std::io::Read;

/// This struct holds the node and edges of an Directed Acyclic Graph
pub struct DirectedAcyclicGraph {
    /// Nodes are stored as a HashSet to achive deduplication
    nodes: HashSet<usize>,
    /// Edges are stored as a HashSet to achieve deduplication
    edges: HashSet<(usize, usize)>,
}

impl DirectedAcyclicGraph {
    /// Returns an empty Directed Acyclic Graph
    pub fn new() -> DirectedAcyclicGraph {
        DirectedAcyclicGraph {
            nodes: HashSet::new(),
            edges: HashSet::new(),
        }
    }

    /// Creates a new Directed Acyclic Graph from anything that implements `Read`
    /// The database structure is as follows
    /// Line 1: N, the number of nodes in the database
    /// Lines 2 through N + 1: the node data, where each node consists of the ids of its left and right parents
    /// Node id 1 is the unique origin of all nodes
    /// The id of each node in the database is its line number
    /// # Arguments
    /// * `reader` - Anything that implements `Read`
    pub fn from_read(reader: impl Read) -> Result<DirectedAcyclicGraph, std::io::Error> {
        let mut reader = BufReader::new(reader);

        let mut line = String::new();
        reader.read_line(&mut line)?;

        // let size = line.trim().parse::<usize>()?;

        let lines: Vec<String> = reader
            .lines()
            .filter(|res| res.is_ok())
            .map(|res| res.unwrap())
            .collect();

        let node_us: Vec<(usize, usize)> = lines
            .iter()
            .map(|l| l.trim().split_whitespace().clone())
            .map(|mut l| (l.next().unwrap(), l.next().unwrap()))
            .map(|(a, b)| (a.parse().unwrap(), b.parse().unwrap()))
            .collect();

        let mut dag = DirectedAcyclicGraph::new();
        for (i, (left, right)) in node_us.into_iter().enumerate() {
            dag.nodes.insert(i + 2);
            dag.nodes.insert(left);
            dag.nodes.insert(right);
            if i + 2 != left {
                dag.edges.insert((i + 2, left));
            }
            if i + 2 != right {
                dag.edges.insert((i + 2, right));
            }
        }

        Ok(dag)
    }

    /// Get the all possible paths from `node` to node with id 1
    /// # Arguments
    /// * `node` - Node Id to search
    pub fn depths(&self, node: usize) -> Vec<Vec<usize>> {
        let neighbors: Vec<usize> = self
            .edges
            .iter()
            .cloned()
            .filter(|(from, _)| *from == node)
            .map(|(_, to)| to)
            .collect();

        if node == 1 {
            return vec![vec![1]];
        }

        let mut depths: Vec<Vec<usize>> = Vec::with_capacity(neighbors.len());

        for neighbor in neighbors {
            if node != neighbor {
                depths.extend(
                    self.depths(neighbor)
                        .into_iter()
                        .filter(|d| !d.contains(&node))
                        .map(|mut d| {
                            d.push(node);
                            d
                        }),
                );
            }
        }

        return depths;
    }

    /// Average depth from all nodes to node 1
    pub fn avg_depth(&self) -> f64 {
        let mut total = 0.0;
        let mut depth_count = 0;
        for node in self.nodes().iter() {
            if node == &1 {
                depth_count += 1;
                continue;
            }

            let depths = self.depths(*node);
            total += depths.iter().map(|depth| depth.len() - 1).min().unwrap() as f64;
            depth_count += 1;
        }

        total / depth_count as f64
    }

    /// Average node count at each depth excluding depth 0
    pub fn avg_node_per_depth(&self) -> f64 {
        let mut node_count_per_depth = HashMap::new();

        for node in self.nodes() {
            if node == &1 {
                continue;
            }
            for depth in self.depths(*node) {
                let count = node_count_per_depth.entry(depth.len()).or_insert(0);
                *count += 1;
            }
        }

        node_count_per_depth.values().sum::<i32>() as f64 / node_count_per_depth.len() as f64
    }

    /// Average in-reference per node
    pub fn avg_ref(&self) -> f64 {
        let mut total = 0;
        for node in self.nodes() {
            total += self.edges().iter().filter(|(_, to)| *to == *node).count();
        }

        total as f64 / self.nodes.len() as f64
    }

    /// Longest depth
    pub fn max_depth(&self) -> usize {
        let mut max = 0;
        for node in self.nodes() {
            for depth in self.depths(*node) {
                if depth.len() > max {
                    max = depth.len();
                }
            }
        }

        max
    }

    /// Borrow nodes of the DAG
    pub fn nodes(&self) -> &HashSet<usize> {
        &self.nodes
    }

    /// Borrow edges of the DAG
    pub fn edges(&self) -> &HashSet<(usize, usize)> {
        &self.edges
    }

    /// Inserts a new edge to the DAG
    /// Adds the nodes to the DAG if they dont exist
    /// Returns if the edge got actually added to DAG
    /// # Arguments
    /// * `from` - Start node id
    /// * `to` - Destination node id
    pub fn add_edge(&mut self, from: usize, to: usize) -> bool {
        if from == to {
            return false;
        }
        self.nodes.insert(from);
        self.nodes.insert(to);

        self.edges.insert((from, to))
    }


    /// Removes the specified edge from the DAF if it exists on the edges
    /// Also purges any edgeless nodes
    /// Warning: This method does not currently validates if after removal origin becomes unreachable
    /// So be careful while removing any edges
    /// Returns if the edge got actually removed from DAG
    /// # Arguments
    /// * `from` - Starting node id
    /// * `to` - Destination node id
    pub fn remove_edge(&mut self, from: usize, to: usize) -> bool {
        if self.edges.contains(&(from, to)) {
            self.edges.remove(&(from, to));
            self.purge_stale_nodes();
            return true;
        }

        return false;
    }

    /// Removes the specified node from the DAG alongside with any edges that references that node
    /// Returns if the node got actually removed from the DAG
    /// # Arguments
    /// * `node` - Node id to remove (can't be 1)
    pub fn remove_node(&mut self, node: usize) -> bool {
        if node == 1 || !self.nodes.contains(&node) {
            return false;
        }

        self.nodes.remove(&node);
        self.purge_stale_edges();
        return true;
    }

    /// Purges stale nodes (nodes that does not have any edges) from the DAG
    fn purge_stale_nodes(&mut self) {
        let mut nodes_to_remove = HashSet::new();
        for node in self.nodes.iter() {
            if *node == 1 {
                continue;
            }

            if self.edges.iter().filter(|(from, to)| *from == *node || *to == *node).count() == 0 {
                nodes_to_remove.insert(*node);
            };
        }

        for node in nodes_to_remove {
            self.remove_node(node);
        }
    }

    /// Purges stale edges (edges that reference non existent nodes)
    fn purge_stale_edges(&mut self) {
        let mut edges_to_remove = HashSet::new();
        for (from, to) in self.edges.iter() {
            if !self.nodes.contains(from) || !self.nodes.contains(to) {
                edges_to_remove.insert((*from, *to));
            }
        }

        for (from, to) in edges_to_remove {
            self.remove_edge(from, to);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DirectedAcyclicGraph;

    #[test]
    fn test_if_dag_constructed_correctly() {
        let database = "5
1 1
1 2
2 2
3 6
3 3";

        let nodes = vec![1, 2, 3, 4, 5, 6];
        let edges = vec![(6, 3), (4, 2), (5, 3), (5, 6), (3, 2), (2, 1), (3, 1)];

        let dag = DirectedAcyclicGraph::from_read(database.as_bytes()).unwrap();

        assert_eq!(dag.nodes.len(), nodes.len());
        assert_eq!(dag.edges.len(), edges.len());

        for node in dag.nodes() {
            assert!(nodes.contains(node));
        }

        for edge in dag.edges() {
            assert!(edges.contains(&edge));
        }

        assert_eq!(dag.max_depth(), 5);
    }
}
