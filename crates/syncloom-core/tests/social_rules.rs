use syncloom_core::{Channel, Verdict, fit};

#[test]
fn x_estimates_use_exact_integer_microdollars_for_plain_link_and_metadata_cases() {
    use syncloom_core::social::estimate;
    for (text, plain, image) in [
        ("A test post.", 15000, 20000),
        ("https://example.invalid", 200000, 205000),
        ("EXAMPLE.COM", 200000, 205000),
        ("example。com", 200000, 205000),
        ("123.example", 200000, 205000),
    ] {
        assert_eq!(estimate(text, false).microusd, plain, "{text}");
        assert_eq!(estimate(text, true).microusd, image, "{text}");
    }
    for length in 1..=250 {
        let quote = estimate(&"a".repeat(length), true);
        assert_eq!(quote.microusd, 20000);
        assert_eq!(quote.metadata_microusd, 5000);
        assert_eq!(quote.usd(), "USD 0.02");
    }
}

#[test]
fn social_channel_limits_are_inclusive_conservative_and_never_truncate() {
    for (name, limit) in [("bluesky", 300), ("x", 280), ("threads", 500)] {
        let c = Channel::parse(name).expect("SocialPlatforms channel");
        assert_eq!(fit(c, &"a".repeat(limit)).verdict, Verdict::Pass);
        assert_eq!(fit(c, &"a".repeat(limit + 1)).verdict, Verdict::Review);
        assert_eq!(fit(c, "").verdict, Verdict::Review);
        assert_eq!(c.platform(), name);
    }
    let b = Channel::parse("bluesky").unwrap();
    assert_eq!(
        fit(b, &"👨‍👩‍👧‍👧".repeat(300)).verdict,
        Verdict::Review,
        "3000 byte cap"
    );
    assert_eq!(fit(b, &"e\u{301}".repeat(300)).char_count, 300);
    assert_eq!(
        fit(Channel::parse("threads").unwrap(), &"😀".repeat(126)).verdict,
        Verdict::Review
    );
}
