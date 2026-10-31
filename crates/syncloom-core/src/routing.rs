//! Deterministic source-thought contracts, independent of model output and credentials.
use crate::Channel;
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    En,
    Es,
}

/// A review target, not proof of a connected account or permission to publish.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Destination {
    pub language: Language,
    pub platform: String,
    pub account: String,
    pub handle: String,
    pub approval_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThoughtContract {
    pub destination: Destination,
    pub thesis: String,
    pub visual_hook: String,
    pub products: Vec<String>,
}

// Whole-word evidence, not substring triggers. Each occurrence contributes one vote;
// a Spanish accent contributes at most one additional vote per word, never a veto.
const ENGLISH: &str = "a about after again all also an and any are as at be because been before better between both build building builds but by can clear clients content could day do does each english examples first for from good had has have help helps here how i ideas if in into is it its learn more most my need needs new no not of on one only or our out over own people product products results review same sentence should simple so some teams than that the their them then there these they thing this those through time to token too tools under up use useful using very was we well were what when where which while who will with without work works would write writing you your";
const SPANISH: &str = "a ahora ahorra ahorrar al algo algunos alla antes aplicaciones aprender aprendiendo aqui asi aunque automatizar ayer bien bueno buenos cada cambiar cierto cinco claro claros cliente clientes codigo como compartir con conocer construimos construir contenido contigo contra cosas crear cualquier cuando cuatro de debe decir dejar del demasiado dentro desde despues dia dias diferente dinero donde dos durante ejemplo el ella ellos embargo en entonces entre equipo equipos es esa escribir ese esfuerzo eso esta estamos estan estar este esto estos estoy forma fue funciona ganar gracias grande grandes gusta hablar hacer hacia hasta hay hecho herramientas hoy idea ideas importante incluso informacion ir juntos la las le les llevar lo los luego lugar manera mas me mejor mejora mejores menos mi mientras mis mismo muchas mucho muy nada nadie necesita necesitamos necesitan ni ningun ninguna no nos nosotros nuestra nuestras nuestro nuestros nuevas nuevo nuevos nunca o otra otras otro otros pais para parte pasar pensar pequenos pero poco poder por porque primera primero problema problemas producir productos programar pronto puede pueden puedes que quien quiero repetitivas resultados revision saber salir se sea seguir ser sera si sido siempre siendo sin sino sobre solo somos son su sus tambien tanto tareas te tenemos tener tiempo tiene tienen tienes todo todos trabajar trabajo tres tu un una uno unos usar usted ustedes util utiles vamos ver verdad vez vida volver voy y ya yo";

fn intake_body(text: &str) -> &str {
    let text = text.trim();
    if let Some((label, body)) = text.split_once(':')
        && ["research note", "campaign idea", "thought", "note", "idea"]
            .iter()
            .any(|known| label.trim().eq_ignore_ascii_case(known))
    {
        return body.trim();
    }
    text
}

fn fold_spanish(word: &str) -> String {
    word.chars()
        .filter(|c| !matches!(c, '\u{301}' | '\u{303}' | '\u{308}'))
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            'ñ' => 'n',
            other => other,
        })
        .collect()
}

/// Compare word frequencies and accent evidence. Unknown input and ties use English.
pub fn detect_language(text: &str) -> Language {
    let lower = intake_body(text).to_lowercase();
    let (mut english, mut spanish) = (0, 0);
    for word in lower.unicode_words() {
        let folded = fold_spanish(word);
        english += usize::from(ENGLISH.split_whitespace().any(|known| known == folded));
        spanish += usize::from(SPANISH.split_whitespace().any(|known| known == folded));
        spanish += usize::from(
            word.chars()
                .any(|c| "áéíóúüñ\u{301}\u{303}\u{308}".contains(c)),
        );
    }
    spanish += usize::from(lower.contains(['¿', '¡']));
    if spanish > english {
        Language::Es
    } else {
        Language::En
    }
}

const ABBREVIATIONS: &[&str] = &[
    "e.g.", "i.e.", "vs.", "etc.", "dr.", "mr.", "mrs.", "ms.", "no.", "st.",
];

fn abbreviation_dot(text: &str, index: usize) -> bool {
    let start = text[..index]
        .rfind(char::is_whitespace)
        .map_or(0, |i| i + 1);
    let token = &text[start..];
    let token = token.trim_start_matches(['(', '[', '\"', '\'']);
    let start = text.len() - token.len();
    ABBREVIATIONS.iter().any(|abbr| {
        token
            .get(..abbr.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(abbr))
            && index < start + abbr.len()
    })
}

/// Never split a word, even if the first word alone exceeds the budget (then return empty).
fn word_prefix(text: &str, limit: usize) -> String {
    let mut result = String::new();
    let mut length = 0;
    for word in text.split_whitespace() {
        let next = length + usize::from(!result.is_empty()) + word.chars().count();
        if next > limit {
            break;
        }
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(word);
        length = next;
    }
    result
        .trim_end_matches(['.', ',', ';', ':', '!', '?'])
        .to_owned()
}

/// First sentence without intake labels or terminal punctuation; at most 200 characters.
pub fn extract_thesis(text: &str) -> String {
    let body = intake_body(text);
    let end = body
        .char_indices()
        .find_map(|(index, c)| {
            if matches!(c, '!' | '?') || (c == '.' && !abbreviation_dot(body, index)) {
                Some(index)
            } else {
                None
            }
        })
        .unwrap_or(body.len());
    let sentence = body[..end]
        .trim()
        .trim_end_matches(['.', ',', ';', ':', '!', '?']);
    let mut chars = sentence.chars();
    let capitalized = chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    });
    word_prefix(&capitalized, 200)
}

/// Produce a source-derived review contract. No file, model, credential or network access.
pub fn analyze_thought(text: &str, channel: Channel) -> ThoughtContract {
    let language = detect_language(text);
    let account = match language {
        Language::En => "camiloexperience",
        Language::Es => "camilojourney",
    };
    let thesis = extract_thesis(text);
    let lower = text.to_lowercase();
    ThoughtContract {
        destination: Destination {
            language,
            platform: channel.platform().into(),
            account: account.into(),
            handle: format!("@{account}"),
            approval_required: true,
        },
        visual_hook: word_prefix(&thesis, 72),
        thesis,
        products: ["Pilaster", "Genpeli", "Invoz"]
            .into_iter()
            .filter(|product| {
                lower
                    .unicode_words()
                    .any(|word| word.eq_ignore_ascii_case(product))
            })
            .map(str::to_owned)
            .collect(),
    }
}
