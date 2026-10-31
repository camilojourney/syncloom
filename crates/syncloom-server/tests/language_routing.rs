mod support;
use serde_json::json;
use support::scripted::{Scripted, text};
use support::*;

#[tokio::test]
async fn intake_and_saved_pieces_expose_the_source_language_contract_not_provider_language() {
    for (thought, language, account, thesis) in [
        (
            "Research note: the los token is in an English thought. Pilaster helps.",
            "en",
            "camiloexperience",
            "The los token is in an English thought",
        ),
        (
            "Campaign idea: nuestros clientes necesitan herramientas simples. Genpeli ayuda.",
            "es",
            "camilojourney",
            "Nuestros clientes necesitan herramientas simples",
        ),
    ] {
        let server = start(Scripted::new(text(
            "The provider writes English regardless.",
        )))
        .await;
        let created = post(
            &server,
            "/v1/thoughts",
            json!({"text":thought,"channels":["linkedin_text"]}),
        )
        .await;
        assert_eq!(created.status, 201, "{}", created.raw);
        let piece = &created.body["pieces"][0];
        assert_eq!(piece["routing"]["destination"]["language"], language);
        assert_eq!(piece["routing"]["destination"]["account"], account);
        assert_eq!(piece["routing"]["thesis"], thesis);
        assert_eq!(piece["source"]["raw_input"], thought);
        let id = piece["piece_id"].as_str().unwrap();
        let saved = get(&server, &format!("/v1/pieces/{id}")).await;
        assert_eq!(saved.status, 200);
        assert_eq!(saved.body["routing"], piece["routing"]);
    }
}
