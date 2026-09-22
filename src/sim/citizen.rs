//! Citizens.
//!
//! A citizen is an agent with a home, a job, and a commute it actually walks.
//! The renderer never interpolates an agent by wall clock: it interpolates by
//! [`Citizen::step_work`] — the fraction of the *current step* the agent has
//! completed. An agent that has done no work this step renders frozen, which is
//! the honest picture of a citizen stuck in traffic.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CitizenState {
    AtHome,
    ToWork,
    AtWork,
    ToHome,
    /// Not "between jobs" — this is a citizen with a home but no work, which is
    /// exactly the condition the cases are sampled from.
    Unemployed,
}

impl CitizenState {
    pub fn name(self) -> &'static str {
        match self {
            CitizenState::AtHome => "at home",
            CitizenState::ToWork => "commuting to work",
            CitizenState::AtWork => "at work",
            CitizenState::ToHome => "commuting home",
            CitizenState::Unemployed => "out of work",
        }
    }

    pub fn is_commuting(self) -> bool {
        matches!(self, CitizenState::ToWork | CitizenState::ToHome)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Citizen {
    pub id: u32,
    pub home: Option<u32>,
    pub work: Option<u32>,
    pub state: CitizenState,
    /// Tiles to walk, first to last.
    pub path: Vec<u32>,
    /// Index of the tile the agent is currently standing on.
    pub path_cursor: usize,
    /// Fraction of the step to `path[path_cursor + 1]` that is complete.
    /// In `[0, 1)`. This is the only interpolation source the renderer has.
    pub step_work: f32,
    pub born_tick: u64,
    /// Set once the citizen has been through a settlement pass. Guards against
    /// a citizen being handed a job in the same tick it was created.
    pub ready: bool,
}

impl Citizen {
    pub fn new(id: u32, home: u32, tick: u64) -> Self {
        Self {
            id,
            home: Some(home),
            work: None,
            state: CitizenState::Unemployed,
            path: Vec::new(),
            path_cursor: 0,
            step_work: 0.0,
            born_tick: tick,
            ready: false,
        }
    }

    /// The tile the agent is standing on, if it is on the map.
    pub fn current_tile(&self) -> Option<u32> {
        self.path.get(self.path_cursor).copied()
    }

    /// The tile being approached, if there is one.
    pub fn next_tile(&self) -> Option<u32> {
        self.path.get(self.path_cursor + 1).copied()
    }

    /// Where to draw the agent: between [`Self::current_tile`] and
    /// [`Self::next_tile`], at [`Self::step_work`]. Returns `None` when the
    /// agent has no route to draw — the caller draws nothing rather than
    /// guessing a position.
    pub fn interpolated_tiles(&self) -> Option<(u32, u32, f32)> {
        let current = self.current_tile()?;
        let next = self.next_tile()?;
        let work = self.step_work.clamp(0.0, 1.0);
        Some((current, next, work))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_citizen_has_a_home_and_no_job() {
        let citizen = Citizen::new(1, 7, 0);
        assert_eq!(citizen.home, Some(7));
        assert_eq!(citizen.work, None);
        assert_eq!(citizen.state, CitizenState::Unemployed);
        assert!(!citizen.ready);
    }

    #[test]
    fn interpolation_uses_work_done_not_a_clock() {
        let mut citizen = Citizen::new(1, 0, 0);
        citizen.path = vec![10, 11, 12];
        citizen.path_cursor = 0;
        citizen.step_work = 0.0;
        assert_eq!(citizen.interpolated_tiles(), Some((10, 11, 0.0)));

        // Work is what moves the agent, so an agent that does none does not move.
        let unmoved = citizen.interpolated_tiles().unwrap();
        assert_eq!(unmoved, (10, 11, 0.0));

        citizen.step_work = 0.5;
        assert_eq!(citizen.interpolated_tiles(), Some((10, 11, 0.5)));
    }

    #[test]
    fn an_agent_with_no_route_has_nothing_to_draw() {
        let citizen = Citizen::new(1, 0, 0);
        assert_eq!(citizen.interpolated_tiles(), None);
        assert_eq!(citizen.current_tile(), None);
    }

    #[test]
    fn the_last_step_has_no_next_tile() {
        let mut citizen = Citizen::new(1, 0, 0);
        citizen.path = vec![10];
        citizen.path_cursor = 0;
        assert_eq!(citizen.current_tile(), Some(10));
        assert_eq!(citizen.next_tile(), None);
        assert_eq!(citizen.interpolated_tiles(), None);
    }

    #[test]
    fn step_work_is_clamped_when_it_is_used_for_rendering() {
        let mut citizen = Citizen::new(1, 0, 0);
        citizen.path = vec![1, 2];
        citizen.step_work = 4.0;
        let (_, _, work) = citizen.interpolated_tiles().expect("a drawable step");
        assert_eq!(work, 1.0);
    }
}
