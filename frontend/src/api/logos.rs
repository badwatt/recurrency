use crate::api::client::{ApiError, api_get, get_client, is_offline_mode};
use crate::models::logo::{LogoResult, LogoSearchResponse};
use serde::Deserialize;

/// Iconify public search API: free, no API key, CORS-enabled.
const ICONIFY_SEARCH_API: &str = "https://api.iconify.design/search";
/// Restrict results to brand/logo icon sets so UI icons (arrows, menus…)
/// never leak into the logo picker.
const ICONIFY_LOGO_PREFIXES: &str = "logos,selfhst,thesvg-color,streamline-color,streamline-logos,thesvg,simple-icons,arcticons,cbi";
const ICONIFY_LIMIT: usize = 60;
const ICONIFY_ICON_SIZE: u32 = 160;
/// Tint for monochrome icon sets so they stay visible on every theme.
/// Palette (full-colour) icon sets ignore this parameter.
const MONO_ICON_TINT: &str = "94a3b8";

/// Wikimedia APIs: free, no key, CORS-enabled. Used as a fallback for brands
/// that are not in the icon sets (utilities, banks, local companies…).
const WIKIDATA_API: &str = "https://www.wikidata.org/w/api.php";
const COMMONS_API: &str = "https://commons.wikimedia.org/w/api.php";
const WIKI_USER_AGENT: &str = concat!(
    "Recurrency/",
    env!("CARGO_PKG_VERSION"),
    " (self-hosted subscription tracker; logo search)"
);
const WIKI_LOGO_WIDTH: u32 = 256;
const COMMONS_SEARCH_LIMIT: usize = 12;
const COMMONS_LOGO_LIMIT: usize = 8;

