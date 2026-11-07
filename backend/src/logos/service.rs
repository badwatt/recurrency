use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use crate::shared::error::{AppError, AppResult};

/// Internet logo sources: selfh.st icons, Dashboard Icons, DuckDuckGo
/// images and Brave images.
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";
const SELFHST_INDEX_URL: &str = "https://cdn.jsdelivr.net/gh/selfhst/icons/index.json";
const SELFHST_PNG_BASE: &str = "https://cdn.jsdelivr.net/gh/selfhst/icons/png/";
const DASHBOARD_INDEX_URL: &str =
    "https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/tree.json";
const DASHBOARD_PNG_BASE: &str = "https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/png/";
const DUCKDUCKGO_BASE: &str = "https://duckduckgo.com";
const BRAVE_IMAGES_URL: &str = "https://search.brave.com/images";
const DDG_FILE_TYPE_QUERY: &str = "filetype:png";

const INDEX_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const SEARCH_TTL: Duration = Duration::from_secs(60 * 60);
const SEARCH_CACHE_MAX_ENTRIES: usize = 128;
const MAX_RESULTS: usize = 60;
const MAX_ICON_RESULTS: usize = 24;
const MAX_DDG_RESULTS: usize = 24;
const MAX_BRAVE_RESULTS: usize = 16;

/// One search hit. `url` is the image to save, `thumbnail` is what the picker
/// renders.
#[derive(Debug, Clone, Serialize)]
pub struct LogoResult {
    pub url: String,
    pub thumbnail: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
struct SelfhstIcon {
    #[serde(rename = "Name", default)]
    name: String,
    #[serde(rename = "Reference", default)]
    reference: String,
    #[serde(rename = "PNG", default)]
    png: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DashboardIconsIndex {
    #[serde(default)]
    png: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DuckDuckGoImage {
    #[serde(default)]
    image: Option<String>,
    #[serde(default)]
    thumbnail: Option<String>,
    #[serde(default)]
    width: Option<i32>,
    #[serde(default)]
    height: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct DuckDuckGoResponse {
    #[serde(default)]
    results: Option<Vec<DuckDuckGoImage>>,
}

type IndexCache<T> = Mutex<Option<(Instant, T)>>;
type SearchCache = Mutex<HashMap<String, (Instant, Vec<LogoResult>)>>;

// The two icon indexes are ~1 MB combined and change rarely: keep them for a
// day. Search hits are cached for an hour because DuckDuckGo and Brave
// rate-limit aggressively.
static SELFHST_INDEX: LazyLock<IndexCache<Vec<SelfhstIcon>>> = LazyLock::new(|| Mutex::new(None));
static DASHBOARD_INDEX: LazyLock<IndexCache<Vec<String>>> = LazyLock::new(|| Mutex::new(None));
static SEARCH_CACHE: LazyLock<SearchCache> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Percent-encode every byte except ASCII alphanumerics and `-._~`.
fn encode_query(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn http_client() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| AppError::Internal(format!("Logo HTTP client: {error}")))
}

async fn fetch_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    headers: &[(&'static str, &'static str)],
) -> AppResult<T> {
    let mut request = client.get(url);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = request
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("GET {url}: {error}")))?;
    if !response.status().is_success() {
        return Err(AppError::Internal(format!(
            "GET {url}: HTTP {}",
            response.status()
        )));
    }
    response
        .json()
        .await
        .map_err(|error| AppError::Internal(format!("GET {url}: invalid JSON: {error}")))
}

async fn fetch_text(
    client: &reqwest::Client,
    url: &str,
    headers: &[(&'static str, &'static str)],
) -> AppResult<String> {
    let mut request = client.get(url);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = request
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("GET {url}: {error}")))?;
    if !response.status().is_success() {
        return Err(AppError::Internal(format!(
            "GET {url}: HTTP {}",
            response.status()
        )));
    }
    response
        .text()
        .await
        .map_err(|error| AppError::Internal(format!("GET {url}: invalid body: {error}")))
}

// ── Caches ──────────────────────────────────────────────────────────

