use anyhow::{Context as _, Result};
use futures::{SinkExt, StreamExt};
use gpui::{prelude::*, *};
use serde::Deserialize;

use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;

use crate::my_context_ext::MyContextExt;

// === Style system definition ===

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StyleRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_full: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg_color: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify_content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_items: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_self: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_direction: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_grow: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_shrink: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_name")]
    pub shadow: Option<String>,

    #[serde(skip_serializing_if = "Option::is_name")]
    pub border_width: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_style: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rounded: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_top: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_right: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_bottom: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_left: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_top: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_right: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_bottom: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_left: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f32>,
}

pub struct MyStyleData {
    pub style_map: StyleMap,
}

pub type StyleMap = HashMap<String, StyleRule>;

pub fn load_styles(path: &PathBuf) -> anyhow::Result<MyStyleData> {
    let content = std::fs::read(path.as_path())?;
    let json = pjson::PJsonReader::from_pjson(&content);
    let json = String::from_utf8_lossy(&json).to_string();
    let styles: StyleMap = serde_json::from_str(&json)?;

    Ok(MyStyleData { style_map: styles })
}

// Utility function: parse color
fn parse_color(hex: &str) -> Rgba {
    let r = Rgba::try_from(hex);
    match r {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("parse_color {} err:{:?}", hex, e);
            Rgba::default()
        }
    }
}

fn parse_font_size(size: &str) -> Pixels {
    match size {
        "sm" => px(12.0),
        "base" => px(16.0),
        "lg" => px(20.0),
        "xl" => px(24.0),
        "2xl" => px(32.0),
        _ => {
            let x = size.parse::<f32>();
            match x {
                Ok(x) => px(x),
                Err(e) => {
                    tracing::error!("parse_font_size {} err: {:?}", size, e);
                    px(16.0)
                }
            }
        }
    }
}

pub trait StylableElement: Sized + Styled {
    fn class(self, classes: impl Into<String>, style_data: &MyStyleData) -> Self {
        let rules = &style_data.style_map;
        let classes: String = classes.into();
        let class_vec: Vec<_> = classes.split(' ').collect();
        let mut self2 = self;
        for class in class_vec {
            if let Some(rule) = rules.get(class) {
                self2 = self2.apply_style_rule(rule);
            }
        }

        self2
    }

    fn apply_style_rule(self, rule: &StyleRule) -> Self;
}

