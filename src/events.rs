/// A cli event which is either some form of input or just a [CliEvent::Tick]
/// signaling that time has passed.
pub enum CliEvent<I> {
    Input(I),
    Tick,
}

/// An input event can either signal the application to [InputEvent::Quit] or
/// to [InputEvent::Continue] running.
pub enum InputEvent {
    Quit,
    Continue,
}
