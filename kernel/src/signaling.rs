use embassy_rp::pac::TIMER;

const RESTART_ALARM_N: usize = 3;

/// Signal core1 that a restart should be issued.
pub fn k_signal_user_restart() {
    // TODO signal via doorbells once on RP235x
    //      or via PSM and re-setup which maybe a better solution
    let next = TIMER.timelr().read() + 100;
    TIMER.alarm(RESTART_ALARM_N).write_value(next);
}

/// Clear interrupt status,
/// should be run inside interrupt handler to ensure it does not re-fire.
pub fn k_signal_user_restart_reset() {
    TIMER.intr().write(|w| w.set_alarm(RESTART_ALARM_N, true));
}

/// Setup the signalling to handle receiving a signal.
/// Should be run once on core1 start.
pub fn k_signal_user_restart_setup() {
    use embassy_rp::interrupt;
    use embassy_rp::interrupt::InterruptExt;
    unsafe { interrupt::TIMER_IRQ_3.enable() };
    TIMER.inte().modify(|w| w.set_alarm(RESTART_ALARM_N, true));
}
