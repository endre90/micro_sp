//! The 3D transform tree.
//!
//! Frames form a tree of parent-child poses. [`lookup`] walks that tree to
//! resolve the transform between any two frames, [`cycles`] guards the tree
//! against becoming a graph, [`loading`] reads scenarios off disk, [`treeviz`]
//! renders the tree for a terminal, and [`interface`] exposes all of it to the
//! rest of the runtime as a state-driven request/response service.

pub mod lookup;
pub mod cycles;
pub mod loading;
pub mod interface;
pub mod treeviz;
