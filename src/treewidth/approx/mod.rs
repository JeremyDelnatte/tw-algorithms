use serde::Serialize;
use strum::EnumIter;

pub mod four_approx;

#[derive(EnumIter, Serialize, Debug, Clone, Copy)]
pub enum ApproxAlgorithm {
    FourApprox,
}

impl ApproxAlgorithm {
    pub fn worst_case_from_optimal(&self, optimal: usize) -> usize {
        match self {
            ApproxAlgorithm::FourApprox => 4 * optimal,
        }
    }
}
