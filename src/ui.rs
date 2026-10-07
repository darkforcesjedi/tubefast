use crate::APP_NAME;
use crate::app::{Action, App, Covers, Load, Repeat, Route};
use crate::art::Art;
use crate::ytm::{Item, Layout};
use eframe::egui::ecolor::Hsva;
use eframe::egui::epaint::{RectShape, Shadow};
use eframe::egui::style::ScrollStyle;
use eframe::egui::text::{LayoutJob, TextFormat, TextWrapping};
use eframe::egui::{
    self, Align, Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, Frame, Galley, Id, Key, Margin, Pos2, Rect, Response,
    RichText, Sense, Shape, Stroke, StrokeKind, TextStyle, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType, pos2, vec2,
};
use egui_phosphor::regular as icon;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

const BASE: Color32 = rgb(0x0E0F12);
const PANEL: Color32 = rgb(0x0A0B0D);
const RAISED: Color32 = rgb(0x1A1B20);
const TEXT: Color32 = rgb(0xECEDEF);
const MUTED: Color32 = Color32::from_rgba_premultiplied(166, 166, 166, 166);
const FAINT: Color32 = Color32::from_rgba_premultiplied(122, 122, 122, 122);
const ACCENT: Color32 = rgb(0xF4623F);
const ON_ACCENT: Color32 = rgb(0x1A0A05);

const SIDEBAR_WIDTH: f32 = 244.0;
const QUEUE_WIDTH: f32 = 336.0;
const QUEUE_MIN_WINDOW: f32 = 1140.0;
const BAR_HEIGHT: f32 = 84.0;
const ACCOUNT_HEIGHT: f32 = 68.0;
const TOPBAR_HEIGHT: f32 = 68.0;
const AMBIENT_HEIGHT: f32 = 380.0;
const HERO_ART: f32 = 216.0;
const BAR_ART: f32 = 52.0;
const ROW_HEIGHT: f32 = 56.0;
const ROW_RADIUS: f32 = 8.0;
const ART_RADIUS: f32 = 8.0;
const CARD_MIN: f32 = 164.0;
const CARD_GAP: f32 = 20.0;
const CARD_TEXT: f32 = 62.0;
const COMPACT_MIN: f32 = 320.0;
const COMPACT_ROWS: usize = 3;
const PAGE_MARGIN: i8 = 32;
const HOVER_TIME: f32 = 0.1;
const MENU_WIDTH: f32 = 248.0;

const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

fn veil(opacity: f32) -> Color32 {
    Color32::from_white_alpha((opacity.clamp(0.0, 1.0) * 255.0) as u8)
}

fn shade(opacity: f32) -> Color32 {
    Color32::from_black_alpha((opacity.clamp(0.0, 1.0) * 255.0) as u8)
}

static FAMILIES: LazyLock<[FontFamily; 5]> =
    LazyLock::new(|| ["medium", "bold", "display", "icons", "icons-fill"].map(|name| FontFamily::Name(name.into())));

fn sans(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

fn medium(size: f32) -> FontId {
    FontId::new(size, FAMILIES[0].clone())
}

fn bold(size: f32) -> FontId {
    FontId::new(size, FAMILIES[1].clone())
}

fn display(size: f32) -> FontId {
    FontId::new(size, FAMILIES[2].clone())
}

fn glyph(size: f32) -> FontId {
    FontId::new(size, FAMILIES[3].clone())
}

fn solid(size: f32) -> FontId {
    FontId::new(size, FAMILIES[4].clone())
}

fn system_font(file: &str) -> Option<&'static [u8]> {
    let fonts = std::path::Path::new(&std::env::var_os("WINDIR")?).join("Fonts");
    let file = std::fs::File::open(fonts.join(file)).ok()?;
    let map = unsafe { memmap2::Mmap::map(&file) }.ok()?;
    Some(&Box::leak(Box::new(map))[..])
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let embedded: [(&str, &'static [u8]); 8] = [
        ("onest-400", include_bytes!("../assets/fonts/onest-latin-400.ttf")),
        ("onest-400-ext", include_bytes!("../assets/fonts/onest-latin-ext-400.ttf")),
        ("onest-500", include_bytes!("../assets/fonts/onest-latin-500.ttf")),
        ("onest-500-ext", include_bytes!("../assets/fonts/onest-latin-ext-500.ttf")),
        ("onest-600", include_bytes!("../assets/fonts/onest-latin-600.ttf")),
        ("onest-600-ext", include_bytes!("../assets/fonts/onest-latin-ext-600.ttf")),
        (
            "shoulders-800",
            include_bytes!("../assets/fonts/big-shoulders-display-latin-800.ttf"),
        ),
        (
            "shoulders-800-ext",
            include_bytes!("../assets/fonts/big-shoulders-display-latin-ext-800.ttf"),
        ),
    ];
    for (name, bytes) in embedded {
        fonts.font_data.insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    }
    fonts
        .font_data
        .insert("phosphor".to_owned(), Arc::new(egui_phosphor::Variant::Regular.font_data()));
    fonts
        .font_data
        .insert("phosphor-fill".to_owned(), Arc::new(egui_phosphor::Variant::Fill.font_data()));
    let system = [
        ("segoe", "segoeui.ttf"),
        ("segoe-bold", "segoeuib.ttf"),
        ("yahei", "msyh.ttc"),
        ("malgun", "malgun.ttf"),
        ("thai", "leelawui.ttf"),
        ("indic", "Nirmala.ttc"),
        ("indic-legacy", "Nirmala.ttf"),
        ("symbols", "seguisym.ttf"),
    ];
    for (name, file) in system {
        if let Some(bytes) = system_font(file) {
            fonts.font_data.insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
        }
    }
    let known = fonts.font_data.clone();
    let stack = |own: [&str; 2], system: &str| -> Vec<String> {
        let fallbacks = [
            system,
            "NotoEmoji-Regular",
            "emoji-icon-font",
            "yahei",
            "malgun",
            "thai",
            "indic",
            "indic-legacy",
            "symbols",
            "Ubuntu-Light",
        ];
        own.into_iter()
            .chain(fallbacks)
            .filter(|name| known.contains_key(*name))
            .map(str::to_owned)
            .collect()
    };
    fonts
        .families
        .insert(FontFamily::Proportional, stack(["onest-400", "onest-400-ext"], "segoe"));
    fonts
        .families
        .insert(FAMILIES[0].clone(), stack(["onest-500", "onest-500-ext"], "segoe"));
    fonts
        .families
        .insert(FAMILIES[1].clone(), stack(["onest-600", "onest-600-ext"], "segoe-bold"));
    fonts
        .families
        .insert(FAMILIES[2].clone(), stack(["shoulders-800", "shoulders-800-ext"], "segoe-bold"));
    fonts.families.insert(FAMILIES[3].clone(), vec!["phosphor".to_owned()]);
    fonts.families.insert(FAMILIES[4].clone(), vec!["phosphor-fill".to_owned()]);
    ctx.set_fonts(fonts);

    ctx.style_mut(|style| {
        let visuals = &mut style.visuals;
        *visuals = egui::Visuals::dark();
        visuals.panel_fill = BASE;
        visuals.window_fill = rgb(0x1B1C21);
        visuals.extreme_bg_color = PANEL;
        visuals.override_text_color = Some(TEXT);
        visuals.window_stroke = Stroke::new(1.0, veil(0.08));
        visuals.window_corner_radius = 10.into();
        visuals.menu_corner_radius = 13.into();
        visuals.popup_shadow = Shadow {
            offset: [0, 10],
            blur: 28,
            spread: 0,
            color: shade(0.5),
        };
        visuals.selection.bg_fill = ACCENT.gamma_multiply(0.45);
        visuals.selection.stroke = Stroke::new(1.0, TEXT);
        visuals.text_cursor.stroke = Stroke::new(1.5, ACCENT);
        visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
        for (widget, fill) in [
            (&mut visuals.widgets.inactive, Color32::TRANSPARENT),
            (&mut visuals.widgets.hovered, veil(0.08)),
            (&mut visuals.widgets.active, veil(0.12)),
            (&mut visuals.widgets.open, veil(0.08)),
        ] {
            widget.bg_fill = fill;
            widget.weak_bg_fill = fill;
            widget.bg_stroke = Stroke::NONE;
            widget.fg_stroke = Stroke::new(1.0, TEXT);
            widget.corner_radius = 6.into();
            widget.expansion = 0.0;
        }
        style.spacing.item_spacing = Vec2::ZERO;
        style.spacing.button_padding = vec2(12.0, 8.0);
        style.spacing.menu_margin = Margin::same(6);
        style.spacing.scroll = ScrollStyle {
            bar_width: 8.0,
            floating_width: 4.0,
            dormant_handle_opacity: 0.0,
            ..ScrollStyle::floating()
        };
        style.interaction.selectable_labels = false;
        style.animation_time = HOVER_TIME;
        style.text_styles = [
            (TextStyle::Body, sans(14.0)),
            (TextStyle::Button, medium(13.5)),
            (TextStyle::Small, sans(12.0)),
            (TextStyle::Heading, bold(20.0)),
            (TextStyle::Monospace, FontId::monospace(13.0)),
        ]
        .into();
    });
}

pub fn app_icon() -> egui::IconData {
    const SIZE: usize = 128;
    const SAMPLES: usize = 3;
    let mut rgba = Vec::with_capacity(SIZE * SIZE * 4);
    for pixel in 0..SIZE * SIZE {
        let (mut covered, mut sum) = (0.0, [0.0; 3]);
        for sample in 0..SAMPLES * SAMPLES {
            let x = (pixel % SIZE) as f32 + ((sample % SAMPLES) as f32 + 0.5) / SAMPLES as f32;
            let y = (pixel / SIZE) as f32 + ((sample / SAMPLES) as f32 + 0.5) / SAMPLES as f32;
            let radius = (x / SIZE as f32 * 2.0 - 1.0).hypot(y / SIZE as f32 * 2.0 - 1.0);
            if radius < 0.96 {
                let color = if radius < 0.17 || (0.53..0.57).contains(&radius) {
                    PANEL
                } else {
                    ACCENT
                };
                covered += 1.0;
                sum = [sum[0] + color.r() as f32, sum[1] + color.g() as f32, sum[2] + color.b() as f32];
            }
        }
        let alpha = covered / (SAMPLES * SAMPLES) as f32 * 255.0;
        rgba.extend(sum.map(|channel| (channel / f32::max(covered, 1.0)) as u8));
        rgba.push(alpha as u8);
    }
    egui::IconData {
        rgba,
        width: SIZE as u32,
        height: SIZE as u32,
    }
}

struct Cx<'a> {
    art: &'a Art,
    covers: &'a Covers,
    playing: &'a str,
    paused: bool,
    liked: &'a [Item],
    single: bool,
    out: &'a mut Vec<Action>,
}

