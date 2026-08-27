use anyhow::{Context, Result};
use configparser::ini::Ini;
use std::num::NonZero;
use vello::{AaConfig, AaSupport, RendererOptions, wgpu::PresentMode};
use winit::window::WindowLevel;

macro_rules! parse_string {
    ($ini:expr, $section:expr, $key:expr, $target:expr) => {
        if let Some(v) = $ini.get($section, $key) {
            $target = v;
        }
    };
}

macro_rules! parse_number {
    ($ini:expr, $section:expr, $key:expr, $target:expr) => {
        $target = $ini
            .get($section, $key)
            .and_then(|v| v.parse().ok())
            .unwrap_or($target);
    };
}

macro_rules! parse_bool {
    ($ini:expr, $section:expr, $key:expr, $target:expr) => {
        if let Ok(Some(v)) = $ini.getboolcoerce($section, $key) {
            $target = v;
        }
    };
}

#[derive(Clone, Debug)]
pub struct Render {
    present_mode: String,
    use_cpu: bool,
    antialiasing_support: String,
    num_init_threads: u32,
    antialiasing_method: String,
}

impl Default for Render {
    fn default() -> Self {
        Self {
            present_mode: "AutoVsync".into(),
            use_cpu: false,
            antialiasing_support: "All".into(),
            num_init_threads: 0,
            antialiasing_method: "Auto".into(),
        }
    }
}

impl Render {
    pub fn present_mode(&self) -> PresentMode {
        match self.present_mode.to_lowercase().as_str().trim() {
            "autovsync" | "auto_vsync" | "auto-vsync" | "0" => PresentMode::AutoVsync,
            "autonovsync" | "auto_novsync" | "auto-novsync" | "1" => PresentMode::AutoNoVsync,
            "fifo" | "2" => PresentMode::Fifo,
            "fiforelaxed" | "fifo_relaxed" | "fifo-relaxed" | "3" => PresentMode::FifoRelaxed,
            "immediate" | "4" => PresentMode::Immediate,
            "mailbox" | "5" => PresentMode::Mailbox,
            _ => PresentMode::AutoVsync,
        }
    }

    pub fn renderer_options(&self) -> RendererOptions {
        let mut ro = RendererOptions {
            use_cpu: self.use_cpu,
            ..Default::default()
        };

        if !self.antialiasing_support.is_empty() {
            let mut aa_support = AaSupport {
                area: false,
                msaa8: false,
                msaa16: false,
            };
            for part in self
                .antialiasing_support
                .split(|c| c == ',' || c == '+' || c == ' ')
            {
                match part.trim().to_lowercase().as_str() {
                    "all" | "everything" | "full" => aa_support = AaSupport::all(),
                    "area" | "0" => aa_support.area = true,
                    "msaa8" | "8" => aa_support.msaa8 = true,
                    "msaa16" | "16" => aa_support.msaa16 = true,
                    _ => {}
                }
            }
            ro.antialiasing_support = aa_support;
        }

        if self.num_init_threads > 0 {
            ro.num_init_threads = Some(NonZero::new(self.num_init_threads as usize).unwrap());
        }

        ro
    }

