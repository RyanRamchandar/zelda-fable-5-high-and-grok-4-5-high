use web_sys::window;

pub fn save(key: &str, json: &str) -> Result<(), String> {
    let window = window().ok_or("no window")?;
    let storage = window
        .local_storage()
        .map_err(|_| "localStorage access")?
        .ok_or("localStorage unavailable")?;
    storage
        .set_item(key, json)
        .map_err(|_| "localStorage set_item failed")?;
    Ok(())
}

pub fn load(key: &str) -> Option<String> {
    let window = window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item(key).ok()?
}