async fn selfhst_index(client: &reqwest::Client) -> Option<Vec<SelfhstIcon>> {
    if let Some(cached) = cached_index(&SELFHST_INDEX) {
        return Some(cached);
    }
    match fetch_json::<Vec<SelfhstIcon>>(client, SELFHST_INDEX_URL, &[]).await {
        Ok(index) => {
            store_index(&SELFHST_INDEX, index.clone());
            Some(index)
        }
        Err(error) => {
            tracing::warn!("selfh.st index unavailable: {error}");
            SELFHST_INDEX
                .lock()
                .ok()
                .and_then(|cache| cache.as_ref().map(|(_, index)| index.clone()))
        }
    }
}

async fn dashboard_index(client: &reqwest::Client) -> Option<Vec<String>> {
    if let Some(cached) = cached_index(&DASHBOARD_INDEX) {
        return Some(cached);
    }
    match fetch_json::<DashboardIconsIndex>(client, DASHBOARD_INDEX_URL, &[]).await {
        Ok(index) => {
            store_index(&DASHBOARD_INDEX, index.png.clone());
            Some(index.png)
        }
        Err(error) => {
            tracing::warn!("Dashboard Icons index unavailable: {error}");
            DASHBOARD_INDEX
                .lock()
                .ok()
                .and_then(|cache| cache.as_ref().map(|(_, files)| files.clone()))
        }
    }
}

fn cached_index<T: Clone>(cache: &IndexCache<T>) -> Option<T> {
    cache.lock().ok().and_then(|cache| {
        cache
            .as_ref()
            .and_then(|(fetched, value)| (fetched.elapsed() < INDEX_TTL).then(|| value.clone()))
    })
}

fn store_index<T>(cache: &IndexCache<T>, value: T) {
    if let Ok(mut cache) = cache.lock() {
        *cache = Some((Instant::now(), value));
    }
}

fn cached_search(query: &str) -> Option<Vec<LogoResult>> {
    let cache = SEARCH_CACHE.lock().ok()?;
    let (fetched, results) = cache.get(query)?;
    (fetched.elapsed() < SEARCH_TTL).then(|| results.clone())
}

fn store_search(query: String, results: Vec<LogoResult>) {
    if let Ok(mut cache) = SEARCH_CACHE.lock() {
        if cache.len() >= SEARCH_CACHE_MAX_ENTRIES {
            cache.retain(|_, (fetched, _)| fetched.elapsed() < SEARCH_TTL);
        }
        cache.insert(query, (Instant::now(), results));
    }
}

// ── Icon sets (selfh.st + Dashboard Icons) ─────────────────────────

fn selfhst_results(index: &[SelfhstIcon], query: &str) -> Vec<LogoResult> {
    let needle = query.to_lowercase();
    index
        .iter()
        .filter(|icon| icon.png.eq_ignore_ascii_case("yes"))
        .filter(|icon| !icon.reference.is_empty())
        .filter(|icon| {
            icon.name.to_lowercase().contains(&needle)
                || icon.reference.to_lowercase().contains(&needle)
        })
        .map(|icon| {
            let url = format!("{SELFHST_PNG_BASE}{}.png", icon.reference);
            LogoResult {
                thumbnail: url.clone(),
                url,
                width: None,
                height: None,
            }
        })
        .collect()
}

fn dashboard_results(files: &[String], query: &str) -> Vec<LogoResult> {
    let needle = query.to_lowercase();
    files
        .iter()
        .filter_map(|file| {
            let slug = file.strip_suffix(".png")?;
            // Skip light/dark theme variants; the base icon is enough.
            if slug.ends_with("-light") || slug.ends_with("-dark") {
                return None;
            }
            if !slug.to_lowercase().contains(&needle) {
                return None;
            }
            let url = format!("{DASHBOARD_PNG_BASE}{file}");
            Some(LogoResult {
                thumbnail: url.clone(),
                url,
                width: None,
                height: None,
            })
        })
        .collect()
}

fn icon_results(mut results: Vec<LogoResult>) -> Vec<LogoResult> {
    // Shorter URLs usually mean a closer slug match.
    results.sort_by_key(|result| result.url.len());
    dedupe(results).into_iter().take(MAX_ICON_RESULTS).collect()
}

// ── DuckDuckGo images ─────────────────────────────────────────────

