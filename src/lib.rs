//! ala-cities — the parts of the game that do not need a GPU — and the
//! interface layer the client and the tools share.
//!
//! Layers, deliberately kept apart:
//!
//! * [`sim`] — the city. Deterministic: same seed, same inputs, same world.
//! * [`gov`] — the record. Governor, tickets, evidence, retirement.
//! * [`session`] — what you did. Interaction capture and feedback.
//! * [`asset_generator`] — target presentation assets from world facts/recipes.
//! * [`asset_controls`] — external builder controls, never player authority.
//! * [`design`] — the tokens: type scale, spacing scale, targets, the gate.
//! * [`render`] — the single wgpu stack every surface draws through.
//! * [`hud`] — the game's HUD vocabulary, tokens and all.
//! * [`ui`] — the shared widget layer: measured layout, scroll, footer.
//! * [`iconreview`] — the review hand-off: read `review.json`, record decisions.
//!
//! The target client renders generator output without exposing authoring
//! controls. The external asset-builder and verification tools call the same
//! library and renderer, keeping authoring, review, and runtime consumption
//! separate without creating a second implementation.

pub mod agentledger;
pub mod asset_controls;
pub mod asset_generator;
pub mod audio;
pub mod buildinfo;
pub mod design;
pub mod founding_day;
pub mod gov;
pub mod hud;
pub mod iconreview;
pub mod icons;
pub mod materials;
pub mod octree;
pub mod raycast;
pub mod render;
pub mod session;
pub mod sim;
pub mod text;
pub mod ui;

pub use gov::{Governor, Op, Season, Ticket, TicketStatus, Verdict};
pub use sim::{Clock, World, SIM_HZ};