impl<T> StylableElement for T
where
    T: Styled,
{
    fn apply_style_rule(mut self, rule: &StyleRule) -> Self {
        if let Some(size_full) = rule.size_full {
            if size_full {
                self = self.size_full();
            }
        }
        if let Some(bg) = &rule.bg_color {
            self = self.bg(parse_color(bg));
        }
        if let Some(c) = &rule.text_color {
            self = self.text_color(parse_color(c));
        }
        if let Some(fs) = &rule.font_size {
            self = self.text_size(parse_font_size(fs));
        }
        if let Some(font_weight) = &rule.font_weight {
            self = self.font_weight(gpui::FontWeight::from(
                font_weight.parse::<f32>().unwrap_or(0.0),
            ));
        }
        if let Some(display) = &rule.display {
            self = match display.as_str() {
                "block" => self.block(),
                "flex" => self.flex(),
                "grid" => self.grid(),
                "none" => {
                    self.style().display = Some(Display::None);
                    self
                }
                _ => self,
            };
        }
        if let Some(jc) = &rule.justify_content {
            self = match jc.as_str() {
                "center" => self.justify_center(),
                "flex-start" => {
                    self.style().justify_content = Some(JustifyContent::FlexStart);
                    self
                }
                "start" => self.justify_start(),
                "flex-end" => {
                    self.style().justify_content = Some(JustifyContent::FlexEnd);
                    self
                }
                "end" => self.justify_end(),
                "space-between" => self.justify_between(),
                "space-around" => self.justify_around(),
                "space-evenly" => {
                    self.style().justify_content = Some(JustifyContent::SpaceEvenly);
                    self
                }
                "stretch" => {
                    self.style().justify_content = Some(JustifyContent::Stretch);
                    self
                }
                _ => self,
            };
        }
        if let Some(ai) = &rule.align_items {
            self = match ai.as_str() {
                "center" => self.items_center(),
                "flex-start" => self.items_start(),
                "start" => self.items_start(),
                "flex-end" => self.items_end(),
                "end" => self.items_end(),
                "baseline" => self.items_baseline(),
                "stretch" => {
                    self.style().align_items = Some(AlignItems::Stretch);
                    self
                }
                _ => self,
            };
        }
        if let Some(ai) = &rule.align_self {
            self = match ai.as_str() {
                "center" => {
                    self.style().align_self = Some(AlignSelf::Center);
                    self
                }
                "flex-start" => {
                    self.style().align_self = Some(AlignSelf::FlexStart);
                    self
                }
                "start" => {
                    self.style().align_self = Some(AlignSelf::Start);
                    self
                }
                "flex-end" => {
                    self.style().align_self = Some(AlignSelf::FlexEnd);
                    self
                }
                "end" => {
                    self.style().align_self = Some(AlignSelf::End);
                    self
                }
                "baseline" => {
                    self.style().align_self = Some(AlignSelf::Baseline);
                    self
                }
                "stretch" => {
                    self.style().align_self = Some(AlignSelf::Stretch);
                    self
                }
                _ => self,
            };
        }
        if let Some(fd) = &rule.flex_direction {
            self = match fd.as_str() {
                "row" => self.flex_row(),
                "column" => self.flex_col(),
                _ => self,
            };
        }
        if let Some(flex_grow) = rule.flex_grow {
            self.style().flex_grow = Some(flex_grow);
        }
        if let Some(flex_shrink) = rule.flex_shrink {
            self.style().flex_shrink = Some(flex_shrink);
        }

        if let Some(w) = rule.width {
            self = self.w(px(w));
        }
        if let Some(h) = rule.height {
            self = self.h(px(h));
        }
        if let Some(shadow) = &rule.shadow {
            match shadow.as_str() {
                "2xs" => self = self.shadow_2xs(),
                "xs" => self = self.shadow_xs(),
                "sm" => self = self.shadow_sm(),
                "md" => self = self.shadow_md(),
                "lg" => self = self.rounded_lg(),
                "xl" => self = self.shadow_xl(),
                "2xl" => self = self.shadow_2xl(),
                "none" => self = self.shadow_none(),
                _ => {}
            }
        }
        if let Some(bw) = rule.border_width {
            self = self.border(px(bw));
        }
        if let Some(bc) = &rule.border_color {
            self = self.border_color(parse_color(bc));
        }
        if let Some(bs) = &rule.border_style {
            if bs == "dashed" {
                self = self.border_dashed();
            }
        }
        if let Some(g) = rule.gap {
            self = self.gap(px(g));
        }
        if let Some(rounded) = &rule.rounded {
            match rounded.as_str() {
                "md" => self = self.rounded_md(),
                "lg" => self = self.rounded_lg(),
                "full" => self = self.rounded_full(),
                _ => {}
            }
        }

        // Set margin
        let mut margin = Edges {
            top: rule.margin.map(px).unwrap_or(px(0.0)),
            right: rule.margin.map(px).unwrap_or(px(0.0)),
            bottom: rule.margin.map(px).unwrap_or(px(0.0)),
            left: rule.margin.map(px).unwrap_or(px(0.0)),
        };
        margin = Edges {
            top: rule.margin_top.map(px).unwrap_or(margin.top),
            right: rule.margin_right.map(px).unwrap_or(margin.right),
            bottom: rule.margin_bottom.map(px).unwrap_or(margin.bottom),
            left: rule.margin_left.map(px).unwrap_or(margin.left),
        };
        self = self
            .m_0()
            .mt(margin.top)
            .mr(margin.right)
            .mb(margin.bottom)
            .ml(margin.left);

        // Set padding
        let mut padding = Edges {
            top: rule.padding.map(px).unwrap_or(px(0.0)),
            right: rule.padding.map(px).unwrap_or(px(0.0)),
            bottom: rule.padding.map(px).unwrap_or(px(0.0)),
            left: rule.padding.map(px).unwrap_or(px(0.0)),
        };
        padding = Edges {
            top: rule.padding_top.map(px).unwrap_or(padding.top),
            right: rule.padding_right.map(px).unwrap_or(padding.right),
            bottom: rule.padding_bottom.map(px).unwrap_or(padding.bottom),
            left: rule.padding_left.map(px).unwrap_or(padding.left),
        };
        self = self
            .p_0()
            .pt(padding.top)
            .pr(padding.right)
            .pb(padding.bottom)
            .pl(padding.left);

        self
    }
}

