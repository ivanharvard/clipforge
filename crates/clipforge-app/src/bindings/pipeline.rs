use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::app_state::AppState;
use crate::{App, PipelineStageView, PipelineState};

fn sync(app: &App, state: &AppState) {
    let stages = state
        .pipeline()
        .iter()
        .map(|stage| PipelineStageView {
            label: stage.step.label().into(),
            enabled: stage.enabled,
        })
        .collect::<Vec<_>>();
    app.global::<PipelineState>()
        .set_stages(ModelRc::new(VecModel::from(stages)));
}

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    sync(app, &state.borrow());
    let pipeline = app.global::<PipelineState>();

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        pipeline.on_enabled_changed(move |index, enabled| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut state = state.borrow_mut();
            state.set_pipeline_enabled(index as usize, enabled);
            sync(&app, &state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        pipeline.on_move_up_clicked(move |index| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut state = state.borrow_mut();
            state.move_pipeline_stage(index as usize, -1);
            sync(&app, &state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        pipeline.on_move_down_clicked(move |index| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut state = state.borrow_mut();
            state.move_pipeline_stage(index as usize, 1);
            sync(&app, &state);
        });
    }
}
