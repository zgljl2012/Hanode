
pub trait NodeLifecycleHooks {
    // Trigger this function when the node is destroyed.
    fn on_stopped(&self);
}
