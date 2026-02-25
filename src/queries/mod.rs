pub mod error;
pub mod types;

pub use error::QueryError;
pub use types::{
    AspectCriteria, AspectType, Body, ExactAspect, QueryResult, RetrogradeCriteria,
    RetrogradePeriod, RetrogradeStatus, VoCCriteria, VoCPeriod, WeddingCandidate, WeddingCriteria,
    ZodiacSign, ALL_ASPECT_TYPES, ALL_BODIES, FAVORABLE_WEDDING_SIGNS,
};
