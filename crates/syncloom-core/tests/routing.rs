use syncloom_core::Channel;
use syncloom_core::routing::{Language, analyze_thought, detect_language, extract_thesis};

#[test]
fn english_lookalikes_are_not_spanish_sentences() {
    for token in ["de", "los", "con", "las", "una", "por"] {
        let input =
            format!("The {token} token is in this English sentence about building better tools.");
        assert_eq!(detect_language(&input), Language::En, "{input}");
    }
    for input in [
        "The workflow is reliable.",
        "We build systems for teams.",
        "The café is open and the résumé is ready.",
        "Con artists work in Los Angeles.",
    ] {
        assert_eq!(detect_language(input), Language::En, "{input}");
    }
}

#[test]
fn spanish_works_without_accents_or_old_marker_words() {
    for input in [
        "Necesitamos escribir mejor contenido cada dia.",
        "Nuestros clientes necesitan herramientas simples y resultados claros.",
        "Quiero compartir ideas utiles sobre equipos pequenos.",
        "Estoy aprendiendo mientras construimos productos nuevos.",
        "Me gusta programar aplicaciones nuevas.",
        "Automatizar tareas repetitivas ahorra tiempo.",
        "El sistema de trabajo mejora con una revision.",
        "¿Cómo diseñamos una solución más útil?",
    ] {
        assert_eq!(detect_language(input), Language::Es, "{input}");
    }
}

#[test]
fn empty_unknown_and_tied_evidence_defaults_to_english() {
    for input in [
        "",
        "123 !!!",
        "Pilaster Genpeli Invoz",
        "the el",
        "workshop confident lasting porcelain",
    ] {
        assert_eq!(detect_language(input), Language::En, "{input}");
    }
    assert_eq!(detect_language("EL EL the"), Language::Es);
    assert_eq!(detect_language("mañana"), Language::Es);
    assert_eq!(detect_language("man\u{303}ana"), Language::Es);
}

#[test]
fn thesis_strips_labels_and_terminal_punctuation_but_keeps_first_sentence() {
    for (input, expected) in [
        (
            "Research note: better tools help teams. Another thought.",
            "Better tools help teams",
        ),
        (
            "Campaign idea: launch Pilaster! Then Genpeli.",
            "Launch Pilaster",
        ),
        (
            "  RESEARCH NOTE:  una idea nueva? Otra idea.",
            "Una idea nueva",
        ),
        ("A plain thought... Another thought.", "A plain thought"),
        (
            "What matters: clear ownership.",
            "What matters: clear ownership",
        ),
        ("", ""),
        ("Research note:", ""),
        ("!!!", ""),
    ] {
        assert_eq!(extract_thesis(input), expected, "{input}");
    }
}

#[test]
fn abbreviations_never_end_the_first_sentence_early() {
    for abbreviation in [
        "e.g.", "i.e.", "vs.", "etc.", "Dr.", "Mr.", "Mrs.", "Ms.", "No.", "St.",
    ] {
        let input = format!("Research note: consider {abbreviation} useful examples. Ignore this.");
        assert_eq!(
            extract_thesis(&input),
            format!("Consider {abbreviation} useful examples")
        );
    }
    assert_eq!(
        extract_thesis("Dr. Smith builds tools. Next."),
        "Dr. Smith builds tools"
    );
}

#[test]
fn thesis_and_hook_cut_at_word_boundaries_counting_unicode_characters() {
    for limit in [72, 200] {
        for length in [limit - 1, limit, limit + 1] {
            let input = format!("{} z", "é".repeat(length));
            let contract = analyze_thought(&input, Channel::LinkedinText);
            let output = if limit == 72 {
                contract.visual_hook
            } else {
                contract.thesis
            };
            let expected = if length <= limit {
                format!("É{}", "é".repeat(length - 1))
            } else {
                String::new()
            };
            // Only the first character is capitalized; an oversized first word is omitted.
            assert_eq!(output, expected, "limit={limit}, length={length}");
        }
    }
    let input = format!("Research note: {} finalword", "word ".repeat(39));
    let contract = analyze_thought(&input, Channel::LinkedinText);
    assert_eq!(
        contract.thesis,
        format!("Word {}", "word ".repeat(38)).trim()
    );
    assert_eq!(
        contract.visual_hook,
        format!("Word {}", "word ".repeat(13)).trim()
    );
}

#[test]
fn portfolio_mentions_are_whole_words_case_insensitive_unique_in_catalog_order() {
    let result = analyze_thought(
        "invoz and PILASTER help Genpeli. Pilaster again; not Invozip or xGenpeli.",
        Channel::LinkedinText,
    );
    assert_eq!(result.products, ["Pilaster", "Genpeli", "Invoz"]);
    assert!(
        analyze_thought("Pilasters Genpelis Invozip", Channel::X)
            .products
            .is_empty()
    );
}

#[test]
fn founder_destinations_follow_language_and_channel_without_claiming_delivery() {
    for (input, language, account) in [
        (
            "The tools help our teams.",
            Language::En,
            "camiloexperience",
        ),
        (
            "Nuestros clientes necesitan herramientas simples.",
            Language::Es,
            "camilojourney",
        ),
    ] {
        for channel in [
            Channel::LinkedinText,
            Channel::X,
            Channel::Threads,
            Channel::Instagram,
        ] {
            let result = analyze_thought(input, channel);
            assert_eq!(result.destination.language, language);
            assert_eq!(result.destination.account, account);
            assert_eq!(result.destination.handle, format!("@{account}"));
            assert_eq!(result.destination.platform, channel.platform());
            assert!(result.destination.approval_required);
        }
    }
}
