use eframe::egui::*;
use tabs::Tabs;

mod browser;
mod tabs;

pub struct App {
    tabs: Tabs,
    debug: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        App::set_style(&cc.egui_ctx);

        Self {
            tabs: Tabs::new(),
            debug: true,
        }
    }
    fn set_style(ctx: &Context) {
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(22.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
            (
                TextStyle::Monospace,
                FontId::new(16.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Button,
                FontId::new(18.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(12.0, FontFamily::Proportional),
            ),
        ]
        .into();
        //(8.0, 3.0) default
        style.spacing.item_spacing = Vec2::new(6.0, 4.0);
        //8.0 default
        style.spacing.scroll_bar_width = 10.0;
        //(4.0, 1.0) default
        style.spacing.button_padding = Vec2::new(4.0, 2.0);

        style.visuals = Visuals::dark();

        ctx.set_style(style);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let Self { tabs, debug: _d } = self;

        // Window::new("🔧 Settings")
        //     .open(_d)
        //     .vscroll(true)
        //     .show(ctx, |ui| {
        //         ctx.style_ui(ui);
        //     });

        tabs.side_buttons(ctx);

        tabs.header(ctx);

        tabs.quick_access(ctx);

        tabs.body(ctx);
    }
}