#[derive(Debug, Deserialize)]
struct IconifySearchResponse {
    #[serde(default)]
    icons: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct WikidataSearchResponse {
    #[serde(default)]
    search: Vec<WikidataSearchHit>,
}

#[derive(Debug, Deserialize)]
struct WikidataSearchHit {
    id: String,
}

#[derive(Debug, Deserialize)]
struct WikidataClaimsResponse {
    #[serde(default)]
    claims: std::collections::HashMap<String, Vec<WikidataClaim>>,
}

#[derive(Debug, Deserialize)]
struct WikidataClaim {
    mainsnak: WikidataSnak,
}

#[derive(Debug, Deserialize)]
struct WikidataSnak {
    #[serde(default)]
    datavalue: Option<WikidataSnakValue>,
}

#[derive(Debug, Deserialize)]
struct WikidataSnakValue {
    value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct CommonsQueryResponse {
    #[serde(default)]
    query: Option<CommonsQuery>,
}

#[derive(Debug, Deserialize)]
struct CommonsQuery {
    #[serde(default)]
    pages: std::collections::HashMap<String, CommonsPage>,
}

#[derive(Debug, Deserialize)]
struct CommonsPage {
    title: String,
    #[serde(default)]
    index: Option<i32>,
    #[serde(default)]
    imageinfo: Vec<CommonsImageInfo>,
}

#[derive(Debug, Deserialize)]
struct CommonsImageInfo {
    #[serde(default)]
    thumburl: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    mime: Option<String>,
}

/// Logos found through Wikidata, plus the Commons file names so the Commons
/// search can skip duplicates.
#[derive(Default)]
struct WikiLogos {
    logos: Vec<LogoResult>,
    files: Vec<String>,
}

/// Percent-encode every byte except ASCII alphanumerics and `-._~`.
fn percent_encode(input: &str) -> String {
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

fn iconify_search_url(query: &str) -> String {
    format!(
        "{ICONIFY_SEARCH_API}?query={}&limit={ICONIFY_LIMIT}&prefixes={ICONIFY_LOGO_PREFIXES}",
        percent_encode(query.trim())
    )
}

/// Convert an Iconify icon id (`logos:netflix`) into a renderable SVG URL.
fn iconify_icon_url(icon: &str) -> Option<String> {
    let (prefix, name) = icon.split_once(':')?;
    if prefix.is_empty() || name.is_empty() {
        return None;
    }
    Some(format!(
        "https://api.iconify.design/{prefix}/{name}.svg?height={ICONIFY_ICON_SIZE}&color=%23{MONO_ICON_TINT}"
    ))
}

/// Words to retry when a multi-word query has no exact brand match.
/// Most specific first: longer words win, ties keep their input order.
fn fallback_query_tokens(query: &str) -> Vec<String> {
    let mut tokens: Vec<&str> = Vec::new();
    for token in query.split(|c: char| !c.is_alphanumeric()) {
        let token = token.trim();
        if token.chars().count() >= 3 && !tokens.contains(&token) {
            tokens.push(token);
        }
    }
    tokens.sort_by_key(|token| std::cmp::Reverse(token.chars().count()));
    tokens.into_iter().take(2).map(str::to_string).collect()
}

/// Iconify exposes `-dark`/`-light` variants as separate icons. When the base
/// icon is also in the result set the variants are redundant (and often
/// invisible on one of the themes), so drop them.
fn drop_redundant_variants(icons: Vec<String>) -> Vec<String> {
    let all: std::collections::HashSet<String> = icons.iter().cloned().collect();
    icons
        .into_iter()
        .filter(|icon| {
            icon.strip_suffix("-dark")
                .or_else(|| icon.strip_suffix("-light"))
                .is_none_or(|base| !all.contains(base))
        })
        .collect()
}

async fn query_iconify(query: &str) -> Result<Vec<LogoResult>, ApiError> {
    let response = get_client()
        .get(iconify_search_url(query))
        .send()
        .await
        .map_err(|e| ApiError::Network(format!("Logo search failed: {e}")))?;

    if !response.status().is_success() {
        return Err(ApiError::Http(
            response.status().as_u16(),
            "Logo search failed".into(),
        ));
    }

    let body: IconifySearchResponse = response
        .json()
        .await
        .map_err(|e| ApiError::Parse(format!("Invalid logo search response: {e}")))?;

    Ok(drop_redundant_variants(body.icons)
        .iter()
        .filter_map(|icon| iconify_icon_url(icon))
        .map(|url| LogoResult {
            thumbnail: url.clone(),
            url,
            width: None,
            height: None,
        })
        .collect())
}

/// Wikimedia requires a descriptive User-Agent on non-browser clients.
fn wiki_get(url: &str) -> reqwest::RequestBuilder {
    let request = get_client().get(url);
    #[cfg(not(target_arch = "wasm32"))]
    let request = request.header("User-Agent", WIKI_USER_AGENT);
    request
}

async fn wiki_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, ApiError> {
    let response = wiki_get(url)
        .send()
        .await
        .map_err(|e| ApiError::Network(format!("Wiki request failed: {e}")))?;
    if !response.status().is_success() {
        return Err(ApiError::Http(
            response.status().as_u16(),
            "Wiki request failed".into(),
        ));
    }
    response
        .json()
        .await
        .map_err(|e| ApiError::Parse(format!("Invalid wiki response: {e}")))
}

/// Stable Commons URL for a file name, rendered as a PNG thumbnail.
fn commons_file_url(file: &str) -> String {
    format!(
        "https://commons.wikimedia.org/wiki/Special:FilePath/{}?width={WIKI_LOGO_WIDTH}",
        percent_encode(file)
    )
}

fn normalize_file_name(name: &str) -> String {
    name.trim()
        .trim_start_matches("File:")
        .replace('_', " ")
        .to_lowercase()
}

/// Rank Commons files: explicit logos first, photos/buildings last, vector
/// logos above bitmaps. `index` preserves the search relevance for ties.
fn commons_result_score(title: &str, mime: &str) -> i32 {
    let title = title.to_lowercase();
    let mut score = 0;
    if ["logo", "logotipo", "wordmark", "isotipo", "isotype"]
        .iter()
        .any(|keyword| title.contains(keyword))
    {
        score += 5;
    }
    if [
        "headquarters",
        "building",
        "tower",
        "office",
        "tapa",
        "con logo",
        "photo",
        "hq",
    ]
    .iter()
    .any(|keyword| title.contains(keyword))
    {
        score -= 10;
    }
    match mime {
        "image/svg+xml" => score + 3,
        "image/png" => score + 2,
        "image/gif" => score + 1,
        _ => score,
    }
}

fn commons_pages_to_logos(mut pages: Vec<CommonsPage>) -> Vec<LogoResult> {
    pages.retain(|page| {
        page.imageinfo
            .first()
            .and_then(|info| info.mime.as_deref())
            .is_some_and(|mime| mime.starts_with("image/"))
    });
    pages.sort_by_key(|page| {
        let mime = page
            .imageinfo
            .first()
            .and_then(|info| info.mime.as_deref())
            .unwrap_or("");
        (
            std::cmp::Reverse(commons_result_score(&page.title, mime)),
            page.index.unwrap_or(i32::MAX),
        )
    });
    pages
        .into_iter()
        .filter_map(|page| {
            let info = page.imageinfo.into_iter().next()?;
            let url = info.thumburl.or(info.url)?;
            Some(LogoResult {
                thumbnail: url.clone(),
                url,
                width: None,
                height: None,
            })
        })
        .take(COMMONS_LOGO_LIMIT)
        .collect()
}

fn dedupe_logos(logos: Vec<LogoResult>) -> Vec<LogoResult> {
    let mut seen = std::collections::HashSet::new();
    logos
        .into_iter()
        .filter(|logo| seen.insert(logo.url.clone()))
        .collect()
}

/// Official logo(s) of the entity from Wikidata property P154.
async fn wikidata_logos(query: &str) -> WikiLogos {
    let search_url = format!(
        "{WIKIDATA_API}?action=wbsearchentities&search={}&language=en&uselang=en&limit=1&format=json&origin=*",
        percent_encode(query)
    );
    let Ok(search) = wiki_json::<WikidataSearchResponse>(&search_url).await else {
        return WikiLogos::default();
    };
    let Some(hit) = search.search.first() else {
        return WikiLogos::default();
    };

    let claims_url = format!(
        "{WIKIDATA_API}?action=wbgetclaims&entity={}&property=P154&format=json&origin=*",
        percent_encode(&hit.id)
    );
    let Ok(claims) = wiki_json::<WikidataClaimsResponse>(&claims_url).await else {
        return WikiLogos::default();
    };

    let mut files: Vec<String> = Vec::new();
    for claim in claims.claims.get("P154").into_iter().flatten() {
        let Some(value) = claim
            .mainsnak
            .datavalue
            .as_ref()
            .and_then(|value| value.value.as_str())
        else {
            continue;
        };
        if !files.iter().any(|file| file == value) {
            files.push(value.to_string());
        }
    }

    let logos = files
        .iter()
        .map(|file| {
            let url = commons_file_url(file);
            LogoResult {
                thumbnail: url.clone(),
                url,
                width: None,
                height: None,
            }
        })
        .collect();

    WikiLogos { logos, files }
}

/// Commons file search scoped to the File namespace (`"{query} logo"`).
async fn commons_logos(query: &str, skip_files: &[String]) -> Vec<LogoResult> {
    let search_term = format!("{query} logo");
    let url = format!(
        "{COMMONS_API}?action=query&generator=search&gsrsearch={}&gsrnamespace=6&gsrlimit={COMMONS_SEARCH_LIMIT}&prop=imageinfo&iiprop=url%7Cmime&iiurlwidth={WIKI_LOGO_WIDTH}&format=json&origin=*",
        percent_encode(&search_term)
    );
    let Ok(response) = wiki_json::<CommonsQueryResponse>(&url).await else {
        return vec![];
    };
    let Some(commons) = response.query else {
        return vec![];
    };
    let skip: std::collections::HashSet<String> = skip_files
        .iter()
        .map(|file| normalize_file_name(file))
        .collect();
    let pages: Vec<CommonsPage> = commons
        .pages
        .into_values()
        .filter(|page| !skip.contains(&normalize_file_name(&page.title)))
        .collect();
    commons_pages_to_logos(pages)
}

/// Server-side search. The backend queries selfh.st icons, Dashboard Icons,
/// DuckDuckGo images and Brave images.
async fn backend_search(query: &str) -> Result<LogoSearchResponse, ApiError> {
    let encoded = percent_encode(query);
    api_get(&format!("/api/logos/search?q={encoded}")).await
}

/// Search logos. In server mode the backend is queried first; offline mode and
/// empty/failed server results fall back to the client-side sources (Iconify,
/// Wikidata P154 and Wikimedia Commons).
pub async fn search(query: &str) -> Result<LogoSearchResponse, ApiError> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(LogoSearchResponse { results: vec![] });
    }

