// Hide the console window on Windows in release builds.
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use eframe::egui::{self, Align2, Color32, ColorImage, FontId, Rect, TextureHandle, TextureOptions, Vec2};
use image::{AnimationDecoder, ImageFormat, RgbaImage};
use std::{
    cmp::Ordering,
    f32::consts::FRAC_PI_2,
    fs::File,
    io::BufReader,
    iter::Peekable,
    path::{Path, PathBuf},
    str::Chars,
    sync::mpsc::{channel, Receiver, Sender},
    time::{Duration, Instant},
};

/// File extensions that RustView tries to open (keep in sync with installer/rustview.iss).
const EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "jpe", "jfif", "png", "gif", "webp", "bmp", "ico", "tif", "tiff", "tga", "qoi",
    "pnm", "ppm", "pgm", "pbm", "pam",
];
/// Larger images are downscaled to this size for the GPU texture (displayed size stays real).
const MAX_TEXTURE_SIDE: u32 = 8192;
const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

// ───────────────────────────── loading ─────────────────────────────

struct Frame {
    image: Option<ColorImage>, // taken (moved to the GPU) on first display
    delay: Duration,
}

struct Loaded {
    path: PathBuf,
    orig: [u32; 2],
    frames: Vec<Frame>,
    file_bytes: u64,
}

struct LoadMsg {
    id: u64,
    result: Result<Loaded, String>,
}

fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn collect_frames(frames: image::Frames) -> Result<Vec<(RgbaImage, Duration)>, String> {
    let mut out = Vec::new();
    for f in frames {
        let f = f.map_err(|e| e.to_string())?;
        let (n, d) = f.delay().numer_denom_ms();
        let mut ms = if d == 0 { 100.0 } else { n as f64 / d as f64 };
        if ms <= 10.0 {
            ms = 100.0; // same behaviour as browsers for "0 delay" GIFs
        }
        out.push((f.into_buffer(), Duration::from_secs_f64(ms / 1000.0)));
    }
    if out.is_empty() {
        return Err("no frames".into());
    }
    Ok(out)
}

fn decode(path: &Path) -> Result<Vec<(RgbaImage, Duration)>, String> {
    let open = || File::open(path).map(BufReader::new).map_err(|e| e.to_string());
    // Detect the format by content, not by extension (a ".jpg" can really be a WebP).
    let reader = image::ImageReader::new(open()?)
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    match reader.format() {
        Some(ImageFormat::Gif) => {
            let dec = image::codecs::gif::GifDecoder::new(open()?).map_err(|e| e.to_string())?;
            collect_frames(dec.into_frames())
        }
        Some(ImageFormat::WebP) => {
            let dec = image::codecs::webp::WebPDecoder::new(open()?).map_err(|e| e.to_string())?;
            if dec.has_animation() {
                collect_frames(dec.into_frames())
            } else {
                let img = reader.decode().map_err(|e| e.to_string())?;
                Ok(vec![(img.to_rgba8(), Duration::ZERO)])
            }
        }
        _ => {
            let img = reader.decode().map_err(|e| e.to_string())?;
            Ok(vec![(img.to_rgba8(), Duration::ZERO)])
        }
    }
}

fn limit_size(img: RgbaImage) -> RgbaImage {
    let (w, h) = img.dimensions();
    let m = w.max(h);
    if m <= MAX_TEXTURE_SIDE {
        return img;
    }
    let s = MAX_TEXTURE_SIDE as f32 / m as f32;
    image::imageops::resize(
        &img,
        ((w as f32 * s).round() as u32).max(1),
        ((h as f32 * s).round() as u32).max(1),
        image::imageops::FilterType::Triangle,
    )
}

fn load(path: &Path) -> Result<Loaded, String> {
    let file_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let decoded = decode(path)?;
    let orig = decoded[0].0.dimensions();
    let frames = decoded
        .into_iter()
        .map(|(img, delay)| {
            let img = limit_size(img);
            let size = [img.width() as usize, img.height() as usize];
            Frame {
                image: Some(ColorImage::from_rgba_unmultiplied(size, img.as_raw())),
                delay,
            }
        })
        .collect();
    Ok(Loaded { path: path.to_path_buf(), orig: [orig.0, orig.1], frames, file_bytes })
}

