use crate::{Beatmap, OsuPP};
use super::{OsuGradualDifficultyAttributes, OsuPerformanceAttributes};

/// Aggregation for a score's current state i.e. what was the
/// maximum combo so far and what are the current hitresults.
///
/// This struct is used for [`OsuGradualPerformanceAttributes`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OsuScoreState {
    /// Maximum combo that the score has had so far.
    /// **Not** the maximum possible combo of the map so far.
    pub max_combo: usize,
    /// Amount of current 300s.
    pub n300: usize,
    /// Amount of current 100s.
    pub n100: usize,
    /// Amount of current 50s.
    pub n50: usize,
    /// Amount of current misses.
    pub n_misses: usize,
}

impl OsuScoreState {
    /// Create a new empty score state.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the total amount of hits by adding everything up.
    #[inline]
    pub fn total_hits(&self) -> usize {
        self.n300 + self.n100 + self.n50 + self.n_misses
    }

    /// Calculate the accuracy between `0.0` and `1.0` for this state.
    #[inline]
    pub fn accuracy(&self) -> f64 {
        let total_hits = self.total_hits();

        if total_hits == 0 {
            return 0.0;
        }

        let numerator = 6 * self.n300 + 2 * self.n100 + self.n50;
        let denominator = 6 * total_hits;

        numerator as f64 / denominator as f64
    }
}

/// Trait for providing osu!standard difficulty attributes.
pub trait OsuAttributeProvider {
    fn get_aim(&self) -> f64;
    fn get_speed(&self) -> f64;
    fn get_overall_difficulty(&self) -> f64;
    fn get_approach_rate(&self) -> f64;
    fn get_max_combo(&self) -> usize;
}

/// Gradually calculate the performance attributes of an osu!standard map.
///
/// After each hit object you can call
/// [`process_next_object`](`OsuGradualPerformanceAttributes::process_next_object`)
/// and it will return the resulting current [`OsuPerformanceAttributes`].
/// To process multiple objects at once, use
/// [`process_next_n_objects`](`OsuGradualPerformanceAttributes::process_next_n_objects`) instead.
///
/// Both methods require an [`OsuScoreState`] that contains the current
/// hitresults as well as the maximum combo so far.
///
/// If you only want to calculate difficulty attributes use
/// [`OsuGradualDifficultyAttributes`](crate::osu::OsuGradualDifficultyAttributes) instead.
#[derive(Debug)]
pub struct OsuGradualPerformanceAttributes<'map> {
    difficulty: OsuGradualDifficultyAttributes,
    performance: OsuPP<'map>,
}

impl<'map> OsuGradualPerformanceAttributes<'map> {
    /// Create a new gradual performance calculator for osu!standard maps.
    pub fn new(map: &'map Beatmap, mods: u32) -> Self {
        let difficulty = OsuGradualDifficultyAttributes::new(map, mods);
        let performance = OsuPP::new(map).mods(mods).passed_objects(0);

        Self {
            difficulty,
            performance,
        }
    }

    /// Process the next hit object and calculate the
    /// performance attributes for the resulting score state.
    pub fn process_next_object(
        &mut self,
        state: OsuScoreState,
    ) -> Option<OsuPerformanceAttributes> {
        self.process_next_n_objects(state, 1)
    }

    /// Same as [`process_next_object`](`OsuGradualPerformanceAttributes::process_next_object`)
    /// but instead of processing only one object it process `n` many.
    ///
    /// If `n` is 0 it will be considered as 1.
    /// If there are still objects to be processed but `n` is larger than the amount
    /// of remaining objects, `n` will be considered as the amount of remaining objects.
    pub fn process_next_n_objects(
        &mut self,
        state: OsuScoreState,
        n: usize,
    ) -> Option<OsuPerformanceAttributes> {
        let sub = (self.difficulty.idx == 0) as usize;
        let difficulty = self.difficulty.nth(n.saturating_sub(sub))?;

        let performance = self
            .performance
            .clone()
            .attributes(difficulty)
            .state(state)
            .passed_objects(self.difficulty.idx + 1)
            .calculate();

        Some(performance)
    }
}

/// Struct representing osu!standard difficulty attributes.
#[derive(Debug, Clone)]
pub struct OsuDifficultyAttributes {
    pub aim: f64,
    pub speed: f64,
    pub overall_difficulty: f64,
    pub approach_rate: f64,
    pub max_combo: usize,
}

/// Implement the OsuAttributeProvider trait for OsuDifficultyAttributes.
impl OsuAttributeProvider for OsuDifficultyAttributes {
    fn get_aim(&self) -> f64 {
        self.aim
    }

    fn get_speed(&self) -> f64 {
        self.speed
    }

    fn get_overall_difficulty(&self) -> f64 {
        self.overall_difficulty
    }

    fn get_approach_rate(&self) -> f64 {
        self.approach_rate
    }

    fn get_max_combo(&self) -> usize {
        self.max_combo
    }
}

/// Update the OsuPP struct to use the OsuAttributeProvider trait.
impl<'m> OsuPP<'m> {
    pub fn attributes<T: OsuAttributeProvider>(self, attributes: T) -> Self {
        self.aim(attributes.get_aim())
            .speed(attributes.get_speed())
            .overall_difficulty(attributes.get_overall_difficulty())
            .approach_rate(attributes.get_approach_rate())
            .max_combo(attributes.get_max_combo())
    }
}