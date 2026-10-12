use crate::domain::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchKeyword {
    keywords: Vec<String>,
    search_type: SearchType,
}

impl SearchKeyword {
    pub fn new(raw_keywords: &str, search_type: SearchType) -> Result<Self, DomainError> {
        let keywords: Vec<String> = raw_keywords
            .trim()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // トリミングした結果、空になってしまった場合はエラーを返す
        if keywords.is_empty() {
            return Err(DomainError::InvalidSearchKeyword);
        }

        Ok(Self {
            keywords,
            search_type,
        })
    }

    pub fn keywords(&self) -> &Vec<String> {
        &self.keywords
    }

    pub fn search_type(&self) -> SearchType {
        self.search_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    And,
    Or,
}

#[cfg(test)]
mod tests {
    use super::{SearchKeyword, SearchType};
    use crate::domain::error::DomainError;

    // 半角スペースで区切られた検索語句を生成できることを確認する。
    #[test]
    fn creates_keyword_from_space_separated_words() {
        let keyword = SearchKeyword::new("rust sqlite", SearchType::And).unwrap();

        assert_eq!(keyword.keywords(), &["rust", "sqlite"]);
    }

    // 前後および連続した空白文字をトリミングして分割できることを確認する。
    #[test]
    fn trims_leading_trailing_and_repeated_whitespace() {
        let keyword = SearchKeyword::new("  rust\t\nsqlite  ", SearchType::Or).unwrap();

        assert_eq!(keyword.keywords(), &["rust", "sqlite"]);
    }

    // 日本語検索語句の前後にある半角スペースをトリミングできることを確認する。
    #[test]
    fn trims_half_width_spaces_around_japanese_keywords() {
        let keyword = SearchKeyword::new("  日本語 検索  ", SearchType::And).unwrap();

        assert_eq!(keyword.keywords(), &["日本語", "検索"]);
    }

    // 日本語検索語句の前後にある全角スペースをトリミングできることを確認する。
    #[test]
    fn trims_full_width_spaces_around_japanese_keywords() {
        let keyword = SearchKeyword::new("　日本語　検索　", SearchType::And).unwrap();

        assert_eq!(keyword.keywords(), &["日本語", "検索"]);
    }

    // 日本語検索語句の前後にある半角・全角スペースを混在してトリミングできることを確認する。
    #[test]
    fn trims_mixed_width_spaces_around_japanese_keywords() {
        let keyword = SearchKeyword::new(" 　日本語　検索  ", SearchType::And).unwrap();

        assert_eq!(keyword.keywords(), &["日本語", "検索"]);
    }

    // 日本語の検索語句を保持したまま分割できることを確認する。
    #[test]
    fn preserves_unicode_keywords() {
        let keyword = SearchKeyword::new("日本語 検索", SearchType::And).unwrap();

        assert_eq!(keyword.keywords(), &["日本語", "検索"]);
    }

    // 半角スペースで日本語検索語句を分割できることを確認する。
    #[test]
    fn splits_japanese_keywords_with_half_width_spaces() {
        let keyword = SearchKeyword::new("日本語 検索", SearchType::And).unwrap();

        assert_eq!(keyword.keywords(), &["日本語", "検索"]);
    }

    // 全角スペースで日本語検索語句を分割できることを確認する。
    #[test]
    fn splits_japanese_keywords_with_full_width_spaces() {
        let keyword = SearchKeyword::new("日本語　検索", SearchType::And).unwrap();

        assert_eq!(keyword.keywords(), &["日本語", "検索"]);
    }

    // 半角・全角スペースが混在する日本語検索語句を分割できることを確認する。
    #[test]
    fn splits_japanese_keywords_with_mixed_width_spaces() {
        let keyword = SearchKeyword::new("日本語 検索　テスト", SearchType::And).unwrap();

        assert_eq!(keyword.keywords(), &["日本語", "検索", "テスト"]);
    }

    // 半角・全角スペース、タブ、改行だけの入力を拒否することを確認する。
    #[test]
    fn rejects_empty_keyword() {
        let result = SearchKeyword::new("　   \t\n　", SearchType::And);

        assert_eq!(result, Err(DomainError::InvalidSearchKeyword));
    }
}
