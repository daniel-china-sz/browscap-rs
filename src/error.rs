use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError{
    #[error("empty pattern")]
    EmptyPattern,
    #[error("fixed wild_card")]
    FixedPattern,
    #[error("invalid record")]
    InvalidRecord,
    
}