/// Extract the `vqd` token from DuckDuckGo's HTML.
/// Matches `vqd="?([\d-]+)"?` and skips non-numeric values.
pub(crate) fn extract_vqd(html: &str) -> Option<String> {
    let mut rest = html;
    while let Some(index) = rest.find("vqd=") {
        let tail = rest[index + 4..].trim_start_matches(['\'', '"']);
        let end = tail
            .find(|c: char| !(c.is_ascii_digit() || c == '-'))
            .unwrap_or(tail.len());
        if end > 0 {
            return Some(tail[..end].to_string());
        }
        rest = &rest[index + 4..];
    }
    None
}

async fn duckduckgo_vqd(client: &reqwest::Client, term: &str) -> Option<String> {
    let url = format!("{DUCKDUCKGO_BASE}/?q={}&ia=images", encode_query(term));
    let html = fetch_text(client, &url, &[]).await.ok()?;
    extract_vqd(&html)
}

async fn duckduckgo_images(client: &reqwest::Client, term: &str) -> Vec<LogoResult> {
    let Some(vqd) = duckduckgo_vqd(client, term).await else {
        return vec![];
    };
    let url = format!(
        "{DUCKDUCKGO_BASE}/i.js?l=us-en&o=json&q={}&vqd={}&f=,,transparent,Wide,&p=1",
        encode_query(term),
        encode_query(&vqd)
    );
    let response = match fetch_json::<DuckDuckGoResponse>(
        client,
        &url,
        &[("Accept", "application/json"), ("Referer", DUCKDUCKGO_BASE)],
    )
    .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!("DuckDuckGo image search failed: {error}");
            return vec![];
        }
    };
    response
        .results
        .unwrap_or_default()
        .into_iter()
        .filter_map(|result| {
            let image = result.image?;
            let thumbnail = result.thumbnail.unwrap_or_else(|| image.clone());
            Some(LogoResult {
                url: image,
                thumbnail,
                width: result.width,
                height: result.height,
            })
        })
        .collect()
}

async fn search_duckduckgo(client: &reqwest::Client, query: &str) -> Vec<LogoResult> {
    let term = format!("{query} logo");
    let mut results = duckduckgo_images(client, &format!("{term} {DDG_FILE_TYPE_QUERY}")).await;
    if results.is_empty() {
        results = duckduckgo_images(client, &term).await;
    }
    sort_png_first(&mut results);
    results.truncate(MAX_DDG_RESULTS);
    results
}

fn url_extension(url: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    path.rsplit_once('.')
        .map(|(_, extension)| extension.to_lowercase())
        .unwrap_or_default()
}

fn sort_png_first(results: &mut [LogoResult]) {
    // Stable sort: DuckDuckGo relevance order is preserved inside each group.
    results.sort_by_key(|result| usize::from(url_extension(&result.url) != "png"));
}

// ── Brave images ──────────────────────────────────────────────────

const BRAVE_IMAGE_PREFIX: &str = "https://imgs.search.brave.com/";

fn brave_fit_is_tiny(url: &str) -> bool {
    let Some(index) = url.find("rs:fit:") else {
        return false;
    };
    let dimensions = &url[index + "rs:fit:".len()..];
    let mut parts = dimensions.split([':', '/']);
    let width = parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0);
    let height = parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0);
    let largest = width.max(height);
    largest > 0 && largest <= 64
}

fn extract_brave_image_urls(html: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut seen = HashSet::new();
    let mut rest = html;
    while let Some(index) = rest.find(BRAVE_IMAGE_PREFIX) {
        let tail = &rest[index..];
        let end = tail
            .find(|c: char| {
                c == '"' || c == '\'' || c == '\\' || c == '<' || c == '>' || c.is_whitespace()
            })
            .unwrap_or(tail.len());
        let url = &tail[..end];
        if !brave_fit_is_tiny(url) && seen.insert(url.to_string()) {
            urls.push(url.to_string());
        }
        rest = &tail[end.max(BRAVE_IMAGE_PREFIX.len())..];
    }
    urls
}

