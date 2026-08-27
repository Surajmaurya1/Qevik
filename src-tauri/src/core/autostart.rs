use tracing::{error, info};

const RUN_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const APP_KEY_NAME: &str = "SpotlightForWindows";

/// Configure the application to start automatically with Windows using HKCU Run key.
#[cfg(windows)]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY_CURRENT_USER,
        KEY_SET_VALUE, REG_SZ,
    };

    let subkey_wide: Vec<u16> = OsStr::new(RUN_KEY_PATH)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let name_wide: Vec<u16> = OsStr::new(APP_KEY_NAME)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut hkey = windows::Win32::System::Registry::HKEY::default();
        let open_status = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR::from_raw(subkey_wide.as_ptr()),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        );

        if open_status != ERROR_SUCCESS {
            let msg = format!(
                "Failed to open HKCU Run registry key: error code {:?}",
                open_status
            );
            error!("{}", msg);
            return Err(msg);
        }

        let result = if enabled {
            let current_exe = std::env::current_exe()
                .map_err(|e| format!("Failed to resolve current exe path: {}", e))?;
            let val_str = format!("\"{}\" --startup", current_exe.to_string_lossy());
            let val_wide: Vec<u16> = OsStr::new(&val_str)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let byte_len = (val_wide.len() * std::mem::size_of::<u16>()) as u32;
            let status = RegSetValueExW(
                hkey,
                PCWSTR::from_raw(name_wide.as_ptr()),
                0,
                REG_SZ,
                Some(std::slice::from_raw_parts(
                    val_wide.as_ptr() as *const u8,
                    byte_len as usize,
                )),
            );

            if status == ERROR_SUCCESS {
                info!("Enabled startup with Windows via registry: {}", val_str);
                Ok(())
            } else {
                let msg = format!("Failed to set Run registry value: error code {:?}", status);
                error!("{}", msg);
                Err(msg)
            }
        } else {
            let status = RegDeleteValueW(hkey, PCWSTR::from_raw(name_wide.as_ptr()));
            if status == ERROR_SUCCESS || status.0 == 2 {
                // 2 is ERROR_FILE_NOT_FOUND (already deleted)
                info!("Disabled startup with Windows via registry");
                Ok(())
            } else {
                let msg = format!(
                    "Failed to delete Run registry value: error code {:?}",
                    status
                );
                error!("{}", msg);
                Err(msg)
            }
        };

        let _ = RegCloseKey(hkey);
        result
    }
}

#[cfg(not(windows))]
pub fn set_autostart(_enabled: bool) -> Result<(), String> {
    Ok(())
}
