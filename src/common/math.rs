use crate::artifacts::*;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Graph {
    pub nodes: HashMap<String, GraphNode>,
    pub edges: HashMap<String, GraphEdge>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphNode {
    pub position: Point3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphEdge {
    pub start_node_id: String,
    pub end_node_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Transform {
    Transform2(Transform2),
    Transform3(Transform3),
    // TODO(doug): Consider making 2d to 3d a different type
    Transform2To3(Transform3),
}
