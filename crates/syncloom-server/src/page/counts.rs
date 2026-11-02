use super::*;

/// Present the saved core result; "characters" never substitutes a different counter.
pub(super) fn summary(piece: &Piece) -> String {
    let verdict = match piece.fit.verdict {
        syncloom_core::Verdict::Pass => "fits",
        syncloom_core::Verdict::Review => "needs review",
    };
    let bytes = if piece.channel == Channel::Bluesky {
        format!(
            " Additional UTF-8 bytes: {} of {}.",
            grouped(piece.text.len()),
            grouped(syncloom_core::channel::BLUESKY_MAX_BYTES)
        )
    } else {
        String::new()
    };
    format!(
        r#"<p class="fit-count">{} of {} characters - {verdict}</p><details class="count-details"><summary>How characters are counted</summary><p class="hint">{} App bounds: {} to {} {}.{bytes} This saved-text check does not guarantee the platform will accept the post.</p></details>"#,
        grouped(piece.fit.char_count),
        grouped(piece.fit.max_chars),
        piece.channel.count_explanation(),
        grouped(piece.fit.min_chars),
        grouped(piece.fit.max_chars),
        piece.channel.count_unit()
    )
}

fn grouped(value: usize) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}
