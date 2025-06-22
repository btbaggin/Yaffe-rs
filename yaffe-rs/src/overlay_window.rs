use crate::logger::LogEntry;
use crate::overlay_state::OverlayState;
use crate::input::Actions;
use crate::ui::{AnimationManager, WidgetTree};
use crate::widgets::OverlayBackground;
use crate::windowing::WindowHelper;
use crate::Graphics;
use crate::job_system::JobResult;


impl crate::windowing::WindowHandler for WidgetTree<OverlayState, ()> {
    fn on_init(&mut self, graphics: &mut Graphics) { crate::assets::preload_assets(graphics); }

    fn on_frame_begin(&mut self, graphics: &mut Graphics, jobs: &mut Vec<JobResult>) {
        process_jobs(graphics, jobs);
    }

    fn on_frame(&mut self, graphics: &mut Graphics) -> bool {
        graphics.cache_settings(&self.data.settings);
        self.render_all(graphics);
        true
    }

    fn on_fixed_update(&mut self, animations: &mut AnimationManager, delta_time: f32, helper: &mut WindowHelper) -> bool {
        animations.process(&mut self.root, delta_time);
        self.data.process_is_running(helper)
    }

    fn on_input(&mut self, animations: &mut AnimationManager, helper: &mut WindowHelper, action: &Actions) -> bool {
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
                    self.root.action(&mut self.data, animations, action, &crate::ui::WidgetId::of::<OverlayBackground>(), &mut ());

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

fn process_jobs(graphics: &mut Graphics, job_results: &mut Vec<JobResult>) {
    crate::job_system::process_results(
        job_results,
        |j| matches!(j, JobResult::LoadImage { .. } | JobResult::SearchGame(_) | JobResult::SearchPlatform(_)),
        |result| match result {
            JobResult::LoadImage { data, dimensions, key } => {
                let mut map = graphics.asset_cache.borrow_mut();
                let asset_slot = crate::assets::get_asset_slot(&mut map, &key);
                asset_slot.set_data(data, dimensions);
            }
            _ => {}
        },
    );
}
