use serde::Serialize;
use strum::EnumIter;

pub mod four_approx;
pub mod four_half_approx;

#[derive(EnumIter, Serialize, Debug, Clone, Copy)]
pub enum ApproxAlgorithm {
    FourApprox,
    FourHalfApprox,
}

impl ApproxAlgorithm {
    pub fn worst_case_from_optimal(&self, optimal: usize) -> usize {
        match self {
            ApproxAlgorithm::FourApprox => 4 * optimal,
            ApproxAlgorithm::FourHalfApprox => 4 * optimal + optimal / 2 + 1,
        }
    }
}
