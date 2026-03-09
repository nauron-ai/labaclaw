pub(super) fn normalize_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

pub(super) fn normalize_lid(value: &str) -> Option<String> {
    split_address_user(value).map(ToOwned::to_owned)
}

pub(super) fn normalize_phone_number(value: &str) -> Option<String> {
    let digits: String = split_address_user(value)?
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect();
    (!digits.is_empty()).then(|| format!("+{digits}"))
}

fn split_address_user(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let after_at = trimmed
        .split_once('@')
        .map(|(user, _)| user)
        .unwrap_or(trimmed);

    let user = after_at
        .split_once(':')
        .map(|(user, _)| user)
        .unwrap_or(after_at)
        .trim()
        .trim_start_matches('+');

    (!user.is_empty()).then_some(user)
}

#[cfg(test)]
mod tests {
    use super::{normalize_lid, normalize_phone_number};

    #[test]
    fn normalize_phone_number_keeps_user_part_after_at_split() {
        assert_eq!(normalize_phone_number("12345@lid"), Some("+12345".into()));
    }

    #[test]
    fn normalize_lid_keeps_user_part_after_at_split() {
        assert_eq!(
            normalize_lid("zeroclaw_user@lid"),
            Some("zeroclaw_user".into())
        );
    }
}
