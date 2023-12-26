//! The `qwant` module handles the scraping of results from the qwant search engine
//! by querying the upstream qwant search engine with user provided query and with a page
//! number if provided.

use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::Html;
use std::collections::HashMap;

use super::search_result_parser::SearchResultParser;
use crate::models::aggregation_models::SearchResult;
use crate::models::engine_models::{EngineError, SearchEngine};
use error_stack::{Report, Result, ResultExt};

/// A new Qwant engine type defined in-order to implement the `SearchEngine` trait which allows to
/// reduce code duplication as well as allows to create vector of different search engines easily.

pub struct Qwant {
    /// The parser used to extract search results from HTML documents.
    parser: SearchResultParser,
}

impl Qwant {
    /// Creates a new instance of Qwant with a default configuration.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing `Qwant` if successful, otherwise an `EngineError`.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            parser: SearchResultParser::new(
                "._2NDle nt3hI",
                "._2NDle nt3hI",
                "._35zId _3A7p7 RMB_d eoseI>a",
                "._35zId _3A7p7 RMB_d eoseI>a",
                "._2-LMx XqdKF _1UMq0 _29nLp _3PXjk>span",
            )?,
        })
    }
}

#[async_trait::async_trait]
impl SearchEngine for Qwant {
    /// Retrieves search results from Qwant based on the provided query, page, user agent, and client.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query.
    /// * `page` - The page number for pagination.
    /// * `user_agent` - The user agent string.
    /// * `client` - The reqwest client for making HTTP requests.
    /// * `_safe_search` - A parameter for safe search (not currently used).
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `HashMap` of search results if successful, otherwise an `EngineError`.
    /// The `Err` variant is explicit for better documentation.
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        _safe_search: u8,
    ) -> Result<HashMap<String, SearchResult>, EngineError> {
        // Page number can be missing or empty string and so appropriate handling is required
        // so that upstream server recieves valid page number.
        let url: String = match page {
            1 | 0 => {
                format!("https://www.qwant.com/?q={query}&s=1")
            }
            _ => {
                format!("https://www.qwant.com/?q={query}&s={page}",)
            }
        };

        // initializing HeaderMap and adding appropriate headers.
        let header_map = HeaderMap::try_from(&HashMap::from([
            ("USER_AGENT".to_string(), user_agent.to_string()),
            ("REFERER".to_string(), "https://google.com/".to_string()),
            (
                "CONTENT_TYPE".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ),
            (
                "COOKIE".to_string(),
                "ab_test_group=1; home=daily".to_string(),
            ),
        ]))
        .change_context(EngineError::UnexpectedError)?;

        let document: Html = Html::parse_document(
            &Qwant::fetch_html_from_upstream(self, &url, header_map, client).await?,
        );

        if self.parser.parse_for_no_results(&document).next().is_some() {
            return Err(Report::new(EngineError::EmptyResultSet));
        }

        // scrape all the results from the html
        self.parser
            .parse_for_results(&document, |title, url, desc| {
                Some(SearchResult::new(
                    title.inner_html().trim(),
                    url.inner_html().trim(),
                    desc.inner_html().trim(),
                    &["qwant"],
                ))
            })
    }
}