async fn search_brave(client: &reqwest::Client, query: &str) -> Vec<LogoResult> {
    let term = format!("{query} logo");
    let url = format!("{BRAVE_IMAGES_URL}?q={}", encode_query(&term));
    let html = match fetch_text(
        client,
        &url,
        &[
            (
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
            ("Accept-Language", "en-US,en;q=0.5"),
            ("Referer", "https://search.brave.com/"),
        ],
    )
    .await
    {
        Ok(html) => html,
        Err(error) => {
            tracing::warn!("Brave image search failed: {error}");
            return vec![];
        }
    };
    extract_brave_image_urls(&html)
        .into_iter()
        .take(MAX_BRAVE_RESULTS)
        .map(|url| LogoResult {
            thumbnail: url.clone(),
            url,
            width: None,
            height: None,
        })
        .collect()
}

// ── Orchestration ─────────────────────────────────────────────────

fn dedupe(results: Vec<LogoResult>) -> Vec<LogoResult> {
    let mut seen = HashSet::new();
    results
        .into_iter()
        .filter(|result| seen.insert(result.url.clone()))
        .collect()
}

fn dedupe_and_limit(results: Vec<LogoResult>, limit: usize) -> Vec<LogoResult> {
    dedupe(results).into_iter().take(limit).collect()
}

pub async fn search_logos(query: &str) -> AppResult<Vec<LogoResult>> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(vec![]);
    }
    let cache_key = query.to_lowercase();
    if let Some(results) = cached_search(&cache_key) {
        return Ok(results);
    }

    let client = http_client()?;
    let (selfhst, dashboard, duckduckgo, brave) = tokio::join!(
        selfhst_index(&client),
        dashboard_index(&client),
        search_duckduckgo(&client, query),
        search_brave(&client, query),
    );

    let mut icons = Vec::new();
    if let Some(index) = selfhst {
        icons.extend(selfhst_results(&index, query));
    }
    if let Some(files) = dashboard {
        icons.extend(dashboard_results(&files, query));
    }

    let mut results = icon_results(icons);
    results.extend(duckduckgo);
    results.extend(brave);
    let results = dedupe_and_limit(results, MAX_RESULTS);
    if !results.is_empty() {
        store_search(cache_key, results.clone());
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_query_matches_non_alphanumeric() {
        assert_eq!(encode_query("Netflix"), "Netflix");
        assert_eq!(encode_query("a b&c"), "a%20b%26c");
        assert_eq!(encode_query("a-b"), "a-b");
    }

    #[test]
    fn extract_vqd_single_quotes() {
        assert_eq!(extract_vqd("vqd='123-456'"), Some("123-456".into()));
    }

    #[test]
    fn extract_vqd_double_quotes() {
        assert_eq!(extract_vqd("vqd=\"123-456\""), Some("123-456".into()));
    }

    #[test]
    fn extract_vqd_ampersand() {
        assert_eq!(extract_vqd("vqd=123-456&x=1"), Some("123-456".into()));
    }

    #[test]
    fn extract_vqd_missing() {
        assert_eq!(extract_vqd("no token here"), None);
    }

    #[test]
    fn extract_vqd_empty_token() {
        assert_eq!(extract_vqd("vqd=''"), None);
    }

    #[test]
    fn extract_vqd_skips_invalid_token() {
        assert_eq!(extract_vqd("vqd=abc vqd=4-99"), Some("4-99".into()));
    }

    #[test]
    fn extract_vqd_long_token() {
        let token = "4-".to_string() + &"1".repeat(200);
        let html = format!("vqd='{token}'");
        assert_eq!(extract_vqd(&html), Some(token));
    }

    #[test]
    fn selfhst_results_filter_and_build_urls() {
        let index = vec![
            SelfhstIcon {
                name: "Netflix".into(),
                reference: "netflix".into(),
                png: "Yes".into(),
            },
            SelfhstIcon {
                name: "Netflix Alt".into(),
                reference: "netflix-alt".into(),
                png: "No".into(),
            },
            SelfhstIcon {
                name: "Spotify".into(),
                reference: "spotify".into(),
                png: "Yes".into(),
            },
        ];
        let results = selfhst_results(&index, "netflix");
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].url,
            "https://cdn.jsdelivr.net/gh/selfhst/icons/png/netflix.png"
        );
    }

    #[test]
    fn dashboard_results_skip_theme_variants() {
        let files = vec![
            "netflix.png".to_string(),
            "netflix-dark.png".to_string(),
            "netflix-light.png".to_string(),
            "spotify.png".to_string(),
        ];
        let results = dashboard_results(&files, "netflix");
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].url,
            "https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/png/netflix.png"
        );
    }

    #[test]
    fn icon_results_sort_dedupe_and_limit() {
        let make = |url: &str| LogoResult {
            url: url.into(),
            thumbnail: url.into(),
            width: None,
            height: None,
        };
        let results = icon_results(vec![make("bbbb"), make("a"), make("bbbb"), make("cc")]);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].url, "a");
        assert_eq!(results[1].url, "cc");
        assert_eq!(results[2].url, "bbbb");
    }

    #[test]
    fn duckduckgo_response_maps_results() {
        let json = r#"{"results":[{"image":"https://x/a.png","thumbnail":"https://x/t.png","width":100,"height":50}]}"#;
        let response: DuckDuckGoResponse = serde_json::from_str(json).unwrap();
        let results: Vec<LogoResult> = response
            .results
            .unwrap_or_default()
            .into_iter()
            .filter_map(|result| {
                let image = result.image?;
                Some(LogoResult {
                    thumbnail: result.thumbnail.unwrap_or_else(|| image.clone()),
                    url: image,
                    width: result.width,
                    height: result.height,
                })
            })
            .collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://x/a.png");
        assert_eq!(results[0].thumbnail, "https://x/t.png");
    }

    #[test]
    fn png_results_sort_first_stably() {
        let make = |url: &str| LogoResult {
            url: url.into(),
            thumbnail: url.into(),
            width: None,
            height: None,
        };
        let mut results = vec![
            make("https://x/a.jpg"),
            make("https://x/b.png?x=1"),
            make("https://x/c.webp"),
        ];
        sort_png_first(&mut results);
        assert_eq!(results[0].url, "https://x/b.png?x=1");
        assert_eq!(results[1].url, "https://x/a.jpg");
        assert_eq!(results[2].url, "https://x/c.webp");
    }

    #[test]
    fn url_extension_ignores_query_and_fragment() {
        assert_eq!(url_extension("https://x/a.PNG?token=1#frag"), "png");
        assert_eq!(url_extension("https://x/no-extension"), "");
    }

    #[test]
    fn brave_extracts_unique_image_urls() {
        let html = r#"a "https://imgs.search.brave.com/ABC/rs:fit:500:0:1:0/g:ce/one" b
            https://imgs.search.brave.com/ABC/rs:fit:500:0:1:0/g:ce/one
            "https://imgs.search.brave.com/DEF/rs:fit:300:200:1:0/g:ce/two""#;
        let urls = extract_brave_image_urls(html);
        assert_eq!(urls.len(), 2);
        assert!(urls[0].contains("/one"));
        assert!(urls[1].contains("/two"));
    }

    #[test]
    fn brave_skips_tiny_favicon_proxies() {
        let html = r#""https://imgs.search.brave.com/ABC/rs:fit:32:32:1:0/g:ce/fav"
            "https://imgs.search.brave.com/DEF/rs:fit:500:0:1:0/g:ce/logo""#;
        let urls = extract_brave_image_urls(html);
        assert_eq!(urls.len(), 1);
        assert!(urls[0].contains("/logo"));
    }

    #[test]
    fn brave_accepts_urls_without_fit_segment() {
        let html = r#""https://imgs.search.brave.com/ABC/g:ce/logo""#;
        assert_eq!(extract_brave_image_urls(html).len(), 1);
    }

    #[test]
    fn dedupe_keeps_first_occurrence() {
        let make = |url: &str| LogoResult {
            url: url.into(),
            thumbnail: url.into(),
            width: None,
            height: None,
        };
        let results = dedupe(vec![make("a"), make("b"), make("a")]);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].url, "a");
        assert_eq!(results[1].url, "b");
    }

    #[tokio::test]
    async fn empty_query_returns_no_results_without_network() {
        let results = search_logos("   ").await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    #[ignore = "network: queries selfh.st, Dashboard Icons, DuckDuckGo and Brave"]
    async fn live_search_returns_internet_sources() {
        let iberdrola = search_logos("iberdrola").await.unwrap();
        assert!(!iberdrola.is_empty());
        assert!(
            iberdrola.iter().any(|result| {
                result.url.contains("imgs.search.brave.com") || result.url.contains("bing.net")
            }),
            "expected image-search results for a brand missing from the icon sets"
        );

        let netflix = search_logos("netflix").await.unwrap();
        assert!(
            netflix
                .iter()
                .any(|result| result.url.contains("cdn.jsdelivr.net")),
            "expected selfh.st or Dashboard Icons results for a known brand"
        );
    }
}
