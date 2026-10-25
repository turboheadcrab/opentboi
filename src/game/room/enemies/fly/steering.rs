//! How a fly steers: finding its goal, pulling towards it (and pecking it), its jitter, and its
//! drift kept within the room (`alive()`'s fly branch, DoAction 13978–14115).

use glam::Vec2;

use super::{Fly, FlySurroundings, Goal, kind_parameters};
use crate::game::odds::Odds;
use crate::game::parameters::FlyGoal;
use crate::game::random::Random;
use crate::game::room::enemies::EnemyParameters;

/// One in 1000 frames beside its breakable a fly or Pooter pecks it, and if the peck lands
/// (`killshit()`) it becomes a Pooter.
const PECK_ODDS: Odds = Odds::one_in(1000);

impl Fly {
    /// Every third frame, a spot up to 240 px off, as far as the product of two random fractions of
    /// it (DoAction 13997, DoAction 13998), becomes its goal if a breakable stands there (DoAction
    /// 14001).
    pub(super) fn look_for_goal(
        &mut self,
        surroundings: &impl FlySurroundings,
        parameters: &EnemyParameters,
        random: &mut Random,
    ) {
        let reach = random.next_unit() * random.next_unit() * parameters.flies.goal_search_reach.get();
        let target = self.position + random.direction(reach);
        if let Some(cell) = surroundings.breakable_cell(target) {
            self.goal = Some(Goal::Cell(cell));
        }
    }

    /// Pulls towards its goal (resting 30 px away, 25 for the chasers, 15 for the red fly: DoAction
    /// 14011, DoAction 14016, DoAction 14018, with the vertical gap halved: DoAction 14023, and
    /// DoAction 14025 and DoAction 14027 for the pull); at rest by a breakable, one frame in 1000 it
    /// pecks it (DoAction 14031), and a landed peck makes it a Pooter, the two-shot one one time in
    /// three (DoAction 14037). The ones that do not chase keep 70 % of their drift meanwhile
    /// (DoAction 14047, DoAction 14049, DoAction 14050).
    pub(super) fn approach_goal(
        &mut self,
        player: Vec2,
        parameters: &EnemyParameters,
        random: &mut Random,
    ) -> Option<Vec2> {
        let (target, rest_distance) = match self.goal {
            None => return None,
            Some(Goal::Cell(cell)) => (cell, parameters.flies.goal_rest_distance.get()),
            Some(Goal::Player) => {
                let rest = match kind_parameters(self.kind, parameters).goal {
                    FlyGoal::Player(rest) => rest,
                    FlyGoal::Breakables => parameters.flies.goal_rest_distance,
                };
                (player, rest.get())
            }
        };
        let mut peck = None;
        let gap = Vec2::new(self.position.x - target.x, (self.position.y - target.y) / 2.0);
        let distance = gap.length();
        let pull = if distance > rest_distance {
            ((distance - rest_distance) * parameters.flies.approach_per_pixel).min(parameters.flies.approach_maximum)
                * parameters.flies.approach_scale
        } else {
            if random.happens(PECK_ODDS)
                && let Some(Goal::Cell(cell)) = self.goal
            {
                peck = Some(cell);
            }
            0.0
        };
        if kind_parameters(self.kind, parameters).goal == FlyGoal::Breakables {
            self.drift *= parameters.flies.drift_retention_approaching;
        }
        if distance > 0.0 {
            self.velocity -= gap * (pull / distance);
        }
        peck
    }

    /// Every fourth frame the drift gains its jitter (0.17, 0.3 for the attack fly: DoAction 14062,
    /// DoAction 14066, DoAction 14070), keeps 80 % (DoAction 14072, DoAction 14073) and halves
    /// when only one of its own cell and the cell 16 drifts ahead is solid (DoAction 14075,
    /// DoAction 14076, DoAction 14077, DoAction 14080, DoAction 14081).
    pub(super) fn jitter(
        &mut self,
        surroundings: &impl FlySurroundings,
        parameters: &EnemyParameters,
        random: &mut Random,
    ) {
        self.drift += random.direction(kind_parameters(self.kind, parameters).jitter);
        self.drift *= parameters.flies.jitter_retention;
        let here = surroundings.is_solid(self.position);
        let ahead = surroundings.is_solid(self.position + self.drift * parameters.flies.edge_lookahead);
        // The original also speeds the drift up when only its own cell is solid, but that case is
        // caught by the halving first, so it never happens.
        if here != ahead {
            self.drift *= parameters.flies.edge_retention;
        }
    }

    /// Every third frame, outside the stage's middle the drift turns back (DoAction 14095,
    /// DoAction 14099, DoAction 14103, DoAction 14107), then the speed gains twice the drift and
    /// keeps 60 % (DoAction 14112, DoAction 14113, DoAction 14114, DoAction 14115).
    pub(super) fn drift_within_room(&mut self, parameters: &EnemyParameters) {
        let (minimum, maximum) = (parameters.flies.stage_minimum, parameters.flies.stage_maximum);
        let push = parameters.flies.boundary_push.get();
        if self.position.y < minimum.y {
            self.drift.y += push;
        }
        if self.position.y > maximum.y {
            self.drift.y -= push;
        }
        if self.position.x > maximum.x {
            self.drift.x -= push;
        }
        if self.position.x < minimum.x {
            self.drift.x += push;
        }
        self.velocity += self.drift * parameters.flies.drift_gain;
        self.velocity *= parameters.flies.velocity_retention;
    }
}
