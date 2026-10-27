//! The values the enemies and their shots follow.

use crate::game::parameters::{
    ClottyParameters, EnemyShotParameters, FireParameters, FleshMoundParameters, FlyParameters, GameParameters,
    GaperParameters, HeadlessParameters, HopperParameters, HorfParameters, MulliganParameters, PooterParameters,
    SpikeParameters, SplatParameters,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnemyParameters {
    pub flies: FlyParameters,
    pub gapers: GaperParameters,
    pub headless: HeadlessParameters,
    pub horfs: HorfParameters,
    pub clottys: ClottyParameters,
    pub hoppers: HopperParameters,
    pub flesh_mounds: FleshMoundParameters,
    pub mulligans: MulliganParameters,
    pub pooters: PooterParameters,
    pub shots: EnemyShotParameters,
    pub fire: FireParameters,
    pub spikes: SpikeParameters,
    pub splats: SplatParameters,
}

impl EnemyParameters {
    pub fn from_game(parameters: &GameParameters) -> Self {
        Self {
            flies: parameters.flies,
            gapers: parameters.gapers,
            headless: parameters.headless,
            horfs: parameters.horfs,
            clottys: parameters.clottys,
            hoppers: parameters.hoppers,
            flesh_mounds: parameters.flesh_mounds,
            mulligans: parameters.mulligans,
            pooters: parameters.pooters,
            shots: parameters.enemy_shots,
            fire: parameters.fire,
            spikes: parameters.spikes,
            splats: parameters.splats,
        }
    }
}