// ───────────────────────────── folder listing ─────────────────────────────

fn take_num(it: &mut Peekable<Chars<'_>>) -> u128 {
    let mut n: u128 = 0;
    while let Some(&c) = it.peek() {
        match c.to_digit(10) {
            Some(d) => {
                n = n.saturating_mul(10).saturating_add(d as u128);
                it.next();
            }
            None => break,
        }
    }
    n
}

/// "img2.png" < "img10.png", case-insensitive (like Windows Explorer).
fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (a, b) = (a.to_lowercase(), b.to_lowercase());
    let (mut ai, mut bi) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (ai.peek().copied(), bi.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let (na, nb) = (take_num(&mut ai), take_num(&mut bi));
                if na != nb {
                    return na.cmp(&nb);
                }
            }
            (Some(x), Some(y)) => {
                if x != y {
                    return x.cmp(&y);
                }
                ai.next();
                bi.next();
            }
        }
    }
}

fn list_dir(dir: &Path) -> Vec<PathBuf> {
    let dir = if dir.as_os_str().is_empty() { Path::new(".") } else { dir };
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && is_supported(p))
        .collect();
    v.sort_by(|a, b| {
        natural_cmp(
            &a.file_name().unwrap_or_default().to_string_lossy(),
            &b.file_name().unwrap_or_default().to_string_lossy(),
        )
    });
    v
}

fn human_size(b: u64) -> String {
    const U: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 { format!("{b} B") } else { format!("{v:.1} {}", U[i]) }
}

// ───────────────────────────── app ─────────────────────────────

struct Current {
    loaded: Loaded,
    textures: Vec<Option<TextureHandle>>,
    frame: usize,
    playing: bool,
    frame_start: Instant,
}

#[derive(Default)]
struct Keys {
    next: bool,
    prev: bool,
    first: bool,
    last: bool,
    space: bool,
    step_fwd: bool,
    step_back: bool,
    zoom_in: bool,
    zoom_out: bool,
    fit: bool,
    actual: bool,
    fullscreen: bool,
    rot: bool,
    shift: bool,
    info: bool,
    help: bool,
    esc: bool,
}

struct Viewer {
    files: Vec<PathBuf>,
    index: usize,
    current: Option<Current>,
    error: Option<String>,
    loading: bool,
    req_id: u64,
    tx: Sender<LoadMsg>,
    rx: Receiver<LoadMsg>,
    // view state
    fit: bool,
    scale: f32,
    pan: Vec2,
    rotation: u8, // quarter turns clockwise
    key_zoom: f32,
    show_info: bool,
    show_help: bool,
    fullscreen: bool,
}

fn texture_options() -> TextureOptions {
    // smooth when shrinking, crisp pixels when zooming in
    TextureOptions { magnification: egui::TextureFilter::Nearest, ..TextureOptions::LINEAR }
}

impl Viewer {
    fn new(cc: &eframe::CreationContext<'_>, arg: Option<PathBuf>) -> Self {
        let (tx, rx) = channel();
        let mut v = Viewer {
            files: Vec::new(),
            index: 0,
            current: None,
            error: None,
            loading: false,
            req_id: 0,
            tx,
            rx,
            fit: true,
            scale: 1.0,
            pan: Vec2::ZERO,
            rotation: 0,
            key_zoom: 1.0,
            show_info: true,
            show_help: false,
            fullscreen: false,
        };
        if let Some(p) = arg {
            v.open_path(&cc.egui_ctx, p);
        }
        v
    }

