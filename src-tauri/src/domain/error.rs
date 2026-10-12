#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    InvalidSearchType,
    EmptySearchKeyword,
    InvalidSearchKeyword,
}
