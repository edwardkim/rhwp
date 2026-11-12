//! Strong references for owner-thread producer work, including panic unwinding.
//! A producer must pin resources before recording work and only mark completion
//! after its backend proof succeeds. Unknown completion never returns credits.

use std::rc::Rc;

pub(crate) struct FrameResources<S, T> {
    pub scratch: Option<Rc<S>>,
    working: Vec<Rc<S>>,
    tiles: Vec<Rc<T>>,
    quarantined: bool,
}

impl<S, T> Default for FrameResources<S, T> {
    fn default() -> Self {
        Self {
            scratch: None,
            working: Vec::new(),
            tiles: Vec::new(),
            quarantined: false,
        }
    }
}

impl<S, T> FrameResources<S, T> {
    pub fn pin_working(&mut self, target: &Rc<S>) {
        if !self.working.iter().any(|held| Rc::ptr_eq(held, target)) {
            self.working.push(Rc::clone(target));
        }
    }

    pub fn pin_tile(&mut self, tile: &Rc<T>) {
        if !self.tiles.iter().any(|held| Rc::ptr_eq(held, tile)) {
            self.tiles.push(Rc::clone(tile));
        }
    }

    pub fn working_count(&self) -> usize {
        self.working.len()
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    pub fn complete_working(&mut self) -> bool {
        if self.quarantined {
            return false;
        }
        self.working.clear();
        true
    }

    pub fn complete_frame(&mut self) -> bool {
        if !self.complete_working() {
            return false;
        }
        self.tiles.clear();
        true
    }

    pub fn clear_completed(&mut self) -> bool {
        if !self.complete_frame() {
            return false;
        }
        self.scratch.take();
        true
    }

    pub fn quarantine(&mut self) {
        self.quarantined = true;
    }
}

impl<S, T> Drop for FrameResources<S, T> {
    fn drop(&mut self) {
        if self.quarantined {
            // Neither cancel nor stack unwinding proves producer completion.
            std::mem::forget(self.scratch.take());
            std::mem::forget(std::mem::take(&mut self.working));
            std::mem::forget(std::mem::take(&mut self.tiles));
        }
    }
}
