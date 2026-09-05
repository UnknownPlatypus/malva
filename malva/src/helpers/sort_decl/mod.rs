use crate::config::DeclarationOrder;

mod concentric;
mod shorthand;
mod smacss;

pub fn sort<T>(decls: &mut [(&str, T)], order: &DeclarationOrder) {
    match order {
        DeclarationOrder::Alphabetical => insertion_sort(decls, alphabetical_key),
        DeclarationOrder::Smacss => insertion_sort(decls, smacss::rank_in_smacss),
        DeclarationOrder::Concentric => insertion_sort(decls, concentric::rank_in_concentric),
    }
}

/// Declarations only swap after a direct comparison. Unkeyed ones thus act as a barrier,
/// and a shorthand never crosses a longhand it overrides.
fn insertion_sort<'s, T, K: Ord>(decls: &mut [(&'s str, T)], key: impl Fn(&'s str) -> Option<K>) {
    let mut keys: Vec<_> = decls.iter().map(|(name, _)| key(name)).collect();
    for i in 1..decls.len() {
        let mut j = i;
        while j > 0
            && out_of_order(&keys[j - 1], &keys[j])
            && !overrides_either(decls[j - 1].0, decls[j].0)
        {
            decls.swap(j - 1, j);
            keys.swap(j - 1, j);
            j -= 1;
        }
    }
}

fn out_of_order<K: Ord>(prev: &Option<K>, next: &Option<K>) -> bool {
    matches!((prev, next), (Some(prev), Some(next)) if prev > next)
}

fn overrides_either(a: &str, b: &str) -> bool {
    let (a, b) = (strip_vendor_prefix(a), strip_vendor_prefix(b));
    // Property names are case-insensitive, but the table only has lowercase keys.
    if a.bytes().chain(b.bytes()).any(|c| c.is_ascii_uppercase()) {
        return overrides_either(&a.to_ascii_lowercase(), &b.to_ascii_lowercase());
    }
    shorthand::overrides(a, b) || shorthand::overrides(b, a)
}

fn alphabetical_key(name: &str) -> Option<&str> {
    (!name.starts_with("--")).then(|| strip_vendor_prefix(name))
}

fn rank_in_list(name: &str, list: &[&str]) -> Option<usize> {
    let name = strip_vendor_prefix(name);
    list.iter()
        .position(|property| property.eq_ignore_ascii_case(name))
}

fn strip_vendor_prefix(s: &str) -> &str {
    s.strip_prefix('-')
        .and_then(|s| {
            let trimmed = s.trim_start_matches(|c: char| c.is_ascii_alphanumeric());
            if s == trimmed { None } else { Some(trimmed) }
        })
        .and_then(|s| s.strip_prefix('-'))
        .unwrap_or(s)
}
