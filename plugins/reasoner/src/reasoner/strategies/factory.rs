use crate::state::StateManager;
use crate::strategies::base::Strategy;
use crate::strategies::beam_search::BeamSearchStrategy;
use crate::strategies::experiments::mcts_002_alpha::MCTS002AlphaStrategy;
use crate::strategies::experiments::mcts_002alt_alpha::MCTS002AltAlphaStrategy;
use crate::strategies::mcts::MonteCarloTreeSearchStrategy;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReasoningStrategy {
    BeamSearch,
    MCTS,
    MCTS002Alpha,
    MCTS002AltAlpha,
}

impl ReasoningStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReasoningStrategy::BeamSearch => "beam_search",
            ReasoningStrategy::MCTS => "mcts",
            ReasoningStrategy::MCTS002Alpha => "mcts_002_alpha",
            ReasoningStrategy::MCTS002AltAlpha => "mcts_002alt_alpha",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "beam_search" => Some(ReasoningStrategy::BeamSearch),
            "mcts" => Some(ReasoningStrategy::MCTS),
            "mcts_002_alpha" => Some(ReasoningStrategy::MCTS002Alpha),
            "mcts_002alt_alpha" => Some(ReasoningStrategy::MCTS002AltAlpha),
            _ => None,
        }
    }
}

pub struct StrategyFactory;

impl StrategyFactory {
    pub fn create_strategy(
        strategy_type: ReasoningStrategy,
        state_manager: Arc<StateManager>,
        beam_width: Option<usize>,
        num_simulations: Option<usize>,
    ) -> Arc<dyn Strategy> {
        match strategy_type {
            ReasoningStrategy::BeamSearch => {
                Arc::new(BeamSearchStrategy::new(state_manager, beam_width))
            }
            ReasoningStrategy::MCTS => Arc::new(MonteCarloTreeSearchStrategy::new(
                state_manager,
                num_simulations,
            )),
            ReasoningStrategy::MCTS002Alpha => {
                Arc::new(MCTS002AlphaStrategy::new(state_manager, num_simulations))
            }
            ReasoningStrategy::MCTS002AltAlpha => {
                Arc::new(MCTS002AltAlphaStrategy::new(state_manager, num_simulations))
            }
        }
    }
}