impl Cx<'_> {
    fn is_liked(&self, item: &Item) -> bool {
        item.is_song() && self.liked.iter().any(|liked| liked.video_id == item.video_id)
    }

    fn is_playing(&self, item: &Item) -> bool {
        item.is_song() && item.video_id == self.playing
    }

    fn activate(&mut self, items: &[Item], index: usize) {
        let item = &items[index];
        if self.is_playing(item) {
            self.out.push(Action::Toggle);
        } else if item.is_song() {
            let (queue, at) = if self.single {
                (vec![item.clone()], 0)
            } else {
                (items.to_vec(), index)
            };
            self.out.push(Action::Play(queue, at));
        } else if !item.browse_id.is_empty() {
            self.out.push(Action::Go(Route::Browse(item.browse_id.clone())));
        }
    }
}

struct Scrub {
    bar: Rect,
    played: f32,
    buffered: f32,
    hover: f32,
}

pub fn draw(app: &mut App, ctx: &egui::Context) -> Vec<Action> {
    let mut out = Vec::new();
    let plain = |fill: Color32| Frame::new().fill(fill);
    let scrub = egui::TopBottomPanel::bottom("player")
        .exact_height(BAR_HEIGHT)
        .show_separator_line(false)
        .frame(plain(PANEL))
        .show(ctx, |ui| player_bar(app, ui, &mut out))
        .inner;
    egui::SidePanel::left("sidebar")
        .exact_width(SIDEBAR_WIDTH)
        .resizable(false)
        .show_separator_line(false)
        .frame(plain(PANEL))
        .show(ctx, |ui| sidebar(app, ui, &mut out));
    let queue_fits = app.saved.queue_open && ctx.content_rect().width() >= QUEUE_MIN_WINDOW;
    egui::SidePanel::right("queue")
        .exact_width(QUEUE_WIDTH)
        .resizable(false)
        .show_separator_line(false)
        .frame(plain(PANEL))
        .show_animated(ctx, queue_fits, |ui| queue_panel(app, ui, &mut out));
    egui::CentralPanel::default()
        .frame(plain(BASE))
        .show(ctx, |ui| content(app, ui, &mut out));
    paint_scrubber(&ctx.layer_painter(egui::LayerId::background()), &scrub);
    sign_in_dialog(app, ctx, &mut out);
    notice(app, ctx);
    out
}

fn fit(ui: &Ui, text: &str, font: FontId, color: Color32, width: f32, rows: usize) -> Arc<Galley> {
    let mut job = LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = TextWrapping {
        max_width: width.max(1.0),
        max_rows: rows,
        break_anywhere: rows == 1,
        overflow_character: Some('…'),
    };
    ui.painter().layout_job(job)
}

fn label(ui: &Ui, at: Pos2, anchor: Align2, text: &str, font: FontId, color: Color32, width: f32) -> Rect {
    let galley = fit(ui, text, font, color, width, 1);
    let rect = anchor.anchor_size(at, galley.size());
    ui.painter().galley(rect.min, galley, color);
    rect
}

fn paragraph(ui: &mut Ui, text: &str, font: FontId, color: Color32, rows: usize) {
    let galley = fit(ui, text, font, color, ui.available_width(), rows);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), galley.size().y), Sense::hover());
    ui.painter().galley(rect.min, galley, color);
}

fn eyebrow(ui: &mut Ui, text: &str, inset: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::hover());
    let format = TextFormat {
        font_id: bold(11.0),
        color: FAINT,
        extra_letter_spacing: 1.3,
        ..TextFormat::default()
    };
    let mut job = LayoutJob::default();
    job.append(&text.to_uppercase(), 0.0, format);
    let galley = ui.painter().layout_job(job);
    ui.painter()
        .galley(pos2(rect.left() + inset, rect.center().y - galley.size().y / 2.0), galley, FAINT);
}

fn hover_of(ui: &Ui, response: &Response) -> f32 {
    ui.ctx()
        .animate_bool_with_time(response.id, response.hovered() || response.has_focus(), HOVER_TIME)
}

fn describe(response: &Response, name: &str) {
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, name));
}

fn icon_button(ui: &Ui, rect: Rect, id: Id, symbol: &str, font: FontId, color: Color32, hint: &str) -> Response {
    let response = ui.interact(rect, id, Sense::click());
    let hover = hover_of(ui, &response);
    let pressed = if response.is_pointer_button_down_on() { 1.0 } else { 0.0 };
    ui.painter()
        .circle_filled(rect.center(), rect.width() / 2.0 - pressed, veil(0.09 * hover));
    let tone = if color == MUTED || color == FAINT {
        color.lerp_to_gamma(TEXT, hover)
    } else {
        color
    };
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, symbol, font, tone);
    describe(&response, hint);
    response.on_hover_text(hint)
}

fn pill(ui: &Ui, at: Pos2, id: Id, symbol: &str, text: &str, primary: bool) -> Response {
    let galley = fit(ui, text, bold(14.0), if primary { ON_ACCENT } else { TEXT }, 240.0, 1);
    let rect = Rect::from_min_size(at, vec2(galley.size().x + 62.0, 42.0));
    let response = ui.interact(rect, id, Sense::click());
    let hover = hover_of(ui, &response);
    let body = if response.is_pointer_button_down_on() {
        rect.shrink(1.0)
    } else {
        rect
    };
    let fill = if primary {
        ACCENT.lerp_to_gamma(Color32::WHITE, 0.12 * hover)
    } else {
        veil(0.09 + 0.05 * hover)
    };
    let ink = if primary { ON_ACCENT } else { TEXT };
    ui.painter().rect_filled(body, 21.0, fill);
    ui.painter().text(
        pos2(rect.left() + 28.0, rect.center().y),
        Align2::CENTER_CENTER,
        symbol,
        solid(16.0),
        ink,
    );
    ui.painter()
        .galley(pos2(rect.left() + 42.0, rect.center().y - galley.size().y / 2.0), galley, ink);
    describe(&response, text);
    response
}

fn artwork(ui: &Ui, art: &Art, url: &str, rect: Rect, radius: f32) {
    match art.get(ui.ctx(), url, rect.width()) {
        Some(picture) => egui::Image::from_texture((picture.id, rect.size()))
            .corner_radius(radius)
            .paint_at(ui, rect),
        None => drop(ui.painter().rect_filled(rect, radius, RAISED)),
    }
}

fn bars(ui: &Ui, center: Pos2, moving: bool) {
    let time = if moving { ui.input(|input| input.time) } else { 0.9 };
    for bar in 0..3 {
        let wave = (time * (5.2 + bar as f64 * 1.9) + bar as f64 * 2.1).sin() as f32;
        let x = center.x + (bar as f32 - 1.0) * 5.0;
        let top = center.y + 7.0 - (4.0 + 5.0 * (1.0 + wave));
        ui.painter()
            .rect_filled(Rect::from_min_max(pos2(x - 1.5, top), pos2(x + 1.5, center.y + 7.0)), 1.0, ACCENT);
    }
    if moving && ui.input(|input| input.focused) {
        ui.ctx().request_repaint_after(Duration::from_millis(90));
    }
}

fn clock(ms: u64) -> String {
    format!("{}:{:02}", ms / 60_000, ms / 1000 % 60)
}

