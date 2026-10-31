//! Structured visual content and a bounded, offline Chromium CDP renderer.
use crate::StudioError;
use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::{
    net::TcpStream,
    process::{Child, Command},
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};

pub const TEMPLATE_VERSION: &str = "1";
pub const BROWSER_VERSION: &str = "145.0.7632.6";
pub const FONT_SHA256: &str = "525979822591a3447cfc49d943d6f7683508e25543407871c0ed8fed05fd2bd9";
pub const WIDTH: u32 = 1080;
pub const HEIGHT: u32 = 1350;

#[derive(Clone, Debug)]
pub struct RendererConfig {
    pub binary: PathBuf,
    pub version: String,
    pub font: PathBuf,
    pub font_sha256: String,
    pub deadline: Duration,
}

/// Paths are operator configuration, never render-request input. No browser is installed.
pub fn renderer_config(get: impl Fn(&str) -> Option<String>) -> RendererConfig {
    let home = PathBuf::from(get("HOME").unwrap_or_default());
    RendererConfig {
        binary: get("SYNCLOOM_RENDERER_PATH").map(PathBuf::from).unwrap_or_else(|| home.join(
            "Library/Caches/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-mac-arm64/chrome-headless-shell")),
        version: get("SYNCLOOM_RENDERER_VERSION").unwrap_or_else(|| BROWSER_VERSION.into()),
        font: get("SYNCLOOM_RENDERER_FONT").map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/System/Library/Fonts/Supplemental/Arial.ttf")),
        font_sha256: get("SYNCLOOM_RENDERER_FONT_SHA256").unwrap_or_else(|| FONT_SHA256.into()),
        deadline: Duration::from_secs(30),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderRequest {
    pub expected_revision: String,
    pub template: String,
    pub slides: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualContent {
    pub template_version: String,
    pub template: String,
    pub slides: Vec<String>,
}

impl RenderRequest {
    pub fn content(&self) -> Result<VisualContent, StudioError> {
        if !matches!(self.template.as_str(), "midnight" | "paper")
            || !(1..=10).contains(&self.slides.len())
            || self
                .slides
                .iter()
                .any(|s| s.trim().is_empty() || s.chars().count() > 800)
        {
            return Err(StudioError::InvalidRequest);
        }
        Ok(VisualContent {
            template_version: TEMPLATE_VERSION.into(),
            template: self.template.clone(),
            slides: self.slides.iter().map(|s| s.trim().to_owned()).collect(),
        })
    }
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetHashes {
    pub image_sha256: String,
    pub carousel_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assets {
    pub revision: String,
    pub template_version: String,
    pub image_sha256: String,
    pub carousel_sha256: String,
    pub pages: usize,
    pub renderer_version: String,
    pub font_sha256: String,
}

#[derive(Clone, Copy, Debug)]
pub enum AssetKind {
    Image,
    Carousel,
}
impl AssetKind {
    pub fn mime(self) -> &'static str {
        match self {
            Self::Image => "image/png",
            Self::Carousel => "application/pdf",
        }
    }
    pub fn hash(self, assets: &Assets) -> &str {
        match self {
            Self::Image => &assets.image_sha256,
            Self::Carousel => &assets.carousel_sha256,
        }
    }
}

pub struct Rendered {
    pub image: Vec<u8>,
    pub carousel: Vec<u8>,
}

impl RendererConfig {
    pub async fn render(&self, content: &VisualContent) -> Result<Rendered, StudioError> {
        // The child lives outside the timed future so every outcome kills and reaps it.
        let profile = tempfile::tempdir().map_err(|_| StudioError::RendererFailed)?;
        let mut child = None;
        let result = tokio::time::timeout(
            self.deadline,
            self.render_inner(content, profile.path(), &mut child),
        )
        .await
        .unwrap_or(Err(StudioError::RendererTimeout));
        if let Some(mut child) = child {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        result
    }

    fn load_font(&self, content: &VisualContent) -> Result<Vec<u8>, StudioError> {
        let font = std::fs::read(&self.font).map_err(|_| StudioError::RendererFontMismatch)?;
        if digest(&font) != self.font_sha256 {
            return Err(StudioError::RendererFontMismatch);
        }
        let face =
            ttf_parser::Face::parse(&font, 0).map_err(|_| StudioError::RendererFontMismatch)?;
        if content
            .slides
            .iter()
            .flat_map(|s| s.chars())
            .any(|c| !c.is_whitespace() && face.glyph_index(c).is_none())
        {
            return Err(StudioError::InvalidRequest);
        }
        Ok(font)
    }

    fn launch(&self, profile: &std::path::Path) -> Result<Child, StudioError> {
        Command::new(&self.binary)
            .args([
                "--headless",
                "--disable-gpu",
                "--use-angle=swiftshader",
                "--force-device-scale-factor=1",
                "--force-color-profile=srgb",
                "--font-render-hinting=none",
                "--disable-background-networking",
                "--disable-component-update",
                "--disable-sync",
                "--no-first-run",
                "--no-default-browser-check",
                "--metrics-recording-only",
                "--host-resolver-rules=MAP * ~NOTFOUND",
                "--remote-debugging-address=127.0.0.1",
                "--remote-debugging-port=0",
            ])
            .arg(format!("--user-data-dir={}", profile.display()))
            .arg("about:blank")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    StudioError::RendererMissing
                } else {
                    StudioError::RendererFailed
                }
            })
    }

    async fn render_inner(
        &self,
        content: &VisualContent,
        profile: &std::path::Path,
        child: &mut Option<Child>,
    ) -> Result<Rendered, StudioError> {
        let font = self.load_font(content)?;
        *child = Some(self.launch(profile)?);
        let mut cdp = Cdp::connect(
            profile,
            child.as_mut().expect("just launched"),
            &self.version,
        )
        .await?;
        let frame_id = cdp.attach_page().await?;
        cdp.set_document(frame_id, content, &font).await?;
        cdp.capture().await
    }
}

struct Cdp {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    id: u64,
    session: Option<String>,
}
async fn debug_endpoint(
    profile: &std::path::Path,
    child: &mut Child,
) -> Result<String, StudioError> {
    loop {
        if child
            .try_wait()
            .map_err(|_| StudioError::RendererFailed)?
            .is_some()
        {
            return Err(StudioError::RendererFailed);
        }
        if let Ok(active) = std::fs::read_to_string(profile.join("DevToolsActivePort")) {
            let mut lines = active.lines();
            if let (Some(port), Some(path)) = (lines.next(), lines.next())
                && port.parse::<u16>().is_ok()
                && path.starts_with("/devtools/browser/")
            {
                return Ok(format!("ws://127.0.0.1:{port}{path}"));
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

impl Cdp {
    async fn connect(
        profile: &std::path::Path,
        child: &mut Child,
        expected_version: &str,
    ) -> Result<Self, StudioError> {
        let endpoint = debug_endpoint(profile, child).await?;
        let (socket, _) = tokio_tungstenite::connect_async(&endpoint)
            .await
            .map_err(|_| StudioError::RendererFailed)?;
        let mut cdp = Self {
            socket,
            id: 0,
            session: None,
        };
        let version = cdp.call("Browser.getVersion", json!({})).await?;
        if version["product"]
            .as_str()
            .and_then(|p| p.rsplit('/').next())
            != Some(expected_version)
        {
            return Err(StudioError::RendererVersionMismatch);
        }
        Ok(cdp)
    }

    async fn attach_page(&mut self) -> Result<Value, StudioError> {
        let target = self
            .call("Target.createTarget", json!({"url":"about:blank"}))
            .await?;
        let attached = self
            .call(
                "Target.attachToTarget",
                json!({"targetId":target["targetId"],"flatten":true}),
            )
            .await?;
        self.session = Some(
            attached["sessionId"]
                .as_str()
                .ok_or(StudioError::RendererFailed)?
                .to_owned(),
        );
        self.call("Page.enable", json!({})).await?;
        self.call("Network.enable", json!({})).await?;
        self.call(
            "Network.setBlockedURLs",
            json!({"urls":["http://*","https://*","file://*","ftp://*","ws://*","wss://*"]}),
        )
        .await?;
        self.call(
            "Emulation.setDeviceMetricsOverride",
            json!({
                "width":WIDTH,"height":HEIGHT,"deviceScaleFactor":1,"mobile":false
            }),
        )
        .await?;
        let tree = self.call("Page.getFrameTree", json!({})).await?;
        Ok(tree["frameTree"]["frame"]["id"].clone())
    }

    async fn set_document(
        &mut self,
        frame_id: Value,
        content: &VisualContent,
        font: &[u8],
    ) -> Result<(), StudioError> {
        let html = document(content, font)?;
        self.call(
            "Page.setDocumentContent",
            json!({"frameId":frame_id, "html":html}),
        )
        .await?;
        let ready = self.call("Runtime.evaluate", json!({
            "expression":"document.fonts.ready.then(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve(document.fonts.check('64px Studio'))))))",
            "awaitPromise":true,"returnByValue":true
        })).await?;
        if ready["result"]["value"] != true {
            return Err(StudioError::RendererFontMismatch);
        }
        let fits = self.call("Runtime.evaluate", json!({
            "expression":"Array.from(document.querySelectorAll('.copy')).every(e => e.scrollHeight <= e.clientHeight && e.scrollWidth <= e.clientWidth)",
            "returnByValue":true
        })).await?;
        if fits["result"]["value"] != true {
            return Err(StudioError::InvalidRequest);
        }

        Ok(())
    }

    async fn capture(&mut self) -> Result<Rendered, StudioError> {
        let image = self
            .call(
                "Page.captureScreenshot",
                json!({"format":"png","captureBeyondViewport":false,
            "clip":{"x":0,"y":0,"width":WIDTH,"height":HEIGHT,"scale":1}}),
            )
            .await?;
        let pdf = self
            .call(
                "Page.printToPDF",
                json!({
                    "printBackground":true,"preferCSSPageSize":true,"displayHeaderFooter":false,
                    "marginTop":0,"marginBottom":0,"marginLeft":0,"marginRight":0,
                    "generateTaggedPDF":true
                }),
            )
            .await?;
        Ok(Rendered {
            image: decoded(&image)?,
            carousel: normalize_pdf(&decoded(&pdf)?)?,
        })
    }

    async fn call(&mut self, method: &str, params: Value) -> Result<Value, StudioError> {
        self.id += 1;
        let mut message = json!({"id":self.id,"method":method,"params":params});
        if let Some(session) = &self.session {
            message["sessionId"] = json!(session);
        }
        self.socket
            .send(Message::Text(message.to_string().into()))
            .await
            .map_err(|_| StudioError::RendererFailed)?;
        while let Some(frame) = self.socket.next().await {
            let frame = frame.map_err(|_| StudioError::RendererFailed)?;
            if let Message::Text(text) = frame {
                let response: Value =
                    serde_json::from_str(&text).map_err(|_| StudioError::RendererFailed)?;
                if response["id"] == self.id {
                    if response.get("error").is_some() {
                        return Err(StudioError::RendererFailed);
                    }
                    return Ok(response["result"].clone());
                }
            }
        }
        Err(StudioError::RendererFailed)
    }
}

fn decoded(result: &Value) -> Result<Vec<u8>, StudioError> {
    STANDARD
        .decode(result["data"].as_str().ok_or(StudioError::RendererFailed)?)
        .map_err(|_| StudioError::RendererFailed)
}

fn normalize_pdf(bytes: &[u8]) -> Result<Vec<u8>, StudioError> {
    let mut doc = lopdf::Document::load_mem(bytes).map_err(|_| StudioError::RendererFailed)?;
    for object in doc.objects.values_mut() {
        if let Ok(dict) = object.as_dict_mut() {
            dict.remove(b"CreationDate");
            dict.remove(b"ModDate");
        }
    }
    // Chromium rounds CSS paper dimensions upward internally (1350px became 1013.04pt).
    // The extra fraction is outside the HTML content; trim it without scaling vector text.
    for id in doc.get_pages().values() {
        let page = doc
            .get_object_mut(*id)
            .and_then(lopdf::Object::as_dict_mut)
            .map_err(|_| StudioError::RendererFailed)?;
        page.set(
            "MediaBox",
            vec![
                lopdf::Object::Integer(0),
                lopdf::Object::Integer(0),
                lopdf::Object::Real(WIDTH as f32 * 0.75),
                lopdf::Object::Real(HEIGHT as f32 * 0.75),
            ],
        );
    }
    doc.trailer.remove(b"ID");
    let mut output = Vec::new();
    doc.save_to(&mut output)
        .map_err(|_| StudioError::RendererFailed)?;
    Ok(output)
}

fn gradient(paper: bool) -> Result<Vec<u8>, StudioError> {
    let mut pixels = Vec::with_capacity((WIDTH * HEIGHT * 3) as usize);
    let (start, end): ([u32; 3], [u32; 3]) = if paper {
        ([248, 245, 235], [213, 232, 226])
    } else {
        ([14, 22, 45], [30, 63, 82])
    };
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let n = x + y;
            let total = WIDTH + HEIGHT - 2;
            for c in 0..3 {
                pixels.push(((start[c] * (total - n) + end[c] * n) / total) as u8);
            }
        }
    }
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, WIDTH, HEIGHT);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .and_then(|mut w| w.write_image_data(&pixels))
            .map_err(|_| StudioError::RendererFailed)?;
    }
    Ok(out)
}

fn document(content: &VisualContent, font: &[u8]) -> Result<String, StudioError> {
    let paper = content.template == "paper";
    let color = if paper { "#162a38" } else { "#f4f5f0" };
    let accent = if paper { "#126958" } else { "#a5f3cf" };
    let background = STANDARD.encode(gradient(paper)?);
    let font = STANDARD.encode(font);
    let mut html = format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; font-src data:; style-src 'unsafe-inline'">
<style>
@font-face {{ font-family:Studio;src:url(data:font/ttf;base64,{font});font-weight:100 900 }}
@page {{ size:{WIDTH}px {HEIGHT}px; margin:0 }}
* {{ box-sizing:border-box }} html,body {{ margin:0;padding:0;scrollbar-width:none }}
::-webkit-scrollbar {{ display:none }}
body {{ font-family:Studio;color:{color};-webkit-print-color-adjust:exact;print-color-adjust:exact }}
section {{ width:{WIDTH}px;height:{HEIGHT}px;padding:96px;position:relative;break-after:page;
background-image:url(data:image/png;base64,{background});background-size:{WIDTH}px {HEIGHT}px;overflow:hidden }}
section:last-child {{ break-after:auto }}
.label {{ font-size:23px;letter-spacing:5px;text-transform:uppercase;color:{accent};margin-bottom:90px }}
.copy {{ height:850px;font-size:64px;line-height:1.2;white-space:pre-wrap;overflow-wrap:anywhere }}
footer {{ position:absolute;left:96px;right:96px;bottom:80px;border-top:2px solid {accent};padding-top:24px;
font-size:23px;display:flex;justify-content:space-between }}
</style></head><body>"#
    );
    for (i, slide) in content.slides.iter().enumerate() {
        html.push_str(&format!(r#"<section><div class="label">FIELD NOTES</div><div class="copy">{}</div><footer><span>SYNCLOOM</span><span>{:02} / {:02}</span></footer></section>"#,crate::html::escape(slide), i+1, content.slides.len()));
    }
    html.push_str("</body></html>");
    Ok(html)
}
