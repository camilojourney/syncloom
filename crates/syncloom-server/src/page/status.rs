use super::*;

pub(super) fn review(status: Status) -> String {
    let (role, label) = match status {
        Status::PendingReview => ("pending", "Pending review"),
        Status::Approved => ("approved", "Approved"),
        Status::Rejected => ("neutral", "Rejected"),
    };
    badge(role, &format!("Review: {label}"))
}

pub(super) fn delivery(value: &str, axis: &str) -> String {
    let (role, label) = presentation(value);
    badge(role, &format!("{axis}: {label}"))
}

pub(super) fn label(value: &str) -> &'static str {
    presentation(value).1
}

fn presentation(value: &str) -> (&'static str, &'static str) {
    if let Some(label) = posting_label(value) {
        return ("failed", label);
    }
    match value {
        "contained" => ("contained", "Not sent - saved on this laptop"),
        "exported" => ("exported", "Saved as local files"),
        "posted" => ("posted", "Posted"),
        "posting" | "unknown" => ("unknown", "Sending not confirmed"),
        "refused" => ("failed", "Platform declined this post"),
        "consent_required" => ("failed", "Account permission needed"),
        "rate_limited" => ("failed", "Platform limit reached"),
        "unreachable" => ("failed", "Couldn't connect"),
        _ => ("neutral", "Result not recorded"),
    }
}

pub(super) fn badge(role: &str, label: &str) -> String {
    format!(
        r#"<span class="status badge-{role}">{}</span>"#,
        escape(label)
    )
}

fn posting_label(value: &str) -> Option<&'static str> {
    match value {
        "posting_permission_missing" => Some("Posting permission missing"),
        "posting_permission_unverified" => Some("Posting permission unverified"),
        _ => None,
    }
}

/// Browser sentences for stable readiness reasons. Credential code owns the decision.
pub(super) fn readiness_sentence(reason: &str, platform: &str) -> Option<String> {
    match reason {
        "posting_permission_missing" => Some(format!(
            "Your {platform} account is connected, but Syncloom isn't allowed to post to it. Reconnect and allow posting."
        )),
        "posting_permission_unverified" => Some(format!(
            "Syncloom couldn't confirm it's allowed to post to your {platform} account. Reconnect to check again."
        )),
        "identity_unconfirmed" => Some(format!(
            "Syncloom couldn't confirm this is your {platform} account. Reconnect to check again."
        )),
        _ => None,
    }
}

/// Intent-local descriptions, never claims about complete piece or account history.
pub(super) fn explanation(status: &str, platform: &str) -> String {
    if let Some(sentence) = readiness_sentence(status, platform) {
        return sentence;
    }
    match status {
        "contained" => "Nothing was sent to the platform. Your post stays saved on this laptop.",
        "posted" => "The platform confirmed this post. It won't be sent again here.",
        "posting" | "unknown" => {
            "We couldn't confirm whether this was sent. Check your platform account before doing anything else. This saved result doesn't tell us whether work is still running; this version won't be sent again here."
        }
        "exported" => {
            "This saved a copy as files on this laptop. Check the publishing result above to see whether it was sent separately."
        }
        "refused" => {
            "The platform declined this post. Check the text and the platform's rules before trying again."
        }
        "consent_required" => {
            "Your account needs sign-in or permission. Finish account setup outside this page, then return."
        }
        "rate_limited" => "The platform is limiting requests. Wait before trying again.",
        "unreachable" => {
            "We couldn't connect to the platform. Check your connection before trying again."
        }
        _ => "Check your platform account to find out whether this was sent.",
    }
    .to_owned()
}