fn menu_row(ui: &mut Ui, symbol: &str, font: FontId, tint: Color32, text: &str) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(MENU_WIDTH, 38.0), Sense::click());
    let hover = hover_of(ui, &response);
    ui.painter().rect_filled(rect, 7.0, veil(0.08 * hover));
    let tone = if tint == ACCENT { tint } else { tint.lerp_to_gamma(TEXT, hover) };
    ui.painter()
        .text(pos2(rect.left() + 22.0, rect.center().y), Align2::CENTER_CENTER, symbol, font, tone);
    label(
        ui,
        pos2(rect.left() + 46.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        medium(13.5),
        TEXT,
        rect.width() - 58.0,
    );
    describe(&response, text);
    response.clicked()
}

fn menu_divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(MENU_WIDTH, 11.0), Sense::hover());
    ui.painter().hline(
        rect.shrink2(vec2(8.0, 0.0)).x_range(),
        rect.center().y,
        Stroke::new(1.0, veil(0.08)),
    );
}

fn song_actions(ui: &mut Ui, cx: &mut Cx, item: &Item) {
    ui.set_width(MENU_WIDTH);
    let (head, _) = ui.allocate_exact_size(vec2(MENU_WIDTH, 56.0), Sense::hover());
    let art = Rect::from_min_size(pos2(head.left() + 8.0, head.center().y - 20.0), Vec2::splat(40.0));
    artwork(ui, cx.art, cx.covers.of(item), art, 5.0);
    let (left, width) = (art.right() + 12.0, head.right() - art.right() - 20.0);
    label(
        ui,
        pos2(left, head.center().y - 9.0),
        Align2::LEFT_CENTER,
        &item.title,
        medium(13.5),
        TEXT,
        width,
    );
    label(
        ui,
        pos2(left, head.center().y + 9.0),
        Align2::LEFT_CENTER,
        &item.subtitle,
        sans(12.5),
        MUTED,
        width,
    );
    menu_divider(ui);

    let mut choice = None;
    if menu_row(ui, icon::ARROW_BEND_DOWN_RIGHT, glyph(17.0), MUTED, "Play next") {
        choice = Some(Action::PlayNext(item.clone()));
    }
    if menu_row(ui, icon::LIST_PLUS, glyph(17.0), MUTED, "Add to queue") {
        choice = Some(Action::Enqueue(item.clone()));
    }
    let (font, tint, text) = if cx.is_liked(item) {
        (solid(17.0), ACCENT, "Remove from Liked songs")
    } else {
        (glyph(17.0), MUTED, "Add to Liked songs")
    };
    if menu_row(ui, icon::HEART, font, tint, text) {
        choice = Some(Action::Like(item.clone()));
    }
    if !item.artist_id.is_empty() || !item.album_id.is_empty() {
        menu_divider(ui);
    }
    if !item.artist_id.is_empty() && menu_row(ui, icon::MICROPHONE_STAGE, glyph(17.0), MUTED, "Go to artist") {
        choice = Some(Action::Go(Route::Browse(item.artist_id.clone())));
    }
    if !item.album_id.is_empty() && menu_row(ui, icon::VINYL_RECORD, glyph(17.0), MUTED, "Go to album") {
        choice = Some(Action::Go(Route::Browse(item.album_id.clone())));
    }
    if let Some(action) = choice {
        cx.out.push(action);
        ui.close();
    }
}

fn song_menu(response: &Response, cx: &mut Cx, item: &Item) {
    if item.is_song() {
        response.context_menu(|ui| song_actions(ui, cx, item));
    }
}

fn track_row(ui: &mut Ui, cx: &mut Cx, items: &[Item], index: usize, number: Option<usize>) {
    let item = &items[index];
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::click());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let more_slot = Rect::from_center_size(pos2(rect.right() - 24.0, rect.center().y), Vec2::splat(30.0));
    let more = item
        .is_song()
        .then(|| ui.interact(more_slot, response.id.with("more"), Sense::click()));
    let more_open = more.as_ref().is_some_and(|more| egui::Popup::menu(more).is_open());
    let engaged = response.hovered() || response.context_menu_opened() || more_open;
    let hover = ui.ctx().animate_bool_with_time(response.id, engaged, HOVER_TIME);
    let playing = cx.is_playing(item);
    let mid = rect.center().y;
    ui.painter().rect_filled(rect, ROW_RADIUS, veil(0.06 * hover));
    if response.has_focus() {
        ui.painter()
            .rect_stroke(rect, ROW_RADIUS, Stroke::new(1.0, MUTED), StrokeKind::Inside);
    }

    let mut left = rect.left() + 12.0;
    match number {
        Some(number) => {
            let slot = pos2(left + 12.0, mid);
            if playing {
                bars(ui, slot, !cx.paused);
            } else {
                ui.painter()
                    .text(slot, Align2::CENTER_CENTER, number.to_string(), sans(13.5), FAINT);
            }
            left += 40.0;
        }
        None => {
            let art = Rect::from_min_size(pos2(left, mid - 20.0), Vec2::splat(40.0));
            artwork(ui, cx.art, cx.covers.of(item), art, if item.is_artist { 20.0 } else { 5.0 });
            if playing {
                ui.painter().rect_filled(art, 5.0, shade(0.6));
                bars(ui, art.center(), !cx.paused);
            }
            left += 54.0;
        }
    }

    let mut right = rect.right() - 14.0;
    if let Some(more) = &more {
        if hover > 0.0 {
            let lift = hover_of(ui, more);
            ui.painter().circle_filled(more_slot.center(), 15.0, veil(0.09 * lift));
            ui.painter().text(
                more_slot.center(),
                Align2::CENTER_CENTER,
                icon::DOTS_THREE,
                glyph(19.0),
                MUTED.gamma_multiply(hover).lerp_to_gamma(TEXT, lift),
            );
        }
        describe(more, "More");
        egui::Popup::menu(more).gap(4.0).show(|ui| song_actions(ui, cx, item));
        right -= 36.0;
        label(ui, pos2(right, mid), Align2::RIGHT_CENTER, &item.duration, sans(13.0), MUTED, 60.0);
        right -= 52.0;
        let liked = cx.is_liked(item);
        if liked || hover > 0.0 {
            let slot = Rect::from_center_size(pos2(right - 14.0, mid), Vec2::splat(30.0));
            let (font, color) = if liked {
                (solid(16.0), ACCENT)
            } else {
                (glyph(16.0), MUTED.gamma_multiply(hover))
            };
            let hint = if liked { "Remove from Liked songs" } else { "Add to Liked songs" };
            if icon_button(ui, slot, response.id.with("like"), icon::HEART, font, color, hint).clicked() {
                cx.out.push(Action::Like(item.clone()));
            }
        }
        right -= 40.0;
    } else {
        ui.painter()
            .text(pos2(right - 6.0, mid), Align2::CENTER_CENTER, icon::CARET_RIGHT, glyph(14.0), FAINT);
        right -= 28.0;
    }
    if rect.width() > 700.0 && !item.extra.is_empty() {
        let column = rect.width() * 0.26;
        label(
            ui,
            pos2(right - column, mid),
            Align2::LEFT_CENTER,
            &item.extra,
            sans(13.0),
            MUTED,
            column - 12.0,
        );
        right -= column + 20.0;
    }

    let title = if playing { ACCENT } else { TEXT };
    label(
        ui,
        pos2(left, mid - 10.0),
        Align2::LEFT_CENTER,
        &item.title,
        medium(14.0),
        title,
        right - left,
    );
    label(
        ui,
        pos2(left, mid + 10.0),
        Align2::LEFT_CENTER,
        &item.subtitle,
        sans(13.0),
        MUTED,
        right - left,
    );

    describe(&response, &item.title);
    if response.clicked() {
        cx.activate(items, index);
    }
    song_menu(&response, cx, item);
}

fn card(ui: &Ui, cx: &mut Cx, items: &[Item], index: usize, cell: Rect, id: Id) {
    let item = &items[index];
    let response = ui.interact(cell, id, Sense::click());
    let hover = ui.ctx().animate_bool_with_time(
        id,
        response.hovered() || response.has_focus() || response.context_menu_opened(),
        0.14,
    );
    let playing = cx.is_playing(item);
    let art = Rect::from_min_size(cell.min, Vec2::splat(cell.width()));
    let radius = if item.is_artist { art.width() / 2.0 } else { ART_RADIUS };
    artwork(ui, cx.art, cx.covers.of(item), art, radius);
    if hover > 0.0 || playing {
        let lit = if playing { 1.0 } else { hover };
        ui.painter().rect_filled(art, radius, shade(0.28 * lit));
        if item.is_song() {
            let badge = pos2(art.right() - 32.0, art.bottom() - 26.0 - 6.0 * lit);
            ui.painter().circle_filled(badge, 22.0, ACCENT.gamma_multiply(lit));
            let symbol = if playing && !cx.paused { icon::PAUSE } else { icon::PLAY };
            ui.painter()
                .text(badge, Align2::CENTER_CENTER, symbol, solid(19.0), ON_ACCENT.gamma_multiply(lit));
        }
    }
    if response.has_focus() {
        ui.painter().rect_stroke(art, radius, Stroke::new(1.5, TEXT), StrokeKind::Inside);
    }
    let title = if playing { ACCENT } else { TEXT };
    label(
        ui,
        pos2(cell.left(), art.bottom() + 22.0),
        Align2::LEFT_CENTER,
        &item.title,
        medium(14.0),
        title,
        cell.width(),
    );
    label(
        ui,
        pos2(cell.left(), art.bottom() + 42.0),
        Align2::LEFT_CENTER,
        &item.subtitle,
        sans(13.0),
        MUTED,
        cell.width(),
    );
    describe(&response, &item.title);
    if response.clicked() {
        cx.activate(items, index);
    }
    song_menu(&response, cx, item);
}

