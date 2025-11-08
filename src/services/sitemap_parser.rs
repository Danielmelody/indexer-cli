//! Sitemap parser service
//!
//! This module provides comprehensive XML sitemap parsing functionality with support for:
//! - Regular sitemaps and sitemap indexes
//! - HTTP/HTTPS URLs with gzip compression
//! - Recursive sitemap index parsing with depth limits
//! - URL filtering by pattern, date, and priority
//! - Size and URL count validation

use crate::types::error::IndexerError;
use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::Client;
use roxmltree::Document;
use std::collections::HashSet;
use std::io::Read;
use tracing::{debug, info, warn};

/// Maximum sitemap file size (50MB)
const MAX_SITEMAP_SIZE: usize = 50 * 1024 * 1024;

/// Maximum number of URLs in a sitemap (50,000 as per sitemap.org protocol)
const MAX_URLS: usize = 50_000;

/// Default maximum recursion depth for sitemap indexes
const DEFAULT_MAX_RECURSION_DEPTH: usize = 3;

/// Sitemap parser for extracting URLs from XML sitemaps
#[derive(Debug, Clone)]
pub struct SitemapParser {
    /// HTTP client for downloading sitemaps
    client: Client,
    /// Maximum recursion depth for sitemap indexes
    max_recursion_depth: usize,
    /// Maximum number of URLs to extract
    max_urls: usize,
}

/// Represents a URL entry from a sitemap
#[derive(Debug, Clone, PartialEq)]
pub struct SitemapUrl {
    /// The URL location
    pub loc: String,
    /// Last modification date (optional)
    pub lastmod: Option<DateTime<Utc>>,
    /// Change frequency (optional)
    pub changefreq: Option<String>,
    /// Priority (0.0 - 1.0, optional)
    pub priority: Option<f32>,
}

/// Filters for URL extraction
#[derive(Debug, Clone, Default)]
pub struct SitemapFilters {
    /// URL pattern to match (regex)
    pub url_pattern: Option<Regex>,
    /// Only include URLs modified after this date
    pub lastmod_after: Option<DateTime<Utc>>,
    /// Minimum priority (0.0 - 1.0)
    pub priority_min: Option<f32>,
}

/// Result of sitemap parsing operation
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Extracted URLs
    pub urls: Vec<SitemapUrl>,
    /// Total number of URLs found before filtering
    pub total_count: usize,
    /// Number of URLs after filtering
    pub filtered_count: usize,
}

