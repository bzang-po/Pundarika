use crate::domain::{self, value_object::search::SearchType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchDto {
    pub search_keyword: String,
    pub search_type: SearchType,
}

impl TryInto<domain::value_object::search::SearchKeyword> for SearchDto {
    type Error = domain::error::DomainError;

    fn try_into(self) -> Result<domain::value_object::search::SearchKeyword, Self::Error> {
        todo!()
    }
}