fn compact(ui: &Ui, cx: &mut Cx, items: &[Item], index: usize, cell: Rect, id: Id) {
    let item = &items[index];
    let response = ui.interact(cell, id, Sense::click());
    let hover = ui.ctx().animate_bool_with_time(
        id,
        response.hovered() || response.has_focus() || response.context_menu_opened(),
        HOVER_TIME,
    );
    let playing = cx.is_playing(item);
    ui.painter().rect_filled(cell, ROW_RADIUS, veil(0.06 * hover));
    let art = Rect::from_min_size(pos2(cell.left() + 8.0, cell.center().y - 24.0), Vec2::splat(48.0));
    artwork(ui, cx.art, cx.covers.of(item), art, if item.is_artist { 24.0 } else { 6.0 });
    if playing {
        ui.painter().rect_filled(art, 6.0, shade(0.6));
        bars(ui, art.center(), !cx.paused);
    }
    let (left, width) = (art.right() + 14.0, cell.right() - art.right() - 26.0);
    let title = if playing { ACCENT } else { TEXT };
    label(
        ui,
        pos2(left, cell.center().y - 10.0),
        Align2::LEFT_CENTER,
        &item.title,
        medium(14.0),
        title,
        width,
    );
    label(
        ui,
        pos2(left, cell.center().y + 10.0),
        Align2::LEFT_CENTER,
        &item.subtitle,
        sans(13.0),
        MUTED,
        width,
    );
    describe(&response, &item.title);
    if response.clicked() {
        cx.activate(items, index);
    }
    song_menu(&response, cx, item);
}

fn columns(width: f32, minimum: f32, gap: f32) -> (usize, f32) {
    let count = (((width + gap) / (minimum + gap)).floor() as usize).max(1);
    (count, ((width - gap * (count - 1) as f32) / count as f32).floor())
}

fn grid(ui: &mut Ui, count: usize, columns: usize, cell: Vec2, gap: Vec2, mut paint: impl FnMut(&Ui, usize, Rect)) {
    let rows = count.div_ceil(columns);
    let height = rows as f32 * cell.y + rows.saturating_sub(1) as f32 * gap.y;
    let (area, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    for index in 0..count {
        let offset = vec2(
            (index % columns) as f32 * (cell.x + gap.x),
            (index / columns) as f32 * (cell.y + gap.y),
        );
        let rect = Rect::from_min_size(area.min + offset, cell);
        if ui.is_rect_visible(rect) {
            paint(ui, index, rect);
        }
    }
}

fn section_view(ui: &mut Ui, cx: &mut Cx, title: &str, layout: Layout, items: &[Item], id: Id, numbered: bool) {
    let width = ui.available_width();
    let (cards, card_width) = columns(width, CARD_MIN, CARD_GAP);
    let (cells, cell_width) = columns(width, COMPACT_MIN, 12.0);
    let visible = match layout {
        Layout::Cards => cards,
        Layout::Compact => cells * COMPACT_ROWS,
        Layout::Tracks | Layout::Top => usize::MAX,
    };
    let foldable = items.len() > visible;
    let mut open = ui.data(|data| data.get_temp::<bool>(id)).unwrap_or(false);
    if !title.is_empty() || foldable {
        let (row, _) = ui.allocate_exact_size(vec2(width, 32.0), Sense::hover());
        label(ui, row.left_center(), Align2::LEFT_CENTER, title, bold(20.0), TEXT, width - 120.0);
        if foldable {
            let text = if open { "Show less" } else { "Show all" };
            let galley = fit(ui, text, medium(13.0), MUTED, 120.0, 1);
            let slot = Align2::RIGHT_CENTER
                .anchor_size(row.right_center(), galley.size())
                .expand2(vec2(10.0, 6.0));
            let response = ui.interact(slot, id.with("fold"), Sense::click());
            let hover = hover_of(ui, &response);
            ui.painter().rect_filled(slot, 14.0, veil(0.07 * hover));
            ui.painter()
                .galley_with_override_text_color(slot.shrink2(vec2(10.0, 6.0)).min, galley, MUTED.lerp_to_gamma(TEXT, hover));
            describe(&response, text);
            if response.clicked() {
                open = !open;
                ui.data_mut(|data| data.insert_temp(id, open));
            }
        }
        ui.add_space(12.0);
    }
    let shown = if open { items.len() } else { items.len().min(visible) };
    match layout {
        Layout::Cards => {
            let cell = vec2(card_width, card_width + CARD_TEXT);
            grid(ui, shown, cards, cell, vec2(CARD_GAP, 14.0), |ui, index, rect| {
                card(ui, cx, items, index, rect, id.with(index))
            });
        }
        Layout::Compact => {
            let cell = vec2(cell_width, 64.0);
            grid(ui, shown, cells, cell, vec2(12.0, 2.0), |ui, index, rect| {
                compact(ui, cx, items, index, rect, id.with(index))
            });
        }
        Layout::Tracks => {
            for index in 0..shown {
                track_row(ui, cx, items, index, numbered.then_some(index + 1));
            }
        }
        Layout::Top => {
            top_result(ui, cx, items, id);
            for index in 1..shown {
                track_row(ui, cx, items, index, None);
            }
        }
    }
}

fn top_result(ui: &mut Ui, cx: &mut Cx, items: &[Item], id: Id) {
    let item = &items[0];
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 128.0), Sense::click());
    let hover = hover_of(ui, &response);
    ui.painter().rect_filled(rect, 12.0, veil(0.045 + 0.035 * hover));
    let art = Rect::from_min_size(rect.min + vec2(16.0, 16.0), Vec2::splat(96.0));
    artwork(ui, cx.art, cx.covers.of(item), art, if item.is_artist { 48.0 } else { ART_RADIUS });
    let left = art.right() + 22.0;
    let width = rect.right() - left - 150.0;
    label(
        ui,
        pos2(left, rect.center().y - 16.0),
        Align2::LEFT_CENTER,
        &item.title,
        display(38.0),
        TEXT,
        width,
    );
    label(
        ui,
        pos2(left, rect.center().y + 20.0),
        Align2::LEFT_CENTER,
        &item.subtitle,
        sans(14.0),
        MUTED,
        width,
    );
    let (symbol, text) = if item.is_song() {
        (icon::PLAY, "Play")
    } else {
        (icon::CARET_RIGHT, "Open")
    };
    let button = pill(
        ui,
        pos2(rect.right() - 128.0, rect.center().y - 21.0),
        id.with("top"),
        symbol,
        text,
        item.is_song(),
    );
    describe(&response, &item.title);
    if response.clicked() || button.clicked() {
        cx.activate(items, 0);
    }
    song_menu(&response, cx, item);
    ui.add_space(8.0);
}

fn hero(ui: &mut Ui, cx: &mut Cx, item: &Item, symbol: Option<&str>, playable: bool, pinned: Option<bool>) -> bool {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), HERO_ART), Sense::hover());
    let art = Rect::from_min_size(rect.min, Vec2::splat(HERO_ART));
    match symbol {
        Some(symbol) => {
            ui.painter().rect_filled(art, 12.0, veil(0.06));
            ui.painter().text(art.center(), Align2::CENTER_CENTER, symbol, solid(84.0), ACCENT);
        }
        None => artwork(ui, cx.art, &item.thumb, art, if item.is_artist { HERO_ART / 2.0 } else { 12.0 }),
    }
    let left = art.right() + 30.0;
    let width = (rect.right() - left).max(120.0);
    let mut bottom = rect.bottom();

    let mut at = pos2(left, bottom - 42.0);
    let mut play_clicked = false;
    if playable {
        let play = pill(ui, at, ui.id().with("hero-play"), icon::PLAY, "Play", true);
        at.x = play.rect.right() + 10.0;
        play_clicked = play.clicked();
    }
    if let Some(pinned) = pinned {
        let slot = Rect::from_min_size(at, Vec2::splat(42.0));
        let (font, color, hint) = if pinned {
            (solid(19.0), ACCENT, "Remove from library")
        } else {
            (glyph(19.0), MUTED, "Save to library")
        };
        if icon_button(ui, slot, ui.id().with("hero-pin"), icon::BOOKMARK_SIMPLE, font, color, hint).clicked() {
            cx.out.push(Action::Pin(item.clone()));
        }
    }
    bottom -= 60.0;

    for (text, font, color) in [(&item.subtitle, sans(14.0), MUTED), (&item.extra, medium(15.0), TEXT)] {
        if !text.is_empty() {
            let line = label(ui, pos2(left, bottom), Align2::LEFT_BOTTOM, text, font, color, width);
            bottom = line.top() - 5.0;
        }
    }
    let size = match item.title.chars().count() {
        0..=16 => 68.0,
        17..=34 => 50.0,
        _ => 38.0,
    };
    let title = fit(ui, &item.title, display(size), TEXT, width, 2);
    ui.painter().galley(pos2(left, bottom - title.size().y), title, TEXT);
    play_clicked
}

