use crate::artifacts::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: HashMap<String, GraphNode>,
    pub edges: HashMap<String, GraphEdge>,
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub position: Point3,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub start_node_id: String,
    pub end_node_id: String,
}

#[derive(Debug, Clone)]
pub struct Matrix3x3 {
    // m<Row>_<Col>
    pub m0_0: Number,
    pub m0_1: Number,
    pub m0_2: Number,
    pub m1_0: Number,
    pub m1_1: Number,
    pub m1_2: Number,
    pub m2_0: Number,
    pub m2_1: Number,
    pub m2_2: Number,
}

#[derive(Debug, Clone)]
pub struct Matrix4x4 {
    // m<Row>_<Col>
    pub m0_0: Number,
    pub m0_1: Number,
    pub m0_2: Number,
    pub m0_3: Number,
    pub m1_0: Number,
    pub m1_1: Number,
    pub m1_2: Number,
    pub m1_3: Number,
    pub m2_0: Number,
    pub m2_1: Number,
    pub m2_2: Number,
    pub m2_3: Number,
    pub m3_0: Number,
    pub m3_1: Number,
    pub m3_2: Number,
    pub m3_3: Number,
}

#[derive(Debug, Clone)]
pub enum Transform {
    Transform2(Transform2),
    Transform3(Transform3),
    // TODO(doug): Consider making 2d to 3d a different type
    Transform2To3(Transform3),
}
