use std::sync::atomic::{AtomicBool, Ordering};

static APP_LOCK_ACQUIRED: AtomicBool = AtomicBool::new(false);

pub fn acquire_single_instance_stub() -> Result<(), String> {
    if APP_LOCK_ACQUIRED.swap(true, Ordering::SeqCst) {
        return Err("app lock is already acquired".to_string());
    }
    Ok(())
}