fn skeleton(ui: &mut Ui) {
    let pulse = veil(0.05 + 0.025 * (ui.input(|input| input.time) * 3.2).sin() as f32);
    ui.ctx().request_repaint_after(Duration::from_millis(60));
    let (count, side) = columns(ui.available_width(), CARD_MIN, CARD_GAP);
    for _ in 0..2 {
        let (heading, _) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
        ui.painter()
            .rect_filled(Rect::from_min_size(heading.min + vec2(0.0, 5.0), vec2(190.0, 22.0)), 6.0, pulse);
        ui.add_space(12.0);
        grid(
            ui,
            count,
            count,
            vec2(side, side + CARD_TEXT),
            vec2(CARD_GAP, 0.0),
            |ui, _, cell| {
                ui.painter()
                    .rect_filled(Rect::from_min_size(cell.min, Vec2::splat(side)), ART_RADIUS, pulse);
                ui.painter().rect_filled(
                    Rect::from_min_size(cell.min + vec2(0.0, side + 14.0), vec2(side * 0.7, 13.0)),
                    4.0,
                    pulse,
                );
                ui.painter().rect_filled(
                    Rect::from_min_size(cell.min + vec2(0.0, side + 35.0), vec2(side * 0.45, 12.0)),
                    4.0,
                    pulse,
                );
            },
        );
        ui.add_space(32.0);
    }
}

fn message(ui: &mut Ui, symbol: &str, title: &str, hint: &str) -> Rect {
    ui.add_space(72.0);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 132.0), Sense::hover());
    let center = rect.center_top();
    ui.painter().circle_filled(center + vec2(0.0, 34.0), 34.0, veil(0.06));
    ui.painter()
        .text(center + vec2(0.0, 34.0), Align2::CENTER_CENTER, symbol, glyph(30.0), MUTED);
    label(
        ui,
        center + vec2(0.0, 92.0),
        Align2::CENTER_CENTER,
        title,
        bold(18.0),
        TEXT,
        rect.width(),
    );
    label(
        ui,
        center + vec2(0.0, 118.0),
        Align2::CENTER_CENTER,
        hint,
        sans(14.0),
        MUTED,
        rect.width(),
    );
    rect
}

fn local(ui: &mut Ui, cx: &mut Cx, title: &str, symbol: &str, items: &[Item], empty: [&str; 2]) {
    let count = if items.len() == 1 {
        "1 song".to_owned()
    } else {
        format!("{} songs", items.len())
    };
    let header = Item {
        title: title.to_owned(),
        subtitle: count,
        ..Item::default()
    };
    if hero(ui, cx, &header, Some(symbol), !items.is_empty(), None) {
        cx.out.push(Action::Play(items.to_vec(), 0));
    }
    ui.add_space(28.0);
    if items.is_empty() {
        message(ui, symbol, empty[0], empty[1]);
    }
    for index in 0..items.len() {
        track_row(ui, cx, items, index, None);
    }
}

fn page(ui: &mut Ui, cx: &mut Cx, load: Option<&Load>, pinned: &[Item], query: Option<&str>) {
    match load {
        Some(Load::Ready(page)) => {
            if let Some(header) = &page.header {
                let saved = pinned.iter().any(|item| item.browse_id == header.browse_id);
                let playable = page.sections.iter().flat_map(|section| &section.items).any(Item::is_song);
                if hero(ui, cx, header, None, playable, Some(saved)) {
                    cx.out.push(Action::Play(page.songs(), 0));
                }
                ui.add_space(8.0);
            }
            if let (true, Some(query)) = (page.sections.is_empty(), query) {
                message(
                    ui,
                    icon::MAGNIFYING_GLASS,
                    &format!("Nothing found for \"{query}\""),
                    "Check the spelling or try fewer words.",
                );
            }
            for (index, section) in page.sections.iter().enumerate() {
                ui.add_space(if index == 0 && page.header.is_none() { 4.0 } else { 30.0 });
                let numbered = section.layout == Layout::Tracks
                    && page
                        .header
                        .as_ref()
                        .is_some_and(|header| !header.is_artist && section.items.iter().all(|item| item.thumb == header.thumb));
                let id = ui.id().with(index);
                section_view(ui, cx, &section.title, section.layout, &section.items, id, numbered);
            }
            if page.partial {
                ui.add_space(30.0);
                skeleton(ui);
            }
        }
        Some(Load::Failed(error)) => {
            let rect = message(ui, icon::WARNING_CIRCLE, "This page did not load", error);
            ui.add_space(12.0);
            let (row, _) = ui.allocate_exact_size(vec2(rect.width(), 42.0), Sense::hover());
            if pill(
                ui,
                pos2(row.center().x - 70.0, row.top()),
                ui.id().with("retry"),
                icon::ARROW_CLOCKWISE,
                "Try again",
                false,
            )
            .clicked()
            {
                cx.out.push(Action::Reload);
            }
        }
        _ => skeleton(ui),
    }
}

fn ambient(ui: &Ui, area: Rect, tint: Option<Color32>) {
    let target = tint.map_or(BASE, |tint| {
        let mut color = Hsva::from(tint);
        color.s = (color.s * 1.5).min(0.8);
        color.v = 0.36;
        BASE.lerp_to_gamma(Color32::from(color), 0.8)
    });
    let channel = |index: usize, value: u8| ui.ctx().animate_value_with_time(Id::new(("ambient", index)), value as f32, 0.6) as u8;
    let top = Color32::from_rgb(channel(0, target.r()), channel(1, target.g()), channel(2, target.b()));
    if top == BASE {
        return;
    }
    let rect = Rect::from_min_size(area.min, vec2(area.width(), AMBIENT_HEIGHT.min(area.height())));
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.left_bottom(), BASE);
    mesh.colored_vertex(rect.right_bottom(), BASE);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 2, 3);
    ui.painter().add(Shape::mesh(mesh));
}

fn topbar(app: &mut App, ui: &mut Ui, out: &mut Vec<Action>) {
    let (bar, _) = ui.allocate_exact_size(vec2(ui.available_width(), TOPBAR_HEIGHT), Sense::hover());
    let mid = bar.center().y;
    let mut left = bar.left() + 24.0;
    let steps = [
        (icon::CARET_LEFT, app.at > 0, Action::Back, "Back"),
        (icon::CARET_RIGHT, app.at + 1 < app.history.len(), Action::Forward, "Forward"),
    ];
    for (symbol, enabled, action, hint) in steps {
        let slot = Rect::from_center_size(pos2(left + 18.0, mid), Vec2::splat(36.0));
        if !enabled {
            ui.painter()
                .text(slot.center(), Align2::CENTER_CENTER, symbol, glyph(17.0), FAINT.gamma_multiply(0.5));
        } else if icon_button(ui, slot, Id::new(hint), symbol, glyph(17.0), MUTED, hint).clicked() {
            out.push(action);
        }
        left += 40.0;
    }
    left += 10.0;

    let field = Rect::from_min_size(
        pos2(left, mid - 21.0),
        vec2((bar.right() - PAGE_MARGIN as f32 - left).min(440.0), 42.0),
    );
    let background = ui.painter().add(Shape::Noop);
    ui.painter().text(
        pos2(field.left() + 22.0, mid),
        Align2::CENTER_CENTER,
        icon::MAGNIFYING_GLASS,
        glyph(17.0),
        MUTED,
    );
    let input = Rect::from_min_max(pos2(field.left() + 44.0, field.top()), pos2(field.right() - 64.0, field.bottom()));
    let layout = egui::Layout::left_to_right(Align::Center);
    let response = ui
        .scope_builder(UiBuilder::new().max_rect(input).layout(layout), |ui| {
            let edit = egui::TextEdit::singleline(&mut app.query)
                .id(Id::new("search"))
                .frame(false)
                .margin(Margin::ZERO)
                .desired_width(input.width())
                .font(sans(14.5))
                .text_color(TEXT)
                .hint_text(RichText::new("Search songs, albums, artists").color(FAINT));
            ui.add(edit)
        })
        .inner;
    if std::mem::take(&mut app.focus_search) {
        response.request_focus();
    }
    if response.changed() {
        app.typed = Some(Instant::now());
    }
    let query = app.query.trim();
    if response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter)) && !query.is_empty() {
        app.typed = None;
        out.push(Action::Go(Route::Search(query.to_owned())));
    }
    let focus = ui.ctx().animate_bool_with_time(Id::new("search-focus"), response.has_focus(), 0.12);
    let stroke = Stroke::new(1.0, veil(0.08 + 0.42 * focus));
    ui.painter().set(
        background,
        RectShape::new(field, 21.0, veil(0.07 + 0.02 * focus), stroke, StrokeKind::Inside),
    );
    let trailing = pos2(field.right() - 24.0, mid);
    if app.query.is_empty() {
        ui.painter()
            .text(pos2(field.right() - 16.0, mid), Align2::RIGHT_CENTER, "Ctrl K", sans(12.0), FAINT);
    } else if icon_button(
        ui,
        Rect::from_center_size(trailing, Vec2::splat(30.0)),
        Id::new("search-clear"),
        icon::X,
        glyph(14.0),
        MUTED,
        "Clear",
    )
    .clicked()
    {
        app.query.clear();
        app.typed = None;
        response.request_focus();
    }
    ui.allocate_rect(bar, Sense::hover());
}

