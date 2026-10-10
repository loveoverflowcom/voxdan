//! One modular backend; Script IR and revision decisions are independent of adapters.

pub mod http;
pub mod operator;
pub mod postgres;
pub mod revisions;
pub mod script_ir;