    pub fn antialiasing_method(&self) -> AaConfig {
        match self.antialiasing_method.to_lowercase().as_str().trim() {
            "area" | "0" => AaConfig::Area,
            "msaa8" | "8" => AaConfig::Msaa8,
            "msaa16" | "16" => AaConfig::Msaa16,
            _ => AaConfig::Area,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Window {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
    pub position: Option<(u32, u32)>,
    pub fullscreen: bool,
    pub resizable: bool,
    pub active: bool,
    pub visible: bool,
    pub level: String,
    pub decorations: bool,
    pub transparent: bool,
    pub blur: bool,
    pub maximized: bool,
    pub min_width: u32,
    pub min_height: u32,
    pub max_width: u32,
    pub max_height: u32,

    // macOS
    pub macos_titlebar_transparent: bool,
    pub macos_titlebar_hidden: bool,
    pub macos_titlebar_buttons_hidden: bool,
    pub macos_title_hidden: bool,
    pub macos_fullsize_content_view: bool,
    pub macos_has_shadow: bool,
    pub macos_option_as_alt: String,
    pub macos_tabbing_identifier: String,
    pub macos_borderless_game: bool,
    pub macos_disallow_hidpi: bool,
    // // Windows
    // pub windows_skip_taskbar: bool,
    // pub windows_undecorated_shadow: bool,
    // pub windows_corner_preference: Option<String>,
    // pub windows_class_name: Option<String>,
    // pub windows_border_color: Option<String>,
    // pub windows_title_background_color: Option<String>,
    // pub windows_title_text_color: Option<String>,

    // // X11
    // pub x11_general_name: Option<String>,
    // pub x11_instance_name: Option<String>,
    // pub x11_override_redirect: bool,
    // pub x11_window_type: Option<String>,
    // pub x11_visual_id: Option<u32>,
    // pub x11_screen_id: Option<i32>,
    // pub x11_embed_parent_window: Option<u32>,

    // // Wayland
    // pub wayland_general_name: Option<String>,
    // pub wayland_instance_name: Option<String>,
    // pub wayland_activation_token: Option<String>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            title: "void".into(),
            width: 800,
            height: 600,
            x: 0,
            y: 0,
            position: None,
            fullscreen: false,
            resizable: true,
            active: true,
            visible: true,
            level: "normal".into(),
            decorations: true,
            transparent: false,
            blur: false,
            maximized: false,
            min_width: 0,
            min_height: 0,
            max_width: 0,
            max_height: 0,

            // macOS
            macos_titlebar_transparent: false,
            macos_titlebar_hidden: false,
            macos_titlebar_buttons_hidden: false,
            macos_title_hidden: false,
            macos_fullsize_content_view: false,
            macos_has_shadow: true,
            macos_option_as_alt: "".into(),
            macos_tabbing_identifier: "".into(),
            macos_borderless_game: false,
            macos_disallow_hidpi: false,
            // // Windows
            // windows_skip_taskbar: false,
            // windows_undecorated_shadow: true,
            // windows_corner_preference: None,
            // windows_class_name: None,
            // windows_border_color: None,
            // windows_title_background_color: None,
            // windows_title_text_color: None,

            // // X11
            // x11_general_name: None,
            // x11_instance_name: None,
            // x11_override_redirect: false,
            // x11_window_type: None,
            // x11_visual_id: None,
            // x11_screen_id: None,
            // x11_embed_parent_window: None,

            // // Wayland
            // wayland_general_name: None,
            // wayland_instance_name: None,
            // wayland_activation_token: None,
        }
    }
}

impl Window {
    pub fn attributes(&self) -> winit::window::WindowAttributes {
        let w = &self;

        let mut attr = winit::window::Window::default_attributes()
            .with_title(&w.title)
            .with_inner_size(winit::dpi::LogicalSize::new(w.width, w.height))
            .with_resizable(w.resizable)
            .with_visible(w.visible)
            .with_decorations(w.decorations)
            .with_transparent(w.transparent)
            .with_blur(w.blur)
            .with_maximized(w.maximized)
            .with_fullscreen(if w.fullscreen {
                Some(winit::window::Fullscreen::Borderless(None))
            } else {
                None
            });

        if w.x > 0 || w.y > 0 {
            attr = attr.with_position(winit::dpi::LogicalPosition::new(w.x, w.y));
        }

        if let Some(p) = w.position {
            attr = attr.with_position(winit::dpi::LogicalPosition::new(p.0, p.1));
        }

        let mut min_inner_size = winit::dpi::LogicalSize::new(1, 1);
        if w.min_width > 0 {
            let min_width = w.min_width.max(1);
            min_inner_size.width = min_width;
        }
        if w.min_height > 0 {
            let min_height = w.min_height.max(1);
            min_inner_size.height = min_height;
        }
        attr = attr.with_min_inner_size(min_inner_size);

        if w.max_width > 0 || w.max_height > 0 {
            let mut max_inner_size = winit::dpi::LogicalSize::new(u32::MAX, u32::MAX);
            if w.max_width > 0 {
                max_inner_size.width = w.max_width.max(2);
            }
            if w.max_height > 0 {
                max_inner_size.height = w.max_height.max(2);
            }
            attr = attr.with_max_inner_size(max_inner_size);
        }

        if !w.level.is_empty() {
            match w.level.to_lowercase().trim() {
                "normal" => attr = attr.with_window_level(WindowLevel::Normal),
                "alwaysonbottom" => attr = attr.with_window_level(WindowLevel::AlwaysOnBottom),
                "alwaysontop" => attr = attr.with_window_level(WindowLevel::AlwaysOnTop),
                _ => {}
            }
        }

        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::WindowAttributesExtMacOS;

            attr = attr
                .with_titlebar_transparent(w.macos_titlebar_transparent)
                .with_titlebar_hidden(w.macos_titlebar_hidden)
                .with_titlebar_buttons_hidden(w.macos_titlebar_buttons_hidden)
                .with_title_hidden(w.macos_title_hidden)
                .with_fullsize_content_view(w.macos_fullsize_content_view)
                .with_has_shadow(w.macos_has_shadow)
                .with_borderless_game(w.macos_borderless_game)
                .with_disallow_hidpi(w.macos_disallow_hidpi);

            if !w.macos_option_as_alt.is_empty() {
                let option = match w.macos_option_as_alt.to_lowercase().as_str() {
                    "none" => winit::platform::macos::OptionAsAlt::None,
                    "only_left" => winit::platform::macos::OptionAsAlt::OnlyLeft,
                    "only_right" => winit::platform::macos::OptionAsAlt::OnlyRight,
                    "both" => winit::platform::macos::OptionAsAlt::Both,
                    _ => winit::platform::macos::OptionAsAlt::None,
                };
                attr = attr.with_option_as_alt(option);
            }

            if !w.macos_tabbing_identifier.is_empty() {
                attr = attr.with_tabbing_identifier(&w.macos_tabbing_identifier);
            }

            // let behavior = winit::platform::macos::NSWindowCollectionBehavior::CanJoinAllSpaces
            //     | winit::platform::macos::NSWindowCollectionBehavior::CanJoinAllApplications
            //     | winit::platform::macos::NSWindowCollectionBehavior::FullScreenAuxiliary;
            //
            // attr = attr.with_collection_behavior(behavior.0);
        }

