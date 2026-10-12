use std::fmt::Debug;

use crate::domain::value_object::search::SearchType::{And, Or};
use crate::domain::value_object::search::{SearchKeyword, SearchType};
use log::trace;
use sqlx::QueryBuilder;

#[derive(Debug, PartialEq, Eq)]
enum SearchCondition {
    Like(String),
    Fts(String),
}

pub(crate) trait SqliteFuzzySearchTarget {
    const TABLE: &'static str;
    const SELECT_COLUMNS: &'static str;
    const ID_COLUMN: &'static str;
    const LIKE_COLUMN: &'static str;
    const FTS_TABLE: &'static str;
}

/// 全文検索クエリを構築する
///
/// # Arguments
///
/// * `keywords` - 検索キーワードを含む `SearchKeyword` オブジェクト
///
/// # Returns
///
/// `QueryBuilder<sqlx::Sqlite>` - 構築された全文検索クエリビルダー
pub(crate) fn build_search_query<T: SqliteFuzzySearchTarget>(
    keywords: &SearchKeyword,
) -> QueryBuilder<sqlx::Sqlite> {
    trace!(
        "検索対象テーブル情報 テーブル名: {}, IDカラム: {}, LIKEカラム: {}, FTSテーブル: {}",
        T::TABLE,
        T::ID_COLUMN,
        T::LIKE_COLUMN,
        T::FTS_TABLE
    );
    trace!("検索キーワード: {:?}", keywords);
    let mut query_builder: QueryBuilder<sqlx::Sqlite> =
        QueryBuilder::new(format!("select {} from {}", T::SELECT_COLUMNS, T::TABLE));

    match keywords.search_type() {
        And => query_builder.push(" where 1 = 1"),
        Or => query_builder.push(" where 1 = 0"),
    };

    for condition in collect_search_conditions(keywords) {
        bind_search_condition::<T>(&mut query_builder, condition, keywords.search_type());
    }

    trace!("構築された検索クエリ: {:?}", query_builder.sql());

    query_builder
}

fn collect_search_conditions(keywords: &SearchKeyword) -> Vec<SearchCondition> {
    let mut conditions = keywords
        .keywords()
        .iter()
        .map(|keyword| {
            if keyword.chars().count() <= 2 {
                SearchCondition::Like(format!("%{}%", escape_like(keyword)))
            } else {
                SearchCondition::Fts(escape_fts(keyword))
            }
        })
        .collect::<Vec<_>>();

    let fts_operator = match keywords.search_type() {
        And => " AND ",
        Or => " OR ",
    };
    let fts_keyword = conditions
        .iter()
        .filter_map(|condition| match condition {
            SearchCondition::Fts(keyword) => Some(keyword.as_str()),
            SearchCondition::Like(_) => None,
        })
        .collect::<Vec<_>>()
        .join(fts_operator);

    if !fts_keyword.is_empty() {
        let mut aggregated = Vec::with_capacity(conditions.len());
        let mut fts_added = false;
        for condition in conditions {
            match condition {
                SearchCondition::Fts(_) if !fts_added => {
                    aggregated.push(SearchCondition::Fts(fts_keyword.clone()));
                    fts_added = true;
                }
                SearchCondition::Fts(_) => {}
                SearchCondition::Like(keyword) => {
                    aggregated.push(SearchCondition::Like(keyword));
                }
            }
        }
        conditions = aggregated;
    }

    conditions
}

fn bind_search_condition<T: SqliteFuzzySearchTarget>(
    query_builder: &mut QueryBuilder<sqlx::Sqlite>,
    condition: SearchCondition,
    search_type: SearchType,
) {
    let operator = match search_type {
        And => " and ",
        Or => " or ",
    };

    match condition {
        SearchCondition::Like(keyword) => {
            query_builder
                .push(operator)
                .push(T::LIKE_COLUMN)
                .push(" like ")
                .push_bind(keyword)
                .push(" escape '\\'");
        }
        SearchCondition::Fts(keyword) => {
            query_builder
                .push(operator)
                .push(format!(
                    "exists (select 1 from {} where {}.rowid = {}.{} and {} match ",
                    T::FTS_TABLE,
                    T::FTS_TABLE,
                    T::TABLE,
                    T::ID_COLUMN,
                    T::FTS_TABLE,
                ))
                .push_bind(keyword)
                .push(")");
        }
    }
}

///
/// LIKE句の特殊文字をエスケープする
/// # Arguments
/// * `input` - エスケープ対象の文字列
///
/// # Returns
/// エスケープ後の文字列
fn escape_like(input: &str) -> String {
    trace!("LIKE句エスケープ前文字列: {}", input);

    let mut escaped = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '%' | '_' | '\\' => {
                escaped.push('\\');
                escaped.push(c);
            }
            _ => escaped.push(c),
        }
    }
    trace!("LIKE句エスケープ後文字列: {}", escaped);
    escaped
}