fn content(app: &mut App, ui: &mut Ui, out: &mut Vec<Action>) {
    let area = ui.max_rect();
    let route = app.route().clone();
    let header = match app.pages.get(&route) {
        Some(Load::Ready(page)) => page.header.as_ref().map(|header| (header.thumb.as_str(), HERO_ART)),
        _ => None,
    };
    let playing = app.now_playing().map(|item| (app.saved.covers.of(item), BAR_ART));
    let picture = |source: Option<(&str, f32)>| source.and_then(|(thumb, size)| app.art.get(ui.ctx(), thumb, size));
    let tint = picture(header).or_else(|| picture(playing)).map(|picture| picture.tint);
    ambient(ui, area, tint);
    topbar(app, ui, out);

    let mut cx = Cx {
        art: &app.art,
        covers: &app.saved.covers,
        playing: app.now_playing().map_or("", |item| item.video_id.as_str()),
        paused: app.player.paused(),
        liked: &app.saved.liked,
        single: matches!(route, Route::Search(_)),
        out,
    };
    let margin = Margin {
        left: PAGE_MARGIN,
        right: PAGE_MARGIN,
        top: 8,
        bottom: 56,
    };
    egui::ScrollArea::vertical().id_salt(&route).auto_shrink(false).show(ui, |ui| {
        Frame::new().inner_margin(margin).show(ui, |ui| {
            ui.set_width(ui.available_width());
            match &route {
                Route::Liked => {
                    let empty = ["No liked songs yet", "Tap the heart on any track to keep it here."];
                    local(ui, &mut cx, "Liked songs", icon::HEART, &app.saved.liked, empty);
                }
                Route::Recent => {
                    let empty = ["Nothing played yet", "Tracks you listen to are remembered here."];
                    local(
                        ui,
                        &mut cx,
                        "Recently played",
                        icon::CLOCK_COUNTER_CLOCKWISE,
                        &app.saved.recent,
                        empty,
                    );
                }
                Route::Search(query) => page(ui, &mut cx, app.pages.get(&route), &app.saved.pinned, Some(query)),
                Route::Home => {
                    if !app.saved.recent.is_empty() {
                        let id = ui.id().with("recent");
                        cx.single = true;
                        section_view(ui, &mut cx, "Jump back in", Layout::Cards, &app.saved.recent, id, false);
                        cx.single = false;
                        ui.add_space(26.0);
                    }
                    page(ui, &mut cx, app.pages.get(&route), &app.saved.pinned, None);
                }
                Route::Browse(_) => page(ui, &mut cx, app.pages.get(&route), &app.saved.pinned, None),
            }
        });
    });
}

fn nav_item(ui: &mut Ui, symbol: &str, text: &str, active: bool, trailing: &str) -> Response {
    let (slot, response) = ui.allocate_exact_size(vec2(ui.available_width(), 42.0), Sense::click());
    let rect = slot.shrink2(vec2(12.0, 1.0));
    let hover = hover_of(ui, &response);
    ui.painter()
        .rect_filled(rect, ROW_RADIUS, veil(if active { 0.08 } else { 0.05 * hover }));
    let tone = if active { TEXT } else { MUTED.lerp_to_gamma(TEXT, hover) };
    let (font, color) = if active { (solid(19.0), ACCENT) } else { (glyph(19.0), tone) };
    ui.painter().text(
        pos2(rect.left() + 24.0, rect.center().y),
        Align2::CENTER_CENTER,
        symbol,
        font,
        color,
    );
    label(
        ui,
        pos2(rect.left() + 48.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        medium(14.0),
        tone,
        rect.width() - 96.0,
    );
    label(
        ui,
        pos2(rect.right() - 14.0, rect.center().y),
        Align2::RIGHT_CENTER,
        trailing,
        sans(12.5),
        FAINT,
        40.0,
    );
    describe(&response, text);
    response
}

fn mark(ui: &Ui, center: Pos2, radius: f32) {
    ui.painter().circle_filled(center, radius, ACCENT);
    ui.painter().circle_stroke(center, radius * 0.55, Stroke::new(radius * 0.09, PANEL));
    ui.painter().circle_filled(center, radius * 0.18, PANEL);
}

fn sidebar(app: &App, ui: &mut Ui, out: &mut Vec<Action>) {
    let area = ui.max_rect();
    ui.painter().vline(area.right() - 0.5, area.y_range(), Stroke::new(1.0, veil(0.06)));
    let (brand, _) = ui.allocate_exact_size(vec2(area.width(), 76.0), Sense::hover());
    let center = pos2(brand.left() + 37.0, brand.center().y + 3.0);
    mark(ui, center, 12.0);
    ui.painter()
        .text(pos2(center.x + 22.0, center.y), Align2::LEFT_CENTER, APP_NAME, display(29.0), TEXT);

    let count = |items: &[Item]| if items.is_empty() { String::new() } else { items.len().to_string() };
    let links = [
        (icon::HOUSE, "Home", Route::Home, String::new()),
        (icon::HEART, "Liked songs", Route::Liked, count(&app.saved.liked)),
        (
            icon::CLOCK_COUNTER_CLOCKWISE,
            "Recently played",
            Route::Recent,
            count(&app.saved.recent),
        ),
    ];
    for (symbol, text, route, trailing) in links {
        if nav_item(ui, symbol, text, *app.route() == route, &trailing).clicked() {
            out.push(Action::Go(route));
        }
    }
    ui.add_space(18.0);
    eyebrow(ui, "Library", 26.0);
    let foot = Rect::from_min_max(pos2(area.left(), area.bottom() - ACCOUNT_HEIGHT), area.max);
    account(app, ui, foot, out);
    let pinned = &app.saved.pinned;
    let own = app
        .library
        .iter()
        .filter(|item| !pinned.iter().any(|saved| saved.browse_id == item.browse_id));
    let entries: Vec<&Item> = pinned.iter().chain(own).collect();
    if entries.is_empty() {
        Frame::new().inner_margin(Margin::symmetric(26, 4)).show(ui, |ui| {
            paragraph(ui, "Save an album, playlist or artist and it stays here.", sans(13.0), FAINT, 3);
        });
        return;
    }
    let height = (foot.top() - ui.cursor().top()).max(0.0);
    egui::ScrollArea::vertical().max_height(height).auto_shrink(false).show(ui, |ui| {
        for item in entries {
            let (slot, response) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::click());
            let rect = slot.shrink2(vec2(12.0, 1.0));
            let active = matches!(app.route(), Route::Browse(id) if *id == item.browse_id);
            let hover = hover_of(ui, &response);
            ui.painter()
                .rect_filled(rect, ROW_RADIUS, veil(if active { 0.08 } else { 0.05 * hover }));
            let art = Rect::from_min_size(pos2(rect.left() + 8.0, rect.center().y - 20.0), Vec2::splat(40.0));
            artwork(ui, &app.art, &item.thumb, art, if item.is_artist { 20.0 } else { 5.0 });
            let (left, width) = (art.right() + 12.0, rect.right() - art.right() - 22.0);
            let kind = if item.is_artist { "Artist" } else { item.subtitle.as_str() };
            label(
                ui,
                pos2(left, rect.center().y - 9.0),
                Align2::LEFT_CENTER,
                &item.title,
                medium(13.5),
                TEXT,
                width,
            );
            label(
                ui,
                pos2(left, rect.center().y + 9.0),
                Align2::LEFT_CENTER,
                kind,
                sans(12.5),
                MUTED,
                width,
            );
            describe(&response, &item.title);
            if response.clicked() {
                out.push(Action::Go(Route::Browse(item.browse_id.clone())));
            }
        }
        ui.add_space(12.0);
    });
}

fn account(app: &App, ui: &Ui, foot: Rect, out: &mut Vec<Action>) {
    ui.painter().hline(foot.x_range(), foot.top(), Stroke::new(1.0, veil(0.06)));
    let row = foot.shrink2(vec2(12.0, 10.0));
    let avatar = Rect::from_center_size(pos2(row.left() + 24.0, row.center().y), Vec2::splat(32.0));
    let (left, mid) = (avatar.right() + 12.0, row.center().y);
    match &app.saved.account {
        Some(account) => {
            artwork(ui, &app.art, &account.photo, avatar, 16.0);
            let width = row.right() - left - 44.0;
            label(
                ui,
                pos2(left, mid - 9.0),
                Align2::LEFT_CENTER,
                &account.name,
                medium(13.5),
                TEXT,
                width,
            );
            label(
                ui,
                pos2(left, mid + 9.0),
                Align2::LEFT_CENTER,
                "Signed in",
                sans(12.0),
                MUTED,
                width,
            );
            let slot = Rect::from_center_size(pos2(row.right() - 20.0, mid), Vec2::splat(34.0));
            if icon_button(ui, slot, Id::new("sign-out"), icon::SIGN_OUT, glyph(17.0), MUTED, "Sign out").clicked() {
                out.push(Action::SignOut);
            }
        }
        None => {
            let response = ui.interact(row, Id::new("sign-in"), Sense::click());
            let hover = hover_of(ui, &response);
            let width = row.right() - left - 8.0;
            ui.painter().rect_filled(row, ROW_RADIUS, veil(0.05 * hover));
            ui.painter().text(
                avatar.center(),
                Align2::CENTER_CENTER,
                icon::USER_CIRCLE,
                glyph(27.0),
                MUTED.lerp_to_gamma(TEXT, hover),
            );
            label(ui, pos2(left, mid - 9.0), Align2::LEFT_CENTER, "Sign in", medium(13.5), TEXT, width);
            label(
                ui,
                pos2(left, mid + 9.0),
                Align2::LEFT_CENTER,
                "Mixes and playlists",
                sans(12.0),
                MUTED,
                width,
            );
            describe(&response, "Sign in");
            if response.clicked() {
                out.push(Action::OpenSignIn);
            }
        }
    }
}