    if !is_offline_mode() {
        match backend_search(query).await {
            Ok(response) if !response.results.is_empty() => return Ok(response),
            Ok(_) => {}
            Err(error) => log::warn!("Server logo search failed: {error}"),
        }
    }

    client_side_search(query).await
}

/// Client-side fallback used in offline mode or when the backend has no match.
/// Iconify first (full query, then its most specific words), then the official
/// Wikidata logo and Wikimedia Commons file search.
async fn client_side_search(query: &str) -> Result<LogoSearchResponse, ApiError> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(LogoSearchResponse { results: vec![] });
    }

    // Iconify being unreachable should not prevent the Wikimedia fallback.
    let mut results = match query_iconify(query).await {
        Ok(results) => results,
        Err(error) => {
            log::warn!("Iconify logo search failed: {error}");
            Vec::new()
        }
    };
    if results.is_empty() {
        for token in fallback_query_tokens(query) {
            if token.eq_ignore_ascii_case(query) {
                continue;
            }
            results = query_iconify(&token).await.unwrap_or_default();
            if !results.is_empty() {
                break;
            }
        }
    }

    // Icon sets only cover popular brands. For everything else (utilities,
    // banks, local companies) fall back to the official Wikidata logo and
    // Wikimedia Commons file search before giving up.
    if results.is_empty() {
        let wiki = wikidata_logos(query).await;
        let mut logos = wiki.logos;
        logos.extend(commons_logos(query, &wiki.files).await);
        results = dedupe_logos(logos);
    }

    Ok(LogoSearchResponse { results })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logos_search_url_format() {
        let query = "netflix";
        let path = format!("/api/logos/search?q={}", query);
        assert_eq!(path, "/api/logos/search?q=netflix");
    }

    #[test]
    fn logos_search_url_special_chars() {
        let query = "my service";
        let path = format!("/api/logos/search?q={}", query);
        assert!(path.contains("my service"));
    }

    #[test]
    fn logo_result_fields() {
        use crate::models::logo::LogoResult;
        let r = LogoResult {
            url: "https://logo.clearbit.com/netflix.com".into(),
            thumbnail: "https://logo.clearbit.com/netflix.com".into(),
            width: None,
            height: None,
        };
        assert!(r.url.starts_with("https://"));
        assert!(r.thumbnail.starts_with("https://"));
    }

    #[test]
    fn logo_result_serialization() {
        use crate::models::logo::LogoResult;
        let r = LogoResult {
            url: "https://logo.clearbit.com/spotify.com".into(),
            thumbnail: "https://logo.clearbit.com/spotify.com".into(),
            width: None,
            height: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("url"));
        let back: LogoResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.url, r.url);
    }

    #[test]
    fn logo_search_response_empty() {
        use crate::models::logo::LogoSearchResponse;
        let resp = LogoSearchResponse { results: vec![] };
        assert!(resp.results.is_empty());
    }

    #[test]
    fn logo_search_response_multiple() {
        use crate::models::logo::{LogoResult, LogoSearchResponse};
        let resp = LogoSearchResponse {
            results: vec![
                LogoResult {
                    url: "https://a.com/a.png".into(),
                    thumbnail: "https://a.com/a_thumb.png".into(),
                    width: None,
                    height: None,
                },
                LogoResult {
                    url: "https://b.com/b.png".into(),
                    thumbnail: "https://b.com/b_thumb.png".into(),
                    width: None,
                    height: None,
                },
            ],
        };
        assert_eq!(resp.results.len(), 2);
    }

    #[test]
    fn logos_url_encoded() {
        let query = "netflix+tv";
        let path = format!("/api/logos/search?q={}", query);
        assert!(path.contains("netflix+tv"));
    }

    #[test]
    fn logos_url_unicode() {
        let query = "Netflix日本";
        let path = format!("/api/logos/search?q={}", query);
        assert!(path.contains("Netflix"));
    }

    // ── Iconify search ──
    #[test]
    fn iconify_search_url_encodes_query() {
        let url = iconify_search_url("netflix tv");
        assert!(url.starts_with(ICONIFY_SEARCH_API));
        assert!(url.contains("query=netflix%20tv"));
        assert!(url.contains(&format!("limit={ICONIFY_LIMIT}")));
        assert!(url.contains("prefixes=logos"));
    }

    #[test]
    fn iconify_search_url_encodes_unicode() {
        let url = iconify_search_url("日本");
        assert!(url.contains("query=%E6%97%A5%E6%9C%AC"));
    }

    #[test]
    fn iconify_icon_url_builds_svg_with_tint() {
        assert_eq!(
            iconify_icon_url("logos:netflix").as_deref(),
            Some("https://api.iconify.design/logos/netflix.svg?height=160&color=%2394a3b8")
        );
    }

    #[test]
    fn iconify_icon_url_rejects_malformed_ids() {
        assert!(iconify_icon_url("netflix").is_none());
        assert!(iconify_icon_url(":netflix").is_none());
        assert!(iconify_icon_url("logos:").is_none());
    }

    #[test]
    fn iconify_response_parses_icons() {
        let json = r#"{"icons":["logos:netflix","selfhst:spotify"],"total":2,"collections":{}}"#;
        let resp: IconifySearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.icons.len(), 2);
    }

    #[test]
    fn iconify_response_defaults_to_empty_icons() {
        let resp: IconifySearchResponse = serde_json::from_str("{}").unwrap();
        assert!(resp.icons.is_empty());
    }

    // ── Multi-word queries ──
    #[test]
    fn fallback_tokens_prioritize_the_brand_word() {
        assert_eq!(
            fallback_query_tokens("Ollama Cloud"),
            vec!["Ollama", "Cloud"]
        );
        assert_eq!(
            fallback_query_tokens("Spotify Premium"),
            vec!["Spotify", "Premium"]
        );
    }

    #[test]
    fn fallback_tokens_strip_domain_parts() {
        assert_eq!(fallback_query_tokens("openai.com"), vec!["openai", "com"]);
    }

    #[test]
    fn fallback_tokens_skip_short_words_and_duplicates() {
        assert_eq!(fallback_query_tokens("my gym"), vec!["gym"]);
        assert_eq!(fallback_query_tokens("ollama ollama"), vec!["ollama"]);
    }

    // ── Icon variant cleanup ──
    #[test]
    fn drop_variants_when_base_exists() {
        let icons = vec![
            "selfhst:ollama".to_string(),
            "selfhst:ollama-dark".to_string(),
            "selfhst:ollama-light".to_string(),
            "logos:ollama".to_string(),
        ];
        assert_eq!(
            drop_redundant_variants(icons),
            vec!["selfhst:ollama", "logos:ollama"]
        );
    }

    #[test]
    fn keep_variants_when_base_is_missing() {
        let icons = vec![
            "thesvg-color:ollama-light".to_string(),
            "thesvg-color:ollama-dark".to_string(),
        ];
        assert_eq!(drop_redundant_variants(icons.clone()), icons);
    }

    #[test]
    fn drop_redundant_variants_keeps_regular_icons() {
        let icons = vec![
            "logos:netflix".to_string(),
            "simple-icons:netflix".to_string(),
        ];
        assert_eq!(drop_redundant_variants(icons.clone()), icons);
    }

    // ── Wikimedia fallback ──
    fn commons_page(title: &str, mime: &str, index: i32, url: &str) -> CommonsPage {
        CommonsPage {
            title: title.to_string(),
            index: Some(index),
            imageinfo: vec![CommonsImageInfo {
                thumburl: Some(url.to_string()),
                url: Some(url.to_string()),
                mime: Some(mime.to_string()),
            }],
        }
    }

    #[test]
    fn commons_file_url_encodes_the_file_name() {
        assert_eq!(
            commons_file_url("Logo Iberdrola (2023).svg"),
            "https://commons.wikimedia.org/wiki/Special:FilePath/Logo%20Iberdrola%20%282023%29.svg?width=256"
        );
    }

    #[test]
    fn normalize_file_name_strips_prefix_underscores_and_case() {
        assert_eq!(
            normalize_file_name("File:Logo_Iberdrola_(2023).svg"),
            "logo iberdrola (2023).svg"
        );
    }

    #[test]
    fn commons_score_prefers_logos_over_photos() {
        assert!(
            commons_result_score("File:Logo Iberdrola (2023).svg", "image/svg+xml")
                > commons_result_score("File:Tapa Iberdrola con logo.jpg", "image/jpeg")
        );
    }

    #[test]
    fn commons_score_prefers_vector_over_bitmap() {
        assert!(
            commons_result_score("File:Repsol logo.svg", "image/svg+xml")
                > commons_result_score("File:Repsol logo.png", "image/png")
        );
    }

    #[test]
    fn commons_pages_filter_non_images_and_sort_logos_first() {
        let pages = vec![
            commons_page("File:Netflix headquarters.jpg", "image/jpeg", 1, "photo"),
            commons_page("File:Netflix logo.svg", "image/svg+xml", 3, "logo"),
            commons_page("File:Netflix and Moodle.JPG", "image/jpeg", 2, "photo2"),
            commons_page("File:Netflix logo.pdf", "application/pdf", 4, "pdf"),
        ];
        let logos = commons_pages_to_logos(pages);
        // The PDF is dropped; photos stay but rank below the actual logo.
        assert_eq!(logos.len(), 3);
        assert_eq!(logos[0].url, "logo");
        assert_eq!(logos[2].url, "photo");
    }

    #[test]
    fn commons_pages_skip_results_without_image_url() {
        let mut page = commons_page("File:Logo.svg", "image/svg+xml", 1, "ignored");
        page.imageinfo[0].thumburl = None;
        page.imageinfo[0].url = None;
        assert!(commons_pages_to_logos(vec![page]).is_empty());
    }

    #[test]
    fn wikidata_claims_extract_p154_files() {
        let json = r#"{"claims":{"P154":[{"mainsnak":{"datavalue":{"value":"Logo Iberdrola (2023).svg","type":"string"}}}]}}"#;
        let claims: WikidataClaimsResponse = serde_json::from_str(json).unwrap();
        let files: Vec<&str> = claims
            .claims
            .get("P154")
            .into_iter()
            .flatten()
            .filter_map(|claim| {
                claim
                    .mainsnak
                    .datavalue
                    .as_ref()
                    .and_then(|value| value.value.as_str())
            })
            .collect();
        assert_eq!(files, vec!["Logo Iberdrola (2023).svg"]);
    }

    #[test]
    fn wikidata_search_parses_entity_id() {
        let json = r#"{"search":[{"id":"Q1437502","label":"Iberdrola"}]}"#;
        let response: WikidataSearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.search[0].id, "Q1437502");
    }

    #[test]
    fn commons_response_parses_generator_results() {
        let json = r#"{"query":{"pages":{"1":{"title":"File:Logo.svg","index":2,"imageinfo":[{"thumburl":"https://x/t.png","url":"https://x/o.svg","mime":"image/svg+xml"}]}}}}"#;
        let response: CommonsQueryResponse = serde_json::from_str(json).unwrap();
        let pages: Vec<CommonsPage> = response.query.unwrap().pages.into_values().collect();
        assert_eq!(pages[0].title, "File:Logo.svg");
        assert_eq!(pages[0].imageinfo[0].mime.as_deref(), Some("image/svg+xml"));
    }

    #[test]
    fn search_prefers_the_backend_logo_endpoint() {
        let src = include_str!("logos.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or("");
        assert!(
            prod.contains("/api/logos/search"),
            "server mode must query the backend logo search"
        );
        assert!(
            prod.contains("client_side_search"),
            "offline mode must keep the client-side fallback"
        );
        assert!(
            prod.contains("is_offline_mode"),
            "offline mode must skip the backend request"
        );
    }

    #[test]
    fn dedupe_logos_keeps_first_occurrence() {
        let make = |url: &str| LogoResult {
            url: url.to_string(),
            thumbnail: url.to_string(),
            width: None,
            height: None,
        };
        let logos = dedupe_logos(vec![make("a"), make("b"), make("a")]);
        assert_eq!(logos.len(), 2);
        assert_eq!(logos[0].url, "a");
        assert_eq!(logos[1].url, "b");
    }
}
