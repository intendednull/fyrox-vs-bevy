use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    //winit::WinitSettings,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};


pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        //app.insert_resource(WinitSettings::desktop_app());
        //app.add_startup_system(setup_game_ui);
        app.add_plugin(EguiPlugin);
        app.add_system(simple_gameplay_ui);
    }
}

fn simple_gameplay_ui(mut egui_context: ResMut<EguiContext>) {
    egui::Window::new("Hello").show(egui_context.ctx_mut(), |ui|{
        ui.label("World!");
    });
}