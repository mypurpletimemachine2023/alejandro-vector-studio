//! Canvas-first runtime UI, deliberately separate from serialized desktop workspaces.
use crate::{
    VectorcraftApp,
    compact_policy::{self, Preference},
};
use egui::{Context, Id, Ui, vec2};
use serde_json::json;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Drawer {
    New,
    Tools,
    Layers,
    Properties,
    More,
}
#[derive(Default)]
pub(crate) struct Runtime {
    pub active: bool,
    pub navigate: bool,
    preference: Preference,
    initialized: bool,
    changed: bool,
    drawer: Option<Drawer>,
    viewport: [f32; 4],
    controls: Vec<(&'static str, [f32; 4])>,
    new_size: [u32; 2],
}
impl VectorcraftApp {
    pub fn compact_layout_needs_init(&self) -> bool {
        !self.compact.initialized
    }
    pub fn initialize_compact_layout(&mut self, stored: Option<&str>) {
        if !self.compact.initialized {
            self.compact.preference = Preference::parse(stored.unwrap_or("auto"));
            self.compact.initialized = true;
        }
    }
    pub fn take_compact_layout_change(&mut self) -> Option<&'static str> {
        std::mem::take(&mut self.compact.changed).then(|| self.compact.preference.id())
    }
    pub fn compact_layout_state(&self) -> String {
        json!({"version":1,"applied":if self.compact.active {"compact"} else {"desktop"},"preferred":self.compact.preference.id(),
            "pane":self.compact.drawer.map_or("canvas",|p|match p {Drawer::New=>"new",Drawer::Tools=>"tools",Drawer::Layers=>"layers",Drawer::Properties=>"properties",Drawer::More=>"more"}),
            "interaction":if self.compact.navigate {"navigate"} else {"edit"},"viewport":self.compact.viewport,
            "controls":self.compact.controls.iter().map(|(id,rect)|json!({"id":id,"rect":rect})).collect::<Vec<_>>()}).to_string()
    }
}
pub(crate) fn prepare(app: &mut VectorcraftApp, ctx: &Context) {
    let r = ctx.content_rect();
    let held = ctx.input(|i| i.pointer.any_down() || i.multi_touch().is_some()) || crate::canvas::dragging(ctx);
    app.compact.active = compact_policy::resolve(app.compact.preference, app.compact.active, r.width(), r.height(), held);
    app.compact.viewport = [r.left(), r.top(), r.width(), r.height()];
    app.compact.controls.clear();
}
pub(crate) fn navigating(app: &VectorcraftApp) -> bool {
    app.compact.active && app.compact.navigate
}
fn command(app: &mut VectorcraftApp, id: &str) {
    crate::menus::invoke(app, id, json!({}));
}
fn touch_style(ui: &mut Ui) {
    ui.spacing_mut().interact_size = vec2(44.0, 44.0);
    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
}
fn button(app: &mut VectorcraftApp, ui: &mut Ui, id: &'static str, label: &str, selected: bool) -> bool {
    let r = ui.add(egui::Button::new(label).selected(selected).min_size(vec2(44.0, 44.0)));
    app.compact.controls.push((id, [r.rect.left(), r.rect.top(), r.rect.width(), r.rect.height()]));
    r.clicked()
}
pub(crate) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    let t = crate::theme::Tokens::get(&ctx);
    egui::Panel::top("aca-vector-compact-bar").frame(egui::Frame::NONE.fill(t.panel).inner_margin(4)).show(ui, |ui| {
        touch_style(ui);
        ui.horizontal(|ui| {
            // Use the same native MenuButton/popup path as ui.menu_button, with an explicit
            // phone target instead of its desktop-width text button.
            let (file, _) = egui::containers::menu::MenuButton::from_button(egui::Button::new("File").min_size(vec2(44.0, 44.0))).ui(ui, |ui| {
                touch_style(ui);
                if ui.button("New drawing…").clicked() {
                    app.compact.new_size = [512, 512];
                    app.compact.drawer = Some(Drawer::New);
                    ui.close();
                }
                for (label, id) in [("Open…", "file.open"), ("Save project", "file.save"), ("Save as…", "file.saveAs"), ("Export…", "file.exportAs")]
                {
                    if ui.button(label).clicked() {
                        command(app, id);
                        ui.close();
                    }
                }
            });
            app.compact.controls.push(("file", [file.rect.left(), file.rect.top(), file.rect.width(), file.rect.height()]));
            if button(app, ui, "tools", "Tools", false) {
                app.compact.drawer = Some(Drawer::Tools);
            }
            if button(app, ui, "undo", "Undo", false) {
                command(app, "edit.undo");
            }
            if button(app, ui, "redo", "Redo", false) {
                command(app, "edit.redo");
            }
            if button(app, ui, "more", "More", false) {
                app.compact.drawer = Some(Drawer::More);
            }
        });
        ui.horizontal(|ui| {
            if button(app, ui, "edit", "Edit", !app.compact.navigate) {
                app.compact.navigate = false;
            }
            if button(app, ui, "navigate", "Navigate", app.compact.navigate) {
                app.compact.navigate = true;
            }
            if button(app, ui, "layers", "Layers", app.compact.drawer == Some(Drawer::Layers)) {
                app.compact.drawer = Some(Drawer::Layers);
            }
            if button(app, ui, "properties", "Properties", app.compact.drawer == Some(Drawer::Properties)) {
                app.compact.drawer = Some(Drawer::Properties);
            }
        });
    });
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.pasteboard)).show(ui, |ui| crate::canvas::show(app, ui));
    show_drawer(app, &ctx);
}
fn show_drawer(app: &mut VectorcraftApp, ctx: &Context) {
    let Some(pane) = app.compact.drawer else { return };
    let r = ctx.content_rect();
    let modal = egui::Modal::new(Id::new("aca-vector-compact-drawer")).show(ctx, |ui| {
        touch_style(ui);
        ui.set_width((r.width() - 32.0).clamp(200.0, 440.0));
        ui.horizontal(|ui| {
            if ui.button("Back to canvas").clicked() {
                app.compact.drawer = None;
            }
            ui.label(match pane {
                Drawer::New => "New drawing",
                Drawer::Tools => "Tools",
                Drawer::Layers => "Layers",
                Drawer::Properties => "Properties",
                Drawer::More => "More",
            });
        });
        egui::ScrollArea::vertical().max_height((r.height() - 100.0).max(80.0)).auto_shrink([false, false]).show(ui, |ui| match pane {
            Drawer::New => {
                for (label, value) in ["Width (pt)", "Height (pt)"].into_iter().zip(app.compact.new_size.iter_mut()) {
                    ui.horizontal(|ui| {
                        ui.label(label);
                        ui.add_sized([110.0, 44.0], egui::DragValue::new(value).range(1..=4096));
                    });
                }
                if ui.button("Create drawing").clicked() {
                    let [width, height] = app.compact.new_size;
                    if app.run("file.new", json!({"width":width.clamp(1,4096),"height":height.clamp(1,4096)})).is_ok() {
                        app.compact.drawer = None;
                    }
                }
            }
            Drawer::Tools => {
                for group in vectorcraft_tools::TOOL_GROUPS {
                    for tool in *group {
                        if ui
                            .add(egui::Button::new(tool.label).selected(app.session.tool_id() == tool.id).min_size(vec2(ui.available_width(), 44.0)))
                            .clicked()
                        {
                            app.select_tool(tool.id);
                            app.compact.navigate = false;
                            app.compact.drawer = None;
                        }
                    }
                }
            }
            Drawer::Layers => crate::panels::layers::show(app, ui),
            Drawer::Properties => crate::panels::properties::show(app, ui),
            Drawer::More => {
                ui.label("Layout (saved separately from artwork)");
                for p in Preference::ALL {
                    if ui.selectable_label(app.compact.preference == p, p.label()).clicked() {
                        app.compact.preference = p;
                        app.compact.changed = true;
                        app.compact.drawer = None;
                    }
                }
                if ui.button("Fit artboard").clicked() {
                    command(app, "view.fitArtboard");
                    app.compact.drawer = None;
                }
                if ui.button("App tutorial").clicked() {
                    command(app, "help.discord");
                    app.compact.drawer = None;
                }
            }
        });
    });
    if modal.should_close() {
        app.compact.drawer = None;
    }
}
pub(crate) fn desktop_controls(app: &mut VectorcraftApp, ctx: &Context) {
    let r = ctx.content_rect();
    egui::Area::new(Id::new("aca-vector-layout-control"))
        .fixed_pos(egui::pos2((r.right() - 92.0).max(r.left()), (r.bottom() - 56.0).max(r.top())))
        .show(ctx, |ui| {
            if button(app, ui, "layout", "Layout", false) {
                app.compact.drawer = Some(Drawer::More);
            }
        });
    show_drawer(app, ctx);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aca_mobile_choice_and_drawers_do_not_change_serialized_workspace_or_documents() {
        let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), crate::Services::default());
        let before = serde_json::to_value(&app.ui).unwrap();
        app.initialize_compact_layout(Some("compact"));
        app.compact.drawer = Some(Drawer::Properties);
        app.compact.navigate = true;
        assert_eq!(serde_json::to_value(&app.ui).unwrap(), before);
        assert!(app.session.documents().is_empty());
        assert_eq!(app.take_compact_layout_change(), None);
    }
    #[test]
    fn aca_mobile_resizing_only_changes_runtime_layout_not_desktop_dock() {
        let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), crate::Services::default());
        let before = serde_json::to_value(&app.ui).unwrap();
        let ctx = Context::default();
        ctx.begin_pass(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(390.0, 747.0))), ..Default::default() });
        prepare(&mut app, &ctx);
        assert!(app.compact.active);
        let mut output = ctx.end_pass();
        output.textures_delta.clear();
        assert_eq!(serde_json::to_value(&app.ui).unwrap(), before);
    }
    #[test]
    fn aca_mobile_actual_toolbar_stays_44pt_in_portrait_and_landscape() {
        for (w, h) in [(390.0, 747.0), (844.0, 293.0)] {
            let ctx = Context::default();
            let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), crate::Services::default());
            crate::theme::install_fonts(&ctx);
            app.run("file.new", json!({"width":32,"height":32})).unwrap();
            let raw = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(w, h))), ..Default::default() };
            let mut output = ctx.run_ui(raw, |ui| {
                prepare(&mut app, &ctx);
                show(&mut app, ui);
            });
            output.textures_delta.clear();
            assert!(app.compact.active);
            assert!(app.compact.controls.iter().any(|(id, _)| *id == "file"), "File's Open/Save menu must be included in actual target measurements");
            for (id, [x, y, width, height]) in &app.compact.controls {
                assert!(*width >= 44.0 && *height >= 44.0, "{id}: {width}x{height}");
                assert!(*x >= 0.0 && *y >= 0.0 && x + width <= w + 1.0 && y + height <= h + 1.0, "{id} outside {w}x{h}");
            }
            let r = app.canvas_rect.unwrap();
            assert!(r.width() > 300.0 && r.height() > 120.0, "{r:?}");
        }
    }
}
