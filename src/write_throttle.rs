//! Dynamic write throttle when L0 SSTables exceed safety watermark.

pub struct WriteThrottle {
    pub l0_slowdown_trigger: usize,
    pub l0_stop_trigger: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ThrottleAction {
    Proceed,
    DelayMillis(u64),
    Block,
}

impl WriteThrottle {
    pub fn new(l0_slowdown_trigger: usize, l0_stop_trigger: usize) -> Self {
        Self {
            l0_slowdown_trigger,
            l0_stop_trigger,
        }
    }

    pub fn evaluate(&self, current_l0_count: usize) -> ThrottleAction {
        if current_l0_count >= self.l0_stop_trigger {
            ThrottleAction::Block
        } else if current_l0_count >= self.l0_slowdown_trigger {
            ThrottleAction::DelayMillis(5)
        } else {
            ThrottleAction::Proceed
        }
    }
}
