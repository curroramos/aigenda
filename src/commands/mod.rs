pub mod add;
pub mod list;

#[cfg(feature = "ai")]
pub mod agent;

#[cfg(not(feature = "ai"))]
pub mod agent_stub;

#[cfg(not(feature = "ai"))]
pub use agent_stub as agent;

// (export, search will come later)
