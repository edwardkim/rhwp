use super::{DeviceContext, SVGPlayer, SelectedGraphicsObject};

/// 재생 상태와 선택 객체·클립은 같은 SaveDC 시점에 고정한다.
pub(super) struct SavedDeviceContext {
    context: DeviceContext,
    selected: SelectedGraphicsObject,
    clip_id: Option<String>,
}

impl SVGPlayer {
    pub(super) fn save_playback_context(&mut self) {
        self.context_stack.push(SavedDeviceContext {
            context: self.context_current.clone(),
            selected: self.object_selected.clone(),
            clip_id: self.current_clip_id.clone(),
        });
    }

    pub(super) fn restore_playback_context(&mut self, saved: i16) {
        // 양수는 1부터 시작하는 저장 번호, 음수는 스택 끝에서의 상대 번호다.
        // 잘못된 번호는 현재 상태나 스택을 바꾸지 않는다.
        let index = if saved < 0 {
            self.context_stack
                .len()
                .checked_sub(saved.unsigned_abs().into())
        } else {
            usize::try_from(saved).ok().and_then(|n| n.checked_sub(1))
        };
        let Some(index) = index.filter(|&n| n < self.context_stack.len()) else {
            return;
        };
        let mut state = self.context_stack.remove(index);
        self.context_stack.truncate(index);
        // 생성·삭제된 WMF 객체 표는 재생 스트림의 자원이다. DC 복원으로
        // 과거 표를 되살리거나 SaveDC 이후 생성된 객체를 잃지 않는다.
        std::mem::swap(
            &mut state.context.object_table,
            &mut self.context_current.object_table,
        );
        self.context_current = state.context;
        self.object_selected = state.selected;
        self.current_clip_id = state.clip_id;
    }
}