    fn open_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        if path.is_dir() {
            self.files = list_dir(&path);
            self.index = 0;
        } else {
            self.files = list_dir(path.parent().unwrap_or(Path::new("")));
            match self.files.iter().position(|p| p.file_name() == path.file_name()) {
                Some(i) => self.index = i,
                None => {
                    self.files.push(path);
                    self.index = self.files.len() - 1;
                }
            }
        }
        if self.files.is_empty() {
            self.error = Some("No supported images in this folder".into());
            self.current = None;
        } else {
            self.start_load(ctx);
        }
    }

    fn start_load(&mut self, ctx: &egui::Context) {
        self.req_id += 1;
        self.loading = true;
        self.error = None;
        let (id, path, tx, ctx) = (self.req_id, self.files[self.index].clone(), self.tx.clone(), ctx.clone());
        std::thread::spawn(move || {
            let result = load(&path);
            let _ = tx.send(LoadMsg { id, result });
            ctx.request_repaint();
        });
    }

    fn go(&mut self, ctx: &egui::Context, delta: isize) {
        if self.files.is_empty() {
            return;
        }
        self.index = (self.index as isize + delta).rem_euclid(self.files.len() as isize) as usize;
        self.start_load(ctx);
    }

    fn go_to(&mut self, ctx: &egui::Context, index: usize) {
        if index < self.files.len() && index != self.index {
            self.index = index;
            self.start_load(ctx);
        }
    }

    fn poll_loader(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            if msg.id != self.req_id {
                continue; // stale result (user already moved on)
            }
            self.loading = false;
            match msg.result {
                Ok(loaded) => {
                    let n = loaded.frames.len();
                    let name = loaded.path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                        "{name} ({}/{}) - RustView",
                        self.index + 1,
                        self.files.len()
                    )));
                    self.current = Some(Current {
                        loaded,
                        textures: vec![None; n],
                        frame: 0,
                        playing: n > 1,
                        frame_start: Instant::now(),
                    });
                    self.fit = true;
                    self.pan = Vec2::ZERO;
                    self.rotation = 0;
                }
                Err(e) => {
                    let name = self.files[self.index].file_name().unwrap_or_default().to_string_lossy().to_string();
                    self.error = Some(format!("Can't open {name}\n{e}"));
                    self.current = None;
                }
            }
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        use egui::Key;
        let k = ctx.input(|i| Keys {
            next: i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::PageDown),
            prev: i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::PageUp),
            first: i.key_pressed(Key::Home),
            last: i.key_pressed(Key::End),
            space: i.key_pressed(Key::Space),
            step_fwd: i.key_pressed(Key::Period),
            step_back: i.key_pressed(Key::Comma),
            zoom_in: i.key_pressed(Key::ArrowUp) || i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals),
            zoom_out: i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::Minus),
            fit: i.key_pressed(Key::Num0),
            actual: i.key_pressed(Key::Num1),
            fullscreen: i.key_pressed(Key::F) || i.key_pressed(Key::F11) || i.key_pressed(Key::Enter),
            rot: i.key_pressed(Key::R),
            shift: i.modifiers.shift,
            info: i.key_pressed(Key::I),
            help: i.key_pressed(Key::H) || i.key_pressed(Key::F1),
            esc: i.key_pressed(Key::Escape),
        });

        if k.next {
            self.go(ctx, 1);
        }
        if k.prev {
            self.go(ctx, -1);
        }
        if k.first {
            self.go_to(ctx, 0);
        }
        if k.last {
            self.go_to(ctx, self.files.len().saturating_sub(1));
        }

        let animated = self.current.as_ref().map(|c| c.loaded.frames.len() > 1).unwrap_or(false);
        if k.space {
            if animated {
                if let Some(c) = self.current.as_mut() {
                    c.playing = !c.playing;
                    c.frame_start = Instant::now();
                }
            } else {
                self.go(ctx, 1);
            }
        }
        if animated && (k.step_fwd || k.step_back) {
            if let Some(c) = self.current.as_mut() {
                let n = c.loaded.frames.len();
                c.playing = false;
                c.frame = if k.step_fwd { (c.frame + 1) % n } else { (c.frame + n - 1) % n };
            }
        }

        if k.zoom_in {
            self.key_zoom *= 1.25;
        }
        if k.zoom_out {
            self.key_zoom /= 1.25;
        }
        if k.fit {
            self.fit = true;
        }
        if k.actual {
            self.fit = false;
            self.scale = 1.0;
            self.pan = Vec2::ZERO;
        }
        if k.rot {
            self.rotation = if k.shift { (self.rotation + 3) % 4 } else { (self.rotation + 1) % 4 };
        }
        if k.info {
            self.show_info = !self.show_info;
        }
        if k.help {
            self.show_help = !self.show_help;
        }
        if k.fullscreen {
            self.fullscreen = !self.fullscreen;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
        }
        if k.esc {
            if self.show_help {
                self.show_help = false;
            } else if self.fullscreen {
                self.fullscreen = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    fn tick_animation(&mut self, ctx: &egui::Context) {
        let Some(c) = self.current.as_mut() else { return };
        let n = c.loaded.frames.len();
        if !c.playing || n < 2 {
            return;
        }
        let now = Instant::now();
        let mut delay = c.loaded.frames[c.frame].delay;
        let mut guard = 0;
        while now.duration_since(c.frame_start) >= delay && guard < 8 {
            c.frame_start += delay;
            c.frame = (c.frame + 1) % n;
            delay = c.loaded.frames[c.frame].delay;
            guard += 1;
        }
        if guard == 8 {
            c.frame_start = now; // we're lagging behind, don't try to catch up
        }
        ctx.request_repaint_after(delay.saturating_sub(now.duration_since(c.frame_start)));
    }

    fn zoom_by(&mut self, factor: f32, anchor: Vec2) {
        // `scale` already holds the fit scale while in fit mode
        self.fit = false;
        let new = (self.scale * factor).clamp(0.01, 64.0);
        let f = new / self.scale;
        self.pan = anchor - (anchor - self.pan) * f;
        self.scale = new;
    }

    fn draw(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_gray(24)))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                let painter = ui.painter().clone();

                if self.current.is_some() {
                    self.draw_image(ctx, ui, rect, &resp);
                } else if let Some(err) = &self.error {
                    painter.text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        err,
                        FontId::proportional(16.0),
                        Color32::from_rgb(230, 130, 130),
                    );
                } else if !self.loading {
                    painter.text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        "Drop an image here\nor open one with RustView\n\nH - shortcuts",
                        FontId::proportional(18.0),
                        Color32::from_gray(150),
                    );
                }
                if self.loading {
                    painter.text(
                        rect.left_top() + Vec2::new(10.0, 8.0),
                        Align2::LEFT_TOP,
                        "Loading...",
                        FontId::proportional(13.0),
                        Color32::from_gray(200),
                    );
                }
            });

        if self.show_help {
            egui::Window::new("Shortcuts")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    egui::Grid::new("help").num_columns(2).spacing([24.0, 4.0]).show(ui, |ui| {
                        for (k, d) in [
                            ("Left / Right, PgUp / PgDn", "previous / next image"),
                            ("Home / End", "first / last image"),
                            ("Mouse wheel, Up / Down, + / -", "zoom"),
                            ("Drag, double click", "pan, 100% <-> fit"),
                            ("0 / 1", "fit to window / actual size"),
                            ("R / Shift+R", "rotate (view only)"),
                            ("Space", "play / pause GIF & WebP (else: next)"),
                            (", / .", "previous / next animation frame"),
                            ("F, F11, Enter", "fullscreen"),
                            ("I", "toggle info bar"),
                            ("H, F1", "this window"),
                            ("Esc", "exit fullscreen / close"),
                        ] {
                            ui.monospace(k);
                            ui.label(d);
                            ui.end_row();
                        }
                    });
                });
        }
    }

    fn draw_image(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, rect: Rect, resp: &egui::Response) {
        let [ow, oh] = self.current.as_ref().unwrap().loaded.orig;
        let (w, h) = (ow as f32, oh as f32);
        let (dw, dh) = if self.rotation % 2 == 1 { (h, w) } else { (w, h) };

        let fit_scale = (rect.width() / dw).min(rect.height() / dh).min(1.0).max(0.001);
        if self.fit {
            self.scale = fit_scale;
            self.pan = Vec2::ZERO;
        }

        // zoom: keyboard (around the center) and wheel / pinch (around the cursor)
        let kz = std::mem::replace(&mut self.key_zoom, 1.0);
        if kz != 1.0 {
            self.zoom_by(kz, Vec2::ZERO);
        }
        let (scroll, zoom_delta, pointer) =
            ctx.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta(), i.pointer.hover_pos()));
        let factor = (scroll * 0.002).exp() * zoom_delta;
        if (factor - 1.0).abs() > 1e-4 {
            if let Some(p) = pointer.filter(|p| rect.contains(*p)) {
                self.zoom_by(factor, p - rect.center());
            }
        }

        if resp.double_clicked() {
            if self.fit {
                self.fit = false;
                self.scale = 1.0;
                self.pan = Vec2::ZERO;
            } else {
                self.fit = true;
            }
        }
        if !self.fit && resp.dragged() {
            self.pan += resp.drag_delta();
        }
        if resp.dragged() {
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        } else if !self.fit && resp.hovered() {
            ctx.set_cursor_icon(egui::CursorIcon::Grab);
        }

        // keep at least a bit of the image on screen
        let lim = ((Vec2::new(dw, dh) * self.scale + rect.size()) * 0.5 - Vec2::splat(48.0)).max(Vec2::ZERO);
        self.pan = self.pan.clamp(-lim, lim);

        // texture of the current frame (uploaded lazily)
        let scale = self.scale;
        let center = rect.center() + self.pan;
        let angle = self.rotation as f32 * FRAC_PI_2;
        let cur = self.current.as_mut().unwrap();
        let idx = cur.frame;
        if cur.textures[idx].is_none() {
            if let Some(img) = cur.loaded.frames[idx].image.take() {
                cur.textures[idx] = Some(ctx.load_texture(format!("frame{idx}"), img, texture_options()));
            }
        }
        if let Some(tex) = &cur.textures[idx] {
            let size = Vec2::new(w, h) * scale;
            egui::Image::from_texture(egui::load::SizedTexture::new(tex.id(), size))
                .rotate(angle, Vec2::splat(0.5))
                .paint_at(ui, Rect::from_center_size(center, size));
        }

        // info bar
        if self.show_info {
            let cur = self.current.as_ref().unwrap();
            let name = cur.loaded.path.file_name().unwrap_or_default().to_string_lossy();
            let mut right = format!(
                "{}/{}   {}x{}   {}   {:.0}%",
                self.index + 1,
                self.files.len(),
                ow,
                oh,
                human_size(cur.loaded.file_bytes),
                self.scale * 100.0
            );
            let n = cur.loaded.frames.len();
            if n > 1 {
                right += &format!("   frame {}/{} [{}]", cur.frame + 1, n, if cur.playing { "playing" } else { "paused" });
            }
            let bar = Rect::from_min_max(egui::pos2(rect.left(), rect.bottom() - 26.0), rect.right_bottom());
            let p = ui.painter();
            p.rect_filled(bar, 0.0, Color32::from_black_alpha(170));
            let font = FontId::proportional(13.0);
            p.text(bar.left_center() + Vec2::new(10.0, 0.0), Align2::LEFT_CENTER, name, font.clone(), Color32::from_gray(230));
            p.text(bar.right_center() - Vec2::new(10.0, 0.0), Align2::RIGHT_CENTER, right, font, Color32::from_gray(190));
        }
    }
}