///
/// 全文検索用の特殊文字をエスケープする
/// # Arguments
/// * `input` - エスケープ対象の文字列
///
/// # Returns
/// エスケープ後の文字列
fn escape_fts(input: &str) -> String {
    trace!("全文検索特殊文字エスケープ前文字列: {}", input);

    let escape_keyword = input.replace('"', "\"\"");
    let escaped = format!("\"{}\"", escape_keyword);

    trace!("全文検索特殊文字エスケープ後文字列: {}", escaped);

    escaped
}

#[cfg(test)]
mod tests {
    use super::{
        build_search_query, collect_search_conditions, escape_fts, escape_like, SearchCondition,
        SqliteFuzzySearchTarget,
    };
    use crate::{
        domain::value_object::search::{SearchKeyword, SearchType},
        test_support::logging::init_logger,
    };

    #[derive(Debug)]
    struct TestFuzzySearchTarget;

    impl SqliteFuzzySearchTarget for TestFuzzySearchTarget {
        const TABLE: &'static str = "test";
        const SELECT_COLUMNS: &'static str = "test.id, test.content";
        const ID_COLUMN: &'static str = "id";
        const LIKE_COLUMN: &'static str = "test.content";
        const FTS_TABLE: &'static str = "test_fts";
    }

    // 入力順を維持し、短い語をLIKE、長い語をFTS5用の値へ変換することを確認する。
    #[test]
    fn converts_mixed_length_keywords_in_input_order() {
        init_logger();
        let keywords = SearchKeyword::new("ab database cd", SearchType::And).unwrap();

        assert_eq!(
            collect_search_conditions(&keywords),
            vec![
                SearchCondition::Like("%ab%".to_owned()),
                SearchCondition::Fts("\"database\"".to_owned()),
                SearchCondition::Like("%cd%".to_owned()),
            ]
        );
    }

    // 同じ検索種別のFTS語を1つのMATCH値へ集約し、LIKE語は入力順を維持することを確認する。
    #[test]
    fn aggregates_multiple_search_terms_by_length() {
        init_logger();
        let keywords = SearchKeyword::new("a database cd sqlite", SearchType::And).unwrap();

        assert_eq!(
            collect_search_conditions(&keywords),
            vec![
                SearchCondition::Like("%a%".to_owned()),
                SearchCondition::Fts("\"database\" AND \"sqlite\"".to_owned()),
                SearchCondition::Like("%cd%".to_owned()),
            ]
        );
    }

    // OR検索では複数のFTS語をORで連結することを確認する。
    #[test]
    fn aggregates_multiple_fts_terms_with_or() {
        init_logger();
        let keywords = SearchKeyword::new("database sqlite", SearchType::Or).unwrap();

        let query = build_search_query::<TestFuzzySearchTarget>(&keywords);

        assert_eq!(
            query.sql(),
            "select test.id, test.content from test where 1 = 0 or exists (select 1 from test_fts where test_fts.rowid = test.id and test_fts match ?)"
        );
        assert_eq!(
            collect_search_conditions(&keywords),
            vec![SearchCondition::Fts(
                "\"database\" OR \"sqlite\"".to_owned()
            )]
        );
    }

    // LIKEの特殊文字をエスケープし、値をプレースホルダーへ渡すSQL構造を確認する。
    #[test]
    fn builds_like_condition_with_escape_clause() {
        init_logger();
        let keywords = SearchKeyword::new("%_", SearchType::And).unwrap();
        let query = build_search_query::<TestFuzzySearchTarget>(&keywords);

        assert_eq!(
            query.sql(),
            "select test.id, test.content from test where 1 = 1 and test.content like ? escape '\\'"
        );
        assert_eq!(
            collect_search_conditions(&keywords),
            vec![SearchCondition::Like("%\\%\\_%".to_owned())]
        );
    }

    // LIKEのエスケープ文字であるバックスラッシュも変換できることを確認する。
    #[test]
    fn escapes_like_escape_character() {
        init_logger();
        assert_eq!(escape_like(r"%_\"), r"\%\_\\");
    }

    // FTS5の引用符を二重化し、外側を引用符で囲むことで検索語を安全なリテラルに変換することを確認する。
    #[test]
    fn escapes_fts_special_characters() {
        init_logger();
        let cases = [
            ("database", "\"database\""),
            ("a\"b", "\"a\"\"b\""),
            ("\"\"", "\"\"\"\"\"\""),
            ("", "\"\""),
            ("日本語 AND sqlite", "\"日本語 AND sqlite\""),
        ];

        for (input, expected) in cases {
            assert_eq!(escape_fts(input), expected, "input: {input:?}");
        }
    }

    // FTS5検索では外部コンテンツテーブルとrowidで関連付けるSQL構造を確認する。
    #[test]
    fn builds_fts_exists_condition() {
        init_logger();
        let keywords = SearchKeyword::new("database", SearchType::Or).unwrap();
        let query = build_search_query::<TestFuzzySearchTarget>(&keywords);

        assert_eq!(
            query.sql(),
            "select test.id, test.content from test where 1 = 0 or exists (select 1 from test_fts where test_fts.rowid = test.id and test_fts match ?)"
        );
        assert_eq!(
            collect_search_conditions(&keywords),
            vec![SearchCondition::Fts("\"database\"".to_owned())]
        );
    }
}