        attr
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub path: Option<std::path::PathBuf>,
    pub render: Render,
    pub window: Window,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: None,
            render: Render::default(),
            window: Window::default(),
        }
    }
}

impl Config {
    pub fn from_ini_file(
        path: impl AsRef<std::path::Path>,
        default: Option<Config>,
    ) -> Result<Self> {
        let mut ini = Ini::new();
        ini.load(&path).map_err(|e| anyhow::anyhow!("{e}"))?;

        let mut config = default.unwrap_or_default();
        config.path.replace(path.as_ref().to_path_buf());

        parse_string!(ini, "render", "present_mode", config.render.present_mode);
        parse_bool!(ini, "render", "use_cpu", config.render.use_cpu);
        parse_string!(
            ini,
            "render",
            "antialiasing_support",
            config.render.antialiasing_support
        );
        parse_number!(
            ini,
            "render",
            "num_init_threads",
            config.render.num_init_threads
        );
        parse_string!(
            ini,
            "render",
            "antialiasing_method",
            config.render.antialiasing_method
        );

        parse_string!(ini, "window", "title", config.window.title);
        parse_number!(ini, "window", "width", config.window.width);
        parse_number!(ini, "window", "height", config.window.height);
        parse_bool!(ini, "window", "fullscreen", config.window.fullscreen);
        parse_bool!(ini, "window", "resizable", config.window.resizable);
        parse_bool!(ini, "window", "active", config.window.active);
        parse_bool!(ini, "window", "visible", config.window.visible);
        parse_bool!(ini, "window", "decorations", config.window.decorations);
        parse_bool!(ini, "window", "transparent", config.window.transparent);
        parse_bool!(ini, "window", "blur", config.window.blur);
        parse_bool!(ini, "window", "maximized", config.window.maximized);
        parse_number!(ini, "window", "min_width", config.window.min_width);
        parse_number!(ini, "window", "min_height", config.window.min_height);
        parse_number!(ini, "window", "max_width", config.window.max_width);
        parse_number!(ini, "window", "max_height", config.window.max_height);

        parse_bool!(
            ini,
            "window",
            "macos_titlebar_transparent",
            config.window.macos_titlebar_transparent
        );
        parse_bool!(
            ini,
            "window",
            "macos_titlebar_hidden",
            config.window.macos_titlebar_hidden
        );
        parse_bool!(
            ini,
            "window",
            "macos_titlebar_buttons_hidden",
            config.window.macos_titlebar_buttons_hidden
        );
        parse_bool!(
            ini,
            "window",
            "macos_title_hidden",
            config.window.macos_title_hidden
        );
        parse_bool!(
            ini,
            "window",
            "macos_fullsize_content_view",
            config.window.macos_fullsize_content_view
        );
        parse_bool!(
            ini,
            "window",
            "macos_has_shadow",
            config.window.macos_has_shadow
        );
        parse_bool!(
            ini,
            "window",
            "macos_borderless_game",
            config.window.macos_borderless_game
        );
        parse_bool!(
            ini,
            "window",
            "macos_disallow_hidpi",
            config.window.macos_disallow_hidpi
        );
        parse_string!(
            ini,
            "window",
            "macos_option_as_alt",
            config.window.macos_option_as_alt
        );
        parse_string!(
            ini,
            "window",
            "macos_tabbing_identifier",
            config.window.macos_tabbing_identifier
        );

        if let Some(pos) = ini.get("window", "position") {
            if pos.to_lowercase() != "none" && !pos.is_empty() {
                let parts: Vec<&str> = pos.split(',').collect();
                anyhow::ensure!(
                    parts.len() == 2,
                    "Invalid position '{}'. Expected 'x,y'",
                    pos
                );
                config.window.position = Some((
                    parts[0]
                        .trim()
                        .parse()
                        .context(format!("Invalid x in '{}'", pos))?,
                    parts[1]
                        .trim()
                        .parse()
                        .context(format!("Invalid y in '{}'", pos))?,
                ));
            }
        }

        Ok(config)
    }
}