impl eframe::App for Viewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_loader(ctx);
        if let Some(p) = ctx.input(|i| i.raw.dropped_files.first().and_then(|f| f.path.clone())) {
            self.open_path(ctx, p);
        }
        self.handle_keys(ctx);
        self.tick_animation(ctx);
        self.draw(ctx);
    }
}

fn window_icon() -> egui::IconData {
    match image::load_from_memory(ICON_PNG) {
        Ok(img) => {
            let img = img.resize(128, 128, image::imageops::FilterType::Triangle).to_rgba8();
            egui::IconData { width: img.width(), height: img.height(), rgba: img.into_raw() }
        }
        Err(_) => egui::IconData::default(),
    }
}

fn main() -> eframe::Result {
    let arg = std::env::args_os().nth(1).map(PathBuf::from);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([320.0, 240.0])
            .with_icon(window_icon())
            .with_app_id("rustview")
            .with_title("RustView")
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native("RustView", options, Box::new(move |cc| Ok(Box::new(Viewer::new(cc, arg)))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn natural_sorting() {
        let mut v = vec!["img10.png", "IMG2.png", "img1.png", "a.png"];
        v.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(v, vec!["a.png", "img1.png", "IMG2.png", "img10.png"]);
    }

    #[test]
    fn supported_extensions() {
        assert!(is_supported(Path::new("x.WEBP")));
        assert!(is_supported(Path::new("x.gif")));
        assert!(!is_supported(Path::new("x.txt")));
    }
}