fn sign_in_dialog(app: &mut App, ctx: &egui::Context, out: &mut Vec<Action>) {
    let Some(sign_in) = &mut app.sign_in else { return };
    let frame = Frame::new()
        .fill(rgb(0x15161A))
        .stroke(Stroke::new(1.0, veil(0.08)))
        .corner_radius(18)
        .inner_margin(Margin::same(30));
    let modal = egui::Modal::new(Id::new("sign-in-dialog"))
        .frame(frame)
        .backdrop_color(shade(0.6))
        .show(ctx, |ui| {
            ui.set_width(430.0);
            let width = ui.available_width();
            let (head, _) = ui.allocate_exact_size(vec2(width, 44.0), Sense::hover());
            ui.painter()
                .text(head.left_center(), Align2::LEFT_CENTER, "Sign in", display(38.0), TEXT);
            let slot = Rect::from_center_size(pos2(head.right() - 16.0, head.center().y), Vec2::splat(34.0));
            let closed = icon_button(ui, slot, Id::new("sign-in-close"), icon::X, glyph(16.0), MUTED, "Close").clicked();
            ui.add_space(8.0);
            paragraph(ui, "Your own home feed and playlists from YouTube Music.", sans(14.0), MUTED, 2);
            ui.add_space(24.0);

            let (row, _) = ui.allocate_exact_size(vec2(width, 42.0), Sense::hover());
            if pill(ui, row.min, Id::new("sign-in-browser"), icon::GLOBE, "Continue in browser", true).clicked() && !sign_in.busy {
                out.push(Action::BrowserSignIn);
            }
            ui.add_space(12.0);
            paragraph(
                ui,
                "Opens a separate Edge or Chrome window. Sign in to Google there and it closes by itself.",
                sans(13.0),
                MUTED,
                3,
            );
            ui.add_space(22.0);

            eyebrow(ui, "Or paste your cookie", 0.0);
            let steps = [
                "1.  Open music.youtube.com in your browser and sign in.",
                "2.  Press F12, open Network and click a request to music.youtube.com.",
                "3.  Copy the whole cookie value under Request Headers and paste it here.",
            ];
            for step in steps {
                paragraph(ui, step, sans(13.0), MUTED, 2);
                ui.add_space(5.0);
            }
            ui.add_space(7.0);
            let background = ui.painter().add(Shape::Noop);
            let field = egui::ScrollArea::vertical().max_height(74.0).auto_shrink(false).show(ui, |ui| {
                let edit = egui::TextEdit::multiline(&mut sign_in.cookie)
                    .frame(false)
                    .margin(Margin::same(10))
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .font(FontId::monospace(11.5))
                    .text_color(TEXT)
                    .hint_text(RichText::new("SID=...; SAPISID=...; LOGIN_INFO=...").color(FAINT));
                ui.add(edit);
            });
            ui.painter().set(
                background,
                RectShape::new(field.inner_rect, 10.0, veil(0.06), Stroke::new(1.0, veil(0.08)), StrokeKind::Inside),
            );
            ui.add_space(14.0);
            let (row, _) = ui.allocate_exact_size(vec2(width, 42.0), Sense::hover());
            if pill(
                ui,
                row.min,
                Id::new("sign-in-cookie"),
                icon::CLIPBOARD_TEXT,
                "Use this cookie",
                false,
            )
            .clicked()
                && !sign_in.busy
            {
                out.push(Action::CookieSignIn);
            }

            if !sign_in.status.is_empty() {
                ui.add_space(18.0);
                let indent = if sign_in.busy { 26.0 } else { 0.0 };
                let status = fit(ui, &sign_in.status, medium(13.0), TEXT, width - indent, 3);
                let (row, _) = ui.allocate_exact_size(vec2(width, status.size().y.max(18.0)), Sense::hover());
                if sign_in.busy {
                    egui::Spinner::new()
                        .color(ACCENT)
                        .paint_at(ui, Rect::from_min_size(row.min + vec2(0.0, 1.0), Vec2::splat(16.0)));
                }
                ui.painter().galley(row.min + vec2(indent, 0.0), status, TEXT);
            }
            ui.add_space(20.0);
            paragraph(
                ui,
                "The session stays on this computer, encrypted with your Windows account.",
                sans(12.0),
                FAINT,
                2,
            );
            closed
        });
    if modal.inner || modal.should_close() {
        out.push(Action::CloseSignIn);
    }
}

fn queue_panel(app: &App, ui: &mut Ui, out: &mut Vec<Action>) {
    let area = ui.max_rect();
    ui.painter().vline(area.left() + 0.5, area.y_range(), Stroke::new(1.0, veil(0.06)));
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        Frame::new().inner_margin(Margin::same(20)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            let Some(item) = app.now_playing() else {
                message(
                    ui,
                    icon::VINYL_RECORD,
                    "Nothing playing",
                    "Pick a song and the queue lines up here.",
                );
                return;
            };
            let (art, _) = ui.allocate_exact_size(Vec2::splat(ui.available_width()), Sense::hover());
            artwork(ui, &app.art, app.saved.covers.of(item), art, 12.0);
            ui.add_space(18.0);
            paragraph(ui, &item.title, bold(18.0), TEXT, 2);
            ui.add_space(4.0);
            paragraph(ui, &item.subtitle, sans(13.5), MUTED, 1);
            ui.add_space(22.0);

            let upcoming = app.current.map_or(0, |current| current + 1);
            if upcoming >= app.queue.len() {
                return;
            }
            eyebrow(ui, "Next up", 0.0);
            for (index, item) in app.queue.iter().enumerate().skip(upcoming) {
                let (slot, response) = ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::click());
                if !ui.is_rect_visible(slot) {
                    continue;
                }
                let rect = slot.expand2(vec2(8.0, 0.0));
                let hover = hover_of(ui, &response);
                ui.painter().rect_filled(rect, ROW_RADIUS, veil(0.06 * hover));
                let art = Rect::from_min_size(pos2(slot.left(), slot.center().y - 20.0), Vec2::splat(40.0));
                artwork(ui, &app.art, app.saved.covers.of(item), art, 5.0);
                let (left, width) = (art.right() + 12.0, slot.right() - art.right() - 44.0);
                label(
                    ui,
                    pos2(left, slot.center().y - 9.0),
                    Align2::LEFT_CENTER,
                    &item.title,
                    medium(13.5),
                    TEXT,
                    width,
                );
                label(
                    ui,
                    pos2(left, slot.center().y + 9.0),
                    Align2::LEFT_CENTER,
                    &item.subtitle,
                    sans(12.5),
                    MUTED,
                    width,
                );
                describe(&response, &item.title);
                if hover > 0.0 {
                    let close = Rect::from_center_size(pos2(slot.right() - 12.0, slot.center().y), Vec2::splat(28.0));
                    let tone = MUTED.gamma_multiply(hover);
                    if icon_button(
                        ui,
                        close,
                        response.id.with("remove"),
                        icon::X,
                        glyph(13.0),
                        tone,
                        "Remove from queue",
                    )
                    .clicked()
                    {
                        out.push(Action::Remove(index));
                    }
                }
                if response.clicked() {
                    out.push(Action::Jump(index));
                }
            }
        });
    });
}

