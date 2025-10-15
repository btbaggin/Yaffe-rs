use crate::input::Actions;
use crate::job_system::JobResult;
use crate::logger::LogEntry;
use crate::overlay_state::OverlayState;
use crate::ui::{DeferredAction, WidgetTree};
use crate::windowing::WindowHelper;
use crate::Graphics;

impl crate::windowing::WindowHandler for WidgetTree<OverlayState> {
    fn on_init(&mut self, graphics: &mut Graphics) { crate::assets::preload_assets(graphics); }

    fn on_frame_begin(&mut self, graphics: &mut Graphics, jobs: Vec<JobResult>) { process_jobs(graphics, jobs); }

    fn on_frame(&mut self, graphics: &mut Graphics) -> bool {
        graphics.cache_settings(&self.data.settings);
        self.render(graphics);
        true
    }

    fn on_fixed_update(&mut self, delta_time: f32, helper: &mut WindowHelper) -> bool {
        let fixed = self.fixed_update(delta_time);
        let running = self.data.process_is_running(helper);
        fixed || running
    }

    fn on_input(&mut self, helper: &mut WindowHelper, action: &Actions) -> bool {
        if self.data.process.borrow().is_none() {
            return false;
        }
        match action {
            crate::Actions::ToggleOverlay => {
                self.data.toggle_visibility(helper);
                true
            }
            _ => {
                if self.data.showing {
                    let mut handler = DeferredAction::new();
                    self.action(action, &mut handler);
                    handler.resolve(self);

                    if let Actions::Accept = action {
                        let mut process = self.data.process.borrow_mut();
                        process.as_mut().unwrap().kill().log("Unable to kill running process");
                        *process = None;
                        helper.set_visibility(false);
                        self.data.showing = false;
                        return true;
                    }
                }
                false
            }
        }
    }
}

fn process_jobs(graphics: &mut Graphics, job_results: Vec<JobResult>) {
    for r in job_results {
        if let JobResult::LoadImage { data, dimensions, key } = r {
            let mut map = graphics.asset_cache.borrow_mut();
            let asset_slot = crate::assets::get_asset_slot(&mut map, &key);
            asset_slot.set_data(data, dimensions);
        }
    }
}
