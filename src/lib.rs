//! ala-cities — the parts of the game that do not need a GPU.
//!
//! Three layers, deliberately kept apart:
//!
//! * [`sim`] — the city. Deterministic: same seed, same inputs, same world.
//! * [`gov`] — the record. Governor, tickets, evidence, retirement.
//! * [`session`] — what you did. Interaction capture and feedback.
//!
//! The client (`src/main.rs`) renders these and adds nothing to them. The
//! verifier (`src/bin/verify.rs`) reads them without a window.

pub mod gov;
pub mod session;
pub mod sim;

pub use gov::{Governor, Op, Season, Ticket, TicketStatus, Verdict};
pub use sim::{Clock, World, SIM_HZ};