pub fn init_style_data<T: 'static>(cx: &mut Context<T>, style_path: String) -> MyStyleData
where
    T: SetMyStyleData,
{
    // If the style_path file doesn't exist, replace it with ../../{style_path}
    let style_path = if std::path::Path::new(&style_path).exists() {
        style_path
    } else {
        format!("../../{}", style_path)
    };

    let style_path2 = style_path.clone();

    let style_data: MyStyleData = match load_styles(&PathBuf::from(style_path)) {
        Ok(data) => {
            tracing::info!("✅ Styles reloaded successfully.");
            data
        }
        Err(e) => {
            tracing::error!("{:?}", e);
            thread::sleep(std::time::Duration::from_secs(1));
            panic!("Failed to load styles.")
        }
    };

    watch_style_data(cx, style_path2);

    style_data
}

pub trait SetMyStyleData {
    fn set_style_data(&mut self, data: MyStyleData);
    fn get_style_data(&self) -> &MyStyleData;
}

fn watch_style_data<T: 'static>(cx: &mut Context<T>, style_path: String)
where
    T: SetMyStyleData,
{
    let (th_sender, mut th_receiver) = futures::channel::mpsc::channel::<MyStyleData>(100); // std::sync::mpsc::channel::<MyStyleData>();

    let be = cx.background_executor().clone();

    std::thread::spawn(|| {
        if let Err(e) = run_watcher(PathBuf::from(style_path), be, th_sender) {
            tracing::error!("File watcher failed: {:?}", e);
        }
    });

    cx.my_spawn(async move |entity, cx| {
        loop {
            match th_receiver.next().await {
                Some(r) => {
                    tracing::info!("Refresh window");

                    entity
                        .upgrade()
                        .context("entity upgrade fail.")?
                        .update(cx, |this, cx| {
                            this.set_style_data(r);
                            cx.notify(); // Must notify UI to update
                        })?;
                }
                None => {}
            }
        }

        #[allow(unreachable_code)]
        Ok(())
    })
    .detach();
}

// File watcher (hot reload)
fn run_watcher(
    path: PathBuf,
    be: BackgroundExecutor,
    mut sender: futures::channel::mpsc::Sender<MyStyleData>,
) -> Result<()> {
    use notify::{recommended_watcher, RecursiveMode, Watcher};

    let path2 = path.clone();
    let mut watcher =
        recommended_watcher(move |res: Result<notify::Event, notify::Error>| match res {
            Ok(event) => {
                if event
                    .paths
                    .iter()
                    .any(|path| path.canonicalize().ok() == path.canonicalize().ok())
                {
                    match load_styles(&path) {
                        Ok(style_data) => {
                            tracing::info!("✅ Styles reloaded successfully.");
                            be.block(async {
                                let _ = sender.send(style_data).await;
                            });
                        }
                        Err(e) => {
                            tracing::error!("{:?}", e);
                        }
                    }
                }
            }
            Err(e) => tracing::error!("Watch error: {:?}", e),
        })?;

    watcher.watch(&path2, RecursiveMode::NonRecursive)?;
    loop {
        std::thread::park(); // keep alive, prevent watcher from being dropped.
    }
}
