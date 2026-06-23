use std::collections::HashMap;

use crate::AppState;

use super::page_config::PageConfig;

pub fn resolve_page(state: &AppState, path: &str) -> Option<PageConfig> {
    let pages = state.pages.lock().unwrap();
    resolve_page_from_pages(&pages, path)
}

pub fn request_target(path: &str, query_string: &str) -> String {
    if query_string.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{query_string}")
    }
}

fn resolve_page_from_pages(pages: &HashMap<String, PageConfig>, path: &str) -> Option<PageConfig> {
    if let Some(page) = pages.get(path) {
        return Some(page.clone());
    }

    if let Some(page) = resolve_wildcard_page(pages, path) {
        return Some(page);
    }

    let Some(path_without_query) = path.split_once('?').map(|(path, _)| path) else {
        return None;
    };

    if let Some(page) = pages.get(path_without_query) {
        return Some(page.clone());
    }

    resolve_wildcard_page(pages, path_without_query)
}

fn resolve_wildcard_page(pages: &HashMap<String, PageConfig>, path: &str) -> Option<PageConfig> {
    pages
        .iter()
        .filter(|(pattern, _)| pattern.contains('*') && wildcard_matches(pattern, path))
        .max_by(|(left, _), (right, _)| {
            page_pattern_specificity(left).cmp(&page_pattern_specificity(right))
        })
        .map(|(_, page)| page.clone())
}

fn page_pattern_specificity(pattern: &str) -> (usize, usize) {
    let concrete_chars = pattern
        .chars()
        .filter(|character| *character != '*')
        .count();
    let wildcard_count = pattern
        .chars()
        .filter(|character| *character == '*')
        .count();

    (concrete_chars, usize::MAX - wildcard_count)
}

fn wildcard_matches(pattern: &str, value: &str) -> bool {
    if pattern == "*" || pattern == value {
        return true;
    }

    if !pattern.contains('*') {
        return false;
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    let mut rest = value;

    if let Some(first) = parts.first().filter(|first| !first.is_empty()) {
        let Some(stripped) = rest.strip_prefix(first) else {
            return false;
        };
        rest = stripped;
    }

    for part in parts.iter().skip(1).take(parts.len().saturating_sub(2)) {
        if part.is_empty() {
            continue;
        }

        let Some(index) = rest.find(part) else {
            return false;
        };
        rest = &rest[index + part.len()..];
    }

    if let Some(last) = parts.last().filter(|last| !last.is_empty()) {
        rest.ends_with(last)
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::PageType;

    #[test]
    fn resolve_page_matches_generic_path() {
        let pages = HashMap::from([(
            "/my/shop/*".to_string(),
            test_page_config("/my/shop/*", "generic-shop-page"),
        )]);

        let page = resolve_page_from_pages(&pages, "/my/shop/some-category").unwrap();

        assert_eq!(page.page_id, "generic-shop-page");
    }

    #[test]
    fn resolve_page_matches_query_parameter_pattern() {
        let pages = HashMap::from([(
            "/my/shop/search?query=*".to_string(),
            test_page_config("/my/shop/search?query=*", "search-page"),
        )]);

        let page = resolve_page_from_pages(&pages, "/my/shop/search?query=shoes").unwrap();

        assert_eq!(page.page_id, "search-page");
    }

    #[test]
    fn resolve_page_without_query_still_matches_request_with_query() {
        let pages = HashMap::from([(
            "/my/shop/search".to_string(),
            test_page_config("/my/shop/search", "search-page"),
        )]);

        let page = resolve_page_from_pages(&pages, "/my/shop/search?query=shoes").unwrap();

        assert_eq!(page.page_id, "search-page");
    }

    #[test]
    fn query_parameter_pattern_beats_generic_path_pattern() {
        let pages = HashMap::from([
            (
                "/my/shop/*".to_string(),
                test_page_config("/my/shop/*", "generic-shop-page"),
            ),
            (
                "/my/shop/search?query=*".to_string(),
                test_page_config("/my/shop/search?query=*", "search-page"),
            ),
        ]);

        let page = resolve_page_from_pages(&pages, "/my/shop/search?query=shoes").unwrap();

        assert_eq!(page.page_id, "search-page");
    }

    #[test]
    fn resolve_page_prefers_exact_path_over_generic_path() {
        let pages = HashMap::from([
            (
                "/my/shop/*".to_string(),
                test_page_config("/my/shop/*", "generic-shop-page"),
            ),
            (
                "/my/shop/cart.fancy".to_string(),
                test_page_config("/my/shop/cart.fancy", "cart-page"),
            ),
        ]);

        let page = resolve_page_from_pages(&pages, "/my/shop/cart.fancy").unwrap();

        assert_eq!(page.page_id, "cart-page");
    }

    #[test]
    fn resolve_page_prefers_more_specific_generic_path() {
        let pages = HashMap::from([
            (
                "/my/shop/*".to_string(),
                test_page_config("/my/shop/*", "generic-shop-page"),
            ),
            (
                "/my/shop/special-*".to_string(),
                test_page_config("/my/shop/special-*", "special-shop-page"),
            ),
        ]);

        let page = resolve_page_from_pages(&pages, "/my/shop/special-offers").unwrap();

        assert_eq!(page.page_id, "special-shop-page");
    }

    #[test]
    fn request_target_omits_empty_query_string() {
        assert_eq!(request_target("/index.html", ""), "/index.html");
    }

    #[test]
    fn request_target_includes_query_string() {
        assert_eq!(
            request_target("/my/shop/search", "query=shoes"),
            "/my/shop/search?query=shoes"
        );
    }

    #[test]
    fn wildcard_matches_infix_path_segments() {
        assert!(wildcard_matches(
            "/shop/*/index.html",
            "/shop/sneakers/index.html"
        ));
    }

    fn test_page_config(path: &str, page_id: &str) -> PageConfig {
        PageConfig {
            path: path.to_string(),
            page_id: page_id.to_string(),
            page_type: PageType::Rfa,
            template: "landing".to_string(),
            rfa: "landing_v1".to_string(),
            delivery: crate::page::PageDelivery::Composer,
            timeout_ms: 1000,
            content_type: "text/html; charset=utf-8".to_string(),
            data: HashMap::new(),
            interaction: None,
        }
    }
}
