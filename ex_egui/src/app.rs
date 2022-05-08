use eframe::egui::*;
use tabs::Tabs;

mod browser;
mod tabs;

pub struct App {
    tabs: Tabs,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(Visuals::dark());

        Self { tabs: Tabs::new() }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let Self { tabs } = self;

        tabs.side_buttons(ctx);

        tabs.header(ctx);

        tabs.quick_access(ctx);

        tabs.body(ctx);

        //TODO: footer
        // TopBottomPanel::bottom("footer").show(ctx, |ui| {
        //     ui.label("Hello World!");
        // });
    }
}
