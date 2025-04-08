use crate::image_ops::save_image;
use miette::{Context, IntoDiagnostic, Result};
use mlua::{Lua, Result as LuaResult, Value as LuaValue};
use std::{cell::RefCell, path::PathBuf};
use tokio::task;

thread_local! {
    static IMAGE_SETTINGS: RefCell<ImageConfig> = RefCell::new(ImageConfig::default());
}

/// Struct holding modifiable state used by Lua.
#[derive(Debug, Clone)]
pub struct ImageConfig {
    pub size: u32,
    pub format: String,
    pub foreground: String,
    pub background: String,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            size: 512,
            format: "svg".into(),
            foreground: "#ffffff".into(),
            background: "#000000".into(),
        }
    }
}

pub struct LuaAPI;

impl LuaAPI {
    pub fn new() -> Self {
        Self
    }

    pub fn register_globals(&self, lua: &Lua) -> LuaResult<()> {
        let globals = lua.globals();
        let ciphercanvas = lua.create_table()?;

        // Get current config
        ciphercanvas.set(
            "get_config",
            lua.create_function(|lua, ()| {
                IMAGE_SETTINGS.with(|cfg| {
                    let config = cfg.borrow();
                    let table = lua.create_table()?;
                    table.set("size", config.size)?;
                    table.set("format", config.format.clone())?;
                    table.set("foreground", config.foreground.clone())?;
                    table.set("background", config.background.clone())?;
                    Ok(table)
                })
            })?,
        )?;

        // Set a config value
        ciphercanvas.set(
            "set_config",
            lua.create_function(|_, (key, value): (String, LuaValue)| {
                IMAGE_SETTINGS.with(|cfg| {
                    let mut config = cfg.borrow_mut();
                    match key.as_str() {
                        "size" => {
                            if let LuaValue::Integer(i) = value {
                                config.size = i as u32;
                            }
                        }
                        "format" => {
                            if let LuaValue::String(s) = value {
                                config.format = s.to_str()?.to_string();
                            }
                        }
                        "foreground" => {
                            if let LuaValue::String(s) = value {
                                config.foreground = s.to_str()?.to_string();
                            }
                        }
                        "background" => {
                            if let LuaValue::String(s) = value {
                                config.background = s.to_str()?.to_string();
                            }
                        }
                        _ => {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Unknown config key: {key}"
                            )));
                        }
                    }
                    Ok(())
                })
            })?,
        )?;

        // Render and save image
        ciphercanvas.set(
            "save_image",
            lua.create_async_function(
                |_, (output_path, svg_content): (String, String)| async move {
                    let config = IMAGE_SETTINGS.with(|cfg| cfg.borrow().clone());
                    let output = PathBuf::from(output_path);
                    task::spawn_blocking(move || {
                        save_image(&output, &config.format, &svg_content, config.size)
                    })
                    .await
                    .map_err(|e| mlua::Error::RuntimeError(format!("JoinError: {e}")))?
                    .map_err(|e| mlua::Error::RuntimeError(format!("SaveError: {e}")))?;
                    Ok(())
                },
            )?,
        )?;

        globals.set("ciphercanvas", ciphercanvas)?;
        Ok(())
    }
}

pub async fn execute_script(script_path: PathBuf) -> Result<()> {
    let script_contents = tokio::fs::read_to_string(&script_path)
        .await
        .into_diagnostic()
        .with_context(|| format!("Failed to read Lua script from {:?}", script_path))?;

    let lua = Lua::new();
    let api = LuaAPI::new();
    api.register_globals(&lua).into_diagnostic()?;

    lua.load(&script_contents)
        .exec()
        .into_diagnostic()
        .with_context(|| "Error executing Lua script")?;

    Ok(())
}
