use anyhow::Result;
use winreg::enums::*;
use winreg::RegKey;

// ─── Registry Hive Mapping ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hive {
    Hklm,
    Hkcu,
}

impl Hive {
    fn predef(self) -> RegKey {
        RegKey::predef(match self {
            Hive::Hklm => HKEY_LOCAL_MACHINE,
            Hive::Hkcu => HKEY_CURRENT_USER,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Hive::Hklm => "HKLM",
            Hive::Hkcu => "HKCU",
        }
    }
}

// ─── Registry Read ───────────────────────────────────────────────────────────

pub fn read_dword(hive: Hive, subkey: &str, value: &str) -> Option<u32> {
    let root = hive.predef();
    let key = root.open_subkey_with_flags(subkey, KEY_READ).ok()?;
    key.get_value::<u32, _>(value).ok()
}

pub fn read_string(hive: Hive, subkey: &str, value: &str) -> Option<String> {
    let root = hive.predef();
    let key = root.open_subkey_with_flags(subkey, KEY_READ).ok()?;
    key.get_value::<String, _>(value).ok()
}

// ─── Registry Write ──────────────────────────────────────────────────────────

pub fn write_dword(hive: Hive, subkey: &str, value: &str, data: u32) -> Result<()> {
    let root = hive.predef();
    let key = root.create_subkey_with_flags(subkey, KEY_SET_VALUE | KEY_READ)?;
    key.0.set_value(value, &data)?;
    Ok(())
}

pub fn write_string(hive: Hive, subkey: &str, value: &str, data: &str) -> Result<()> {
    let root = hive.predef();
    let key = root.create_subkey_with_flags(subkey, KEY_SET_VALUE | KEY_READ)?;
    key.0.set_value(value, &data.to_string())?;
    Ok(())
}

pub fn delete_value(hive: Hive, subkey: &str, value: &str) -> Result<()> {
    let root = hive.predef();
    let key = root.open_subkey_with_flags(subkey, KEY_SET_VALUE | KEY_READ)?;
    key.delete_value(value)?;
    Ok(())
}