impl SitemapParser {
    /// Create a new sitemap parser with default settings
    pub fn new() -> Result<Self, IndexerError> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; IndexerCLI/1.0)")
            .gzip(true)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| IndexerError::HttpRequestFailed {
                message: e.to_string(),
            })?;

        Ok(Self {
            client,
            max_recursion_depth: DEFAULT_MAX_RECURSION_DEPTH,
            max_urls: MAX_URLS,
        })
    }

    /// Create a new sitemap parser with custom settings
    pub fn with_config(
        max_recursion_depth: usize,
        max_urls: usize,
    ) -> Result<Self, IndexerError> {
        let mut parser = Self::new()?;
        parser.max_recursion_depth = max_recursion_depth;
        parser.max_urls = max_urls;
        Ok(parser)
    }

    /// Parse a sitemap from a URL
    ///
    /// This method:
    /// - Downloads the sitemap from the given URL
    /// - Automatically handles gzip compression
    /// - Recursively processes sitemap indexes
    /// - Applies filters if provided
    /// - Validates size and URL count limits
    pub async fn parse_sitemap(
        &self,
        url: &str,
        filters: Option<&SitemapFilters>,
    ) -> Result<ParseResult, IndexerError> {
        info!("Parsing sitemap from URL: {}", url);

        // Validate URL
        let parsed_url = url::Url::parse(url).map_err(|_| IndexerError::SitemapInvalidUrl {
            url: url.to_string(),
        })?;

        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            return Err(IndexerError::SitemapInvalidUrl {
                url: url.to_string(),
            });
        }

        // Download sitemap content
        let xml_content = self.download_sitemap(url).await?;

        // Parse and extract URLs
        self.parse_sitemap_content(&xml_content, filters, 0).await
    }

    /// Download sitemap content from URL
    async fn download_sitemap(&self, url: &str) -> Result<String, IndexerError> {
        debug!("Downloading sitemap from: {}", url);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| IndexerError::SitemapDownloadFailed {
                url: url.to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(IndexerError::SitemapDownloadFailed {
                url: url.to_string(),
                message: format!("HTTP {}", response.status()),
            });
        }

        // Check content length
        if let Some(content_length) = response.content_length() {
            if content_length as usize > MAX_SITEMAP_SIZE {
                return Err(IndexerError::SitemapTooLarge {
                    size: content_length as usize,
                    limit: MAX_SITEMAP_SIZE,
                });
            }
        }

        // Read response body
        let bytes = response
            .bytes()
            .await
            .map_err(|e| IndexerError::SitemapDownloadFailed {
                url: url.to_string(),
                message: e.to_string(),
            })?;

        // Check actual size
        if bytes.len() > MAX_SITEMAP_SIZE {
            return Err(IndexerError::SitemapTooLarge {
                size: bytes.len(),
                limit: MAX_SITEMAP_SIZE,
            });
        }

        // Check if content is gzipped
        let content = if Self::is_gzipped(&bytes) {
            debug!("Decompressing gzipped sitemap");
            Self::decompress_gzip(&bytes)?
        } else {
            String::from_utf8(bytes.to_vec()).map_err(|e| IndexerError::SitemapParseError {
                message: format!("Invalid UTF-8: {}", e),
            })?
        };

        Ok(content)
    }

    /// Parse sitemap XML content
    #[async_recursion]
    async fn parse_sitemap_content(
        &self,
        xml_content: &str,
        filters: Option<&SitemapFilters>,
        depth: usize,
    ) -> Result<ParseResult, IndexerError> {
        // Check recursion depth
        if depth > self.max_recursion_depth {
            return Err(IndexerError::SitemapRecursionLimitExceeded {
                limit: self.max_recursion_depth,
            });
        }

        // Parse XML
        let doc = Document::parse(xml_content).map_err(|e| IndexerError::SitemapInvalidXml {
            message: e.to_string(),
        })?;

        // Check if this is a sitemap index
        if Self::is_sitemap_index(&doc) {
            debug!("Detected sitemap index, recursively parsing child sitemaps");
            self.parse_sitemap_index(&doc, filters, depth).await
        } else {
            debug!("Detected regular sitemap, extracting URLs");
            let urls = self.extract_urls(&doc)?;
            let total_count = urls.len();

            // Apply filters
            let filtered_urls = if let Some(filters) = filters {
                self.filter_urls(urls, filters)
            } else {
                urls
            };

            let filtered_count = filtered_urls.len();

            Ok(ParseResult {
                urls: filtered_urls,
                total_count,
                filtered_count,
            })
        }
    }

    /// Parse a sitemap index and recursively fetch all child sitemaps
    async fn parse_sitemap_index(
        &self,
        doc: &Document<'_>,
        filters: Option<&SitemapFilters>,
        depth: usize,
    ) -> Result<ParseResult, IndexerError> {
        let mut all_urls = Vec::new();
        let mut total_count = 0;

        // Find all <sitemap> elements
        let sitemap_locs = self.extract_sitemap_index_locations(doc)?;

        info!(
            "Found {} sitemaps in index at depth {}",
            sitemap_locs.len(),
            depth
        );

        // Recursively parse each sitemap
        for sitemap_url in sitemap_locs {
            debug!("Parsing child sitemap: {}", sitemap_url);

            let xml_content = match self.download_sitemap(&sitemap_url).await {
                Ok(content) => content,
                Err(e) => {
                    warn!("Failed to download child sitemap {}: {}", sitemap_url, e);
                    continue;
                }
            };

            let result = match self
                .parse_sitemap_content(&xml_content, filters, depth + 1)
                .await
            {
                Ok(result) => result,
                Err(e) => {
                    warn!("Failed to parse child sitemap {}: {}", sitemap_url, e);
                    continue;
                }
            };

            total_count += result.total_count;
            all_urls.extend(result.urls);

            // Check URL limit
            if all_urls.len() > self.max_urls {
                warn!(
                    "Reached maximum URL limit of {}, stopping",
                    self.max_urls
                );
                all_urls.truncate(self.max_urls);
                break;
            }
        }

        let filtered_count = all_urls.len();

        Ok(ParseResult {
            urls: all_urls,
            total_count,
            filtered_count,
        })
    }

    /// Check if XML document is a sitemap index
    fn is_sitemap_index(doc: &Document) -> bool {
        doc.root_element()
            .descendants()
            .any(|n| n.has_tag_name("sitemap"))
    }

    /// Extract sitemap locations from a sitemap index
    fn extract_sitemap_index_locations(&self, doc: &Document) -> Result<Vec<String>, IndexerError> {
        let mut locations = Vec::new();

        for node in doc.root_element().descendants() {
            if node.has_tag_name("sitemap") {
                // Find <loc> child
                if let Some(loc_node) = node.descendants().find(|n| n.has_tag_name("loc")) {
                    if let Some(loc_text) = loc_node.text() {
                        locations.push(loc_text.to_string());
                    }
                }
            }
        }

        Ok(locations)
    }

    /// Extract URLs from a regular sitemap
    fn extract_urls(&self, doc: &Document) -> Result<Vec<SitemapUrl>, IndexerError> {
        let mut urls = Vec::new();
        let mut seen_urls = HashSet::new();

        for node in doc.root_element().descendants() {
            if node.has_tag_name("url") {
                if let Some(sitemap_url) = self.parse_url_element(&node) {
                    // Deduplicate
                    if seen_urls.insert(sitemap_url.loc.clone()) {
                        urls.push(sitemap_url);

                        // Check URL limit
                        if urls.len() >= self.max_urls {
                            warn!("Reached maximum URL limit of {}", self.max_urls);
                            break;
                        }
                    }
                }
            }
        }

        info!("Extracted {} unique URLs from sitemap", urls.len());
        Ok(urls)
    }

    /// Parse a single <url> element
    fn parse_url_element(&self, url_node: &roxmltree::Node) -> Option<SitemapUrl> {
        let mut loc = None;
        let mut lastmod = None;
        let mut changefreq = None;
        let mut priority = None;

        for child in url_node.children() {
            match child.tag_name().name() {
                "loc" => {
                    loc = child.text().map(|s| s.to_string());
                }
                "lastmod" => {
                    if let Some(text) = child.text() {
                        lastmod = Self::parse_datetime(text);
                    }
                }
                "changefreq" => {
                    changefreq = child.text().map(|s| s.to_string());
                }
                "priority" => {
                    if let Some(text) = child.text() {
                        priority = text.parse::<f32>().ok();
                    }
                }
                _ => {}
            }
        }

        loc.map(|loc| SitemapUrl {
            loc,
            lastmod,
            changefreq,
            priority,
        })
    }

    /// Filter URLs based on provided filters
    fn filter_urls(&self, urls: Vec<SitemapUrl>, filters: &SitemapFilters) -> Vec<SitemapUrl> {
        urls.into_iter()
            .filter(|url| {
                // Filter by URL pattern
                if let Some(pattern) = &filters.url_pattern {
                    if !pattern.is_match(&url.loc) {
                        return false;
                    }
                }

                // Filter by lastmod date
                if let Some(lastmod_after) = &filters.lastmod_after {
                    if let Some(url_lastmod) = &url.lastmod {
                        if url_lastmod < lastmod_after {
                            return false;
                        }
                    } else {
                        // No lastmod means we can't filter by date
                        return false;
                    }
                }

                // Filter by priority
                if let Some(priority_min) = filters.priority_min {
                    if let Some(url_priority) = url.priority {
                        if url_priority < priority_min {
                            return false;
                        }
                    } else {
                        // No priority means we can't filter by it
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Parse datetime from various sitemap date formats
    fn parse_datetime(date_str: &str) -> Option<DateTime<Utc>> {
        // Try ISO 8601 formats
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return Some(dt.with_timezone(&Utc));
        }

        // Try date only format (YYYY-MM-DD)
        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            return Some(
                naive_date
                    .and_hms_opt(0, 0, 0)?
                    .and_local_timezone(Utc)
                    .single()?,
            );
        }

        None
    }

    /// Check if bytes are gzipped
    fn is_gzipped(bytes: &[u8]) -> bool {
        bytes.len() >= 2 && bytes[0] == 0x1f && bytes[1] == 0x8b
    }

    /// Decompress gzip data
    fn decompress_gzip(bytes: &[u8]) -> Result<String, IndexerError> {
        use flate2::read::GzDecoder;

        let mut decoder = GzDecoder::new(bytes);
        let mut decompressed = String::new();

        decoder
            .read_to_string(&mut decompressed)
            .map_err(|e| IndexerError::SitemapParseError {
                message: format!("Gzip decompression failed: {}", e),
            })?;

        Ok(decompressed)
    }

    /// Parse sitemap XML from a string (for testing or direct XML input)
    pub fn parse_sitemap_xml(&self, xml_content: &str) -> Result<Vec<SitemapUrl>, IndexerError> {
        let doc = Document::parse(xml_content).map_err(|e| IndexerError::SitemapInvalidXml {
            message: e.to_string(),
        })?;

        self.extract_urls(&doc)
    }
}

impl Default for SitemapParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default SitemapParser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url_element() {
        let xml = r#"
            <url>
                <loc>https://example.com/page1</loc>
                <lastmod>2024-01-15</lastmod>
                <changefreq>weekly</changefreq>
                <priority>0.8</priority>
            </url>
        "#;

        let doc = Document::parse(xml).unwrap();
        let parser = SitemapParser::new().unwrap();
        let url_node = doc.root_element();

        let sitemap_url = parser.parse_url_element(&url_node).unwrap();

        assert_eq!(sitemap_url.loc, "https://example.com/page1");
        assert_eq!(sitemap_url.changefreq, Some("weekly".to_string()));
        assert_eq!(sitemap_url.priority, Some(0.8));
    }

    #[test]
    fn test_is_sitemap_index() {
        let index_xml = r#"
            <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
                <sitemap>
                    <loc>https://example.com/sitemap1.xml</loc>
                </sitemap>
            </sitemapindex>
        "#;

        let doc = Document::parse(index_xml).unwrap();
        assert!(SitemapParser::is_sitemap_index(&doc));

        let regular_xml = r#"
            <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
                <url>
                    <loc>https://example.com/page1</loc>
                </url>
            </urlset>
        "#;

        let doc = Document::parse(regular_xml).unwrap();
        assert!(!SitemapParser::is_sitemap_index(&doc));
    }

    #[test]
    fn test_parse_datetime() {
        // ISO 8601 with timezone
        let dt = SitemapParser::parse_datetime("2024-01-15T10:30:00Z");
        assert!(dt.is_some());

        // Date only
        let dt = SitemapParser::parse_datetime("2024-01-15");
        assert!(dt.is_some());

        // Invalid format
        let dt = SitemapParser::parse_datetime("invalid");
        assert!(dt.is_none());
    }

    #[test]
    fn test_is_gzipped() {
        let gzipped = vec![0x1f, 0x8b, 0x08, 0x00];
        assert!(SitemapParser::is_gzipped(&gzipped));

        let not_gzipped = b"<?xml version=\"1.0\"?>";
        assert!(!SitemapParser::is_gzipped(not_gzipped));
    }

    #[test]
    fn test_filter_urls() {
        let parser = SitemapParser::new().unwrap();

        let urls = vec![
            SitemapUrl {
                loc: "https://example.com/page1".to_string(),
                lastmod: None,
                changefreq: None,
                priority: Some(0.8),
            },
            SitemapUrl {
                loc: "https://example.com/page2".to_string(),
                lastmod: None,
                changefreq: None,
                priority: Some(0.5),
            },
        ];

        let filters = SitemapFilters {
            url_pattern: None,
            lastmod_after: None,
            priority_min: Some(0.7),
        };

        let filtered = parser.filter_urls(urls, &filters);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].loc, "https://example.com/page1");
    }
}