fn player_bar(app: &App, ui: &mut Ui, out: &mut Vec<Action>) -> Scrub {
    let bar = ui.max_rect();
    let mid = bar.center().y + 3.0;
    let duration = app.player.duration_ms();
    let item = app.now_playing();

    let zone = Rect::from_min_size(bar.min, vec2(bar.width(), 12.0));
    let sense = if duration > 0 { Sense::click_and_drag() } else { Sense::hover() };
    let scrubber = ui.interact(zone, Id::new("scrubber"), sense);
    let pointed = scrubber
        .interact_pointer_pos()
        .map(|pointer| ((pointer.x - bar.left()) / bar.width()).clamp(0.0, 1.0));
    let mut position = app.player.position_ms().min(duration);
    if let (true, Some(fraction)) = (scrubber.dragged() || scrubber.is_pointer_button_down_on(), pointed) {
        position = (fraction * duration as f32) as u64;
    }
    if let (true, Some(fraction)) = (scrubber.clicked() || scrubber.drag_stopped(), pointed) {
        out.push(Action::Seek((fraction * duration as f32) as u64));
    }
    let scrub = Scrub {
        bar,
        played: if duration > 0 { position as f32 / duration as f32 } else { 0.0 },
        buffered: if item.is_some() { app.player.buffered() } else { 0.0 },
        hover: ui
            .ctx()
            .animate_bool_with_time(scrubber.id, scrubber.hovered() || scrubber.dragged(), HOVER_TIME),
    };

    match item {
        Some(item) => {
            let art = Rect::from_min_size(pos2(bar.left() + 18.0, mid - BAR_ART / 2.0), Vec2::splat(BAR_ART));
            artwork(ui, &app.art, app.saved.covers.of(item), art, 6.0);
            let left = art.right() + 14.0;
            let width = (bar.center().x - 170.0 - left - 44.0).max(60.0);
            let title = label(
                ui,
                pos2(left, mid - 10.0),
                Align2::LEFT_CENTER,
                &item.title,
                medium(14.0),
                TEXT,
                width,
            );
            let artist = label(
                ui,
                pos2(left, mid + 10.0),
                Align2::LEFT_CENTER,
                &item.subtitle,
                sans(12.5),
                MUTED,
                width,
            );
            let liked = app.saved.liked.iter().any(|liked| liked.video_id == item.video_id);
            let slot = Rect::from_center_size(pos2(title.right().max(artist.right()) + 26.0, mid), Vec2::splat(34.0));
            let (font, color, hint) = if liked {
                (solid(17.0), ACCENT, "Remove from Liked songs")
            } else {
                (glyph(17.0), MUTED, "Add to Liked songs")
            };
            if icon_button(ui, slot, Id::new("bar-like"), icon::HEART, font, color, hint).clicked() {
                out.push(Action::Like(item.clone()));
            }
        }
        None => {
            let art = Rect::from_min_size(pos2(bar.left() + 18.0, mid - BAR_ART / 2.0), Vec2::splat(BAR_ART));
            ui.painter().rect_filled(art, 6.0, veil(0.05));
            ui.painter()
                .text(art.center(), Align2::CENTER_CENTER, icon::VINYL_RECORD, glyph(22.0), FAINT);
            label(
                ui,
                pos2(art.right() + 14.0, mid),
                Align2::LEFT_CENTER,
                "Nothing playing",
                sans(13.5),
                FAINT,
                200.0,
            );
        }
    }

    let center = pos2(bar.center().x, mid);
    let active = item.is_some();
    let play = Rect::from_center_size(center, Vec2::splat(42.0));
    let toggle = ui.interact(play, Id::new("bar-play"), Sense::click());
    let lift = hover_of(ui, &toggle);
    let pressed = if toggle.is_pointer_button_down_on() { 1.0 } else { 0.0 };
    ui.painter()
        .circle_filled(center, 20.0 + lift - pressed, if active { TEXT } else { veil(0.14) });
    if app.player.loading() {
        egui::Spinner::new()
            .color(PANEL)
            .paint_at(ui, Rect::from_center_size(center, Vec2::splat(18.0)));
    } else {
        let symbol = if app.playing() { icon::PAUSE } else { icon::PLAY };
        ui.painter().text(
            center,
            Align2::CENTER_CENTER,
            symbol,
            solid(19.0),
            if active { PANEL } else { FAINT },
        );
    }
    let name = if app.playing() { "Pause" } else { "Play" };
    describe(&toggle, name);
    if toggle.on_hover_text(name).clicked() {
        out.push(Action::Toggle);
    }
    let around = |offset: f32, size: f32| Rect::from_center_size(center + vec2(offset, 0.0), Vec2::splat(size));
    let tone = if active { TEXT } else { FAINT };
    if icon_button(
        ui,
        around(-56.0, 36.0),
        Id::new("bar-previous"),
        icon::SKIP_BACK,
        solid(18.0),
        tone,
        "Previous",
    )
    .clicked()
    {
        out.push(Action::Previous);
    }
    if icon_button(
        ui,
        around(56.0, 36.0),
        Id::new("bar-next"),
        icon::SKIP_FORWARD,
        solid(18.0),
        tone,
        "Next",
    )
    .clicked()
    {
        out.push(Action::Next);
    }
    let shuffle = if app.saved.shuffle { ACCENT } else { MUTED };
    if icon_button(
        ui,
        around(-104.0, 34.0),
        Id::new("bar-shuffle"),
        icon::SHUFFLE,
        glyph(17.0),
        shuffle,
        "Shuffle",
    )
    .clicked()
    {
        out.push(Action::ToggleShuffle);
    }
    let (symbol, repeat, hint) = match app.saved.repeat {
        Repeat::Off => (icon::REPEAT, MUTED, "Repeat"),
        Repeat::All => (icon::REPEAT, ACCENT, "Repeat one"),
        Repeat::One => (icon::REPEAT_ONCE, ACCENT, "Stop repeating"),
    };
    if icon_button(ui, around(104.0, 34.0), Id::new("bar-repeat"), symbol, glyph(17.0), repeat, hint).clicked() {
        out.push(Action::CycleRepeat);
    }

    let mut right = bar.right() - 22.0;
    let volume = app.saved.volume;
    let track = Rect::from_min_max(pos2(right - 96.0, mid - 8.0), pos2(right, mid + 8.0));
    let slider = ui.interact(track, Id::new("bar-volume"), Sense::click_and_drag());
    let grip = ui
        .ctx()
        .animate_bool_with_time(slider.id, slider.hovered() || slider.dragged(), HOVER_TIME);
    if let (true, Some(pointer)) = (
        slider.dragged() || slider.is_pointer_button_down_on(),
        slider.interact_pointer_pos(),
    ) {
        out.push(Action::Volume((pointer.x - track.left()) / track.width()));
    }
    let wheel = ui.input(|input| input.smooth_scroll_delta.y);
    if slider.hovered() && wheel != 0.0 {
        out.push(Action::Volume(volume + wheel * 0.002));
    }
    let line = Rect::from_center_size(track.center(), vec2(track.width(), 4.0));
    let level = Rect::from_min_size(line.min, vec2(line.width() * volume, 4.0));
    ui.painter().rect_filled(line, 2.0, veil(0.14));
    ui.painter().rect_filled(level, 2.0, TEXT.lerp_to_gamma(ACCENT, grip));
    ui.painter().circle_filled(pos2(level.right(), mid), 6.0 * grip, TEXT);
    slider.widget_info(|| WidgetInfo::slider(true, volume as f64, "Volume"));
    right -= 96.0 + 22.0;

    let symbol = if volume == 0.0 {
        icon::SPEAKER_X
    } else if volume < 0.5 {
        icon::SPEAKER_LOW
    } else {
        icon::SPEAKER_HIGH
    };
    let memory = Id::new("volume-before-mute");
    if icon_button(
        ui,
        Rect::from_center_size(pos2(right, mid), Vec2::splat(34.0)),
        Id::new("bar-mute"),
        symbol,
        glyph(18.0),
        MUTED,
        "Mute",
    )
    .clicked()
    {
        let restored = ui.data(|data| data.get_temp::<f32>(memory)).unwrap_or(0.8);
        ui.data_mut(|data| data.insert_temp(memory, if volume > 0.0 { volume } else { 0.8 }));
        out.push(Action::Volume(if volume > 0.0 { 0.0 } else { restored }));
    }
    right -= 38.0;
    let queue = if app.saved.queue_open { ACCENT } else { MUTED };
    if icon_button(
        ui,
        Rect::from_center_size(pos2(right, mid), Vec2::splat(34.0)),
        Id::new("bar-queue"),
        icon::QUEUE,
        glyph(18.0),
        queue,
        "Queue",
    )
    .clicked()
    {
        out.push(Action::ToggleQueue);
    }
    right -= 34.0;
    if duration > 0 {
        let time = format!("{}  /  {}", clock(position), clock(duration));
        label(ui, pos2(right, mid), Align2::RIGHT_CENTER, &time, sans(12.5), MUTED, 140.0);
    }
    scrub
}

fn paint_scrubber(painter: &egui::Painter, scrub: &Scrub) {
    let height = 2.0 + 2.0 * scrub.hover;
    let span = |fraction: f32| Rect::from_min_size(scrub.bar.min - vec2(0.0, height / 2.0), vec2(scrub.bar.width() * fraction, height));
    painter.rect_filled(span(1.0), 0.0, rgb(0x26272C));
    painter.rect_filled(span(scrub.buffered), 0.0, rgb(0x45474E));
    painter.rect_filled(span(scrub.played), 0.0, ACCENT);
    painter.circle_filled(pos2(span(scrub.played).right(), scrub.bar.top()), 6.0 * scrub.hover, TEXT);
}

fn notice(app: &App, ctx: &egui::Context) {
    let Some((text, _)) = &app.notice else { return };
    egui::Area::new(Id::new("notice"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -(BAR_HEIGHT + 22.0)))
        .interactable(false)
        .show(ctx, |ui| {
            Frame::new()
                .fill(TEXT)
                .corner_radius(22)
                .inner_margin(Margin::symmetric(20, 12))
                .show(ui, |ui| {
                    ui.label(RichText::new(text).font(medium(13.5)).color(PANEL));
                });
        });
}
