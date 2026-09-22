//! ala-cities — the parts of the game that do not need a GPU — and the
//! interface layer the client and the tools share.
//!
//! Layers, deliberately kept apart:
//!
//! * [`sim`] — the city. Deterministic: same seed, same inputs, same world.
//! * [`gov`] — the record. Governor, tickets, evidence, retirement.
//! * [`session`] — what you did. Interaction capture and feedback.
//! * [`design`] — the tokens: type scale, spacing scale, targets, the gate.
//! * [`render`] — the single wgpu stack every surface draws through.
//! * [`hud`] — the game's HUD vocabulary, tokens and all.
//! * [`ui`] — the shared widget layer: measured layout, scroll, footer.
//! * [`iconreview`] — the review hand-off: read `review.json`, record decisions.
//!
//! The client (`src/main.rs`) renders these and adds nothing to them. The
//! tools (`src/bin/pick.rs`, `src/bin/verify.rs`) render and record through
//! the same modules, which is what a115/a128 bought: one layer, two surfaces.

pub mod design;
pub mod agentledger;
pub mod audio;
pub mod buildinfo;
pub mod gov;
pub mod hud;
pub mod iconreview;
pub mod icons;
pub mod render;
pub mod session;
pub mod sim;
pub mod ui;

pub use gov::{Governor, Op, Season, Ticket, TicketStatus, Verdict};
pub use sim::{Clock, World, SIM_HZ};
