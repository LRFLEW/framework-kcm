// SPDX-License-Identifier: GPL-3.0-or-later
//! Blocking hardware access through framework_lib. Everything here runs on a
//! blocking thread with the EC mutex held; results are plain a{sv} dicts so
//! the D-Bus layer stays thin.
//!
//! Nothing here is display text: enums and locations go out as stable
//! kebab-case keys and the KCM turns them into translated strings, since a
//! root system service can't know the user's language.

use std::collections::HashMap;

use dmidecode::Structure;
use framework_lib::ccgx::{self, MainPdVersions, PdVersions};
use framework_lib::chromium_ec::commands::FpLedBrightnessLevel;
use framework_lib::chromium_ec::{CrosEc, EcCurrentImage, EcError, EcResult};
use framework_lib::commandline::{ClickForceArg, FpBrightnessArg};
use framework_lib::power::{self, FanSpeed, TempSensor, UsbChargingType, UsbPowerRoles};
use framework_lib::smbios::{self, PlatformFamily};
use framework_lib::touchpad::{self, ClickForce};
// from_str() on framework_tool's argument enums gives us its CLI names
use clap::ValueEnum;
use zbus::zvariant::{OwnedValue, Value};

pub type Dict = HashMap<String, OwnedValue>;

pub fn val<'a, T: Into<Value<'a>>>(x: T) -> OwnedValue {
    // Only fails for file descriptors, which we never send
    OwnedValue::try_from(x.into()).expect("value without fds")
}

macro_rules! dict {
    ($($k:expr => $v:expr),* $(,)?) => {{
        let mut d = $crate::ec::Dict::new();
        $(d.insert($k.to_string(), $crate::ec::val($v));)*
        d
    }};
}
pub(crate) use dict;

fn err(e: EcError) -> String {
    match e {
        EcError::Response(status) => format!("EC returned {status:?}"),
        EcError::UnknownResponseCode(code) => format!("EC returned unknown code {code}"),
        EcError::DeviceError(msg) => msg,
    }
}

fn ok<T>(r: EcResult<T>) -> Result<T, String> {
    r.map_err(err)
}

// ----- System information -----------------------------------------------

pub fn system_info(ec: &CrosEc) -> Dict {
    let (bios_version, bios_date) = smbios::get_smbios()
        .and_then(|s| {
            s.structures().find_map(|r| match r {
                Ok(Structure::Bios(b)) => {
                    Some((b.bios_version.to_string(), b.bios_release_date.to_string()))
                }
                _ => None,
            })
        })
        .unwrap_or_default();

    let (ec_ro, ec_rw, ec_image) = ec
        .flash_version()
        .map(|(ro, rw, cur)| {
            let image = match cur {
                EcCurrentImage::RO => "ro",
                EcCurrentImage::RW => "rw",
                EcCurrentImage::Unknown => "unknown",
            };
            (ro, rw, image.to_string())
        })
        .unwrap_or_default();

    let (mic, camera) = ec.get_privacy_info().ok().unzip();

    dict! {
        "product" => smbios::get_product_name().unwrap_or_default(),
        "platform" => smbios::get_platform().map(|p| format!("{p:?}")).unwrap_or_default(),
        "isFramework" => smbios::is_framework(),
        "biosVersion" => bios_version,
        "biosDate" => bios_date,
        "ecVersion" => ec.version_info().unwrap_or_default(),
        "ecRoVersion" => ec_ro,
        "ecRwVersion" => ec_rw,
        "ecCurrentImage" => ec_image,
        "pdVersions" => pd_versions(ec),
        "micEnabled" => mic.unwrap_or(false),
        "cameraEnabled" => camera.unwrap_or(false),
        "privacyKnown" => mic.is_some(),
    }
}

/// One dict per PD controller: `side` ("right", "left" or "" when not
/// split by side), `index` and `version`.
fn pd_versions(ec: &CrosEc) -> Vec<Dict> {
    let pd = |side: &str, index: usize, version: String| {
        dict! { "side" => side, "index" => index as i32, "version" => version }
    };
    match ccgx::get_pd_controller_versions(ec) {
        Ok(PdVersions::RightLeft((right, left))) => {
            vec![pd("right", 0, right.active_fw_ver()), pd("left", 1, left.active_fw_ver())]
        }
        Ok(PdVersions::Single(p)) => vec![pd("", 0, p.active_fw_ver())],
        Ok(PdVersions::Many(pds)) => pds.iter().enumerate().map(|(i, p)| pd("", i, p.active_fw_ver())).collect(),
        // Not every platform exposes the PD chips over I2C passthrough,
        // fall back to asking the EC
        Err(_) => match power::read_pd_version(ec) {
            Ok(MainPdVersions::RightLeft((right, left))) => {
                vec![pd("right", 0, right.app.to_string()), pd("left", 1, left.app.to_string())]
            }
            Ok(MainPdVersions::Single(p)) => vec![pd("", 0, p.app.to_string())],
            Ok(MainPdVersions::Many(pds)) => pds.iter().enumerate().map(|(i, p)| pd("", i, p.app.to_string())).collect(),
            Err(_) => vec![],
        },
    }
}

// ----- Battery ------------------------------------------------------------

pub fn power(ec: &CrosEc) -> Result<Dict, String> {
    let info = power::power_info(ec).ok_or("Failed to read power information")?;
    let mut d = dict! {
        "acPresent" => info.ac_present,
        "batteryPresent" => info.battery.is_some(),
    };
    if let Some(b) = info.battery {
        d.extend(dict! {
            "percentage" => b.charge_percentage as i32,
            "charging" => b.charging,
            "discharging" => b.discharging,
            "critical" => b.level_critical,
            "cycleCount" => b.cycle_count as i32,
            "designCapacity" => b.design_capacity as i32,
            "lastFullCapacity" => b.last_full_charge_capacity as i32,
            "remainingCapacity" => b.remaining_capacity as i32,
            "voltage" => b.present_voltage as i32,
            "designVoltage" => b.design_voltage as i32,
            "rate" => b.present_rate as i32,
            "manufacturer" => b.manufacturer,
            "model" => b.model_number,
            "chemistry" => b.battery_type,
        });
    }
    Ok(d)
}

pub fn charge_limit(ec: &CrosEc) -> Result<(u8, u8), String> {
    ok(ec.get_charge_limit())
}

pub fn set_charge_limit(ec: &CrosEc, max: i32) -> Result<(), String> {
    // Same bounds framework_tool enforces
    if !(25..=100).contains(&max) {
        return Err("Charge limit must be between 25% and 100%".into());
    }
    let (min, _) = ok(ec.get_charge_limit())?;
    ok(ec.set_charge_limit(min, max as u8))
}

pub fn set_charge_rate_limit(ec: &CrosEc, rate: f64, soc: Option<f64>) -> Result<(), String> {
    if !(0.1..=1.0).contains(&rate) {
        return Err("Charge rate must be between 0.1C and 1.0C".into());
    }
    if soc.is_some_and(|s| !(0.0..=100.0).contains(&s)) {
        return Err("Battery level must be between 0% and 100%".into());
    }
    ok(ec.set_charge_rate_limit(rate as f32, soc.map(|s| s as f32)))
}

// ----- Thermals & fans ----------------------------------------------------

pub fn thermal(ec: &CrosEc) -> Result<(Vec<Dict>, Vec<Dict>, Dict), String> {
    let info = ok(power::get_thermal(ec))?;

    let sensors = info
        .sensors
        .into_iter()
        .map(|s| {
            let (temp, status) = match s.temp {
                TempSensor::Ok(t) => (t as i32, "ok"),
                TempSensor::NotPresent => (-1, "not-present"),
                TempSensor::Error => (-1, "error"),
                TempSensor::NotPowered => (-1, "not-powered"),
                TempSensor::NotCalibrated => (-1, "not-calibrated"),
            };
            dict! {
                "index" => s.index as i32,
                "location" => sensor_location(&s.name),
                "name" => s.name,
                "temp" => temp,
                "status" => status,
            }
        })
        .collect();

    let fans = info
        .fans
        .into_iter()
        .filter(|f| f.speed != FanSpeed::NotPresent)
        .map(|f| {
            let (rpm, stalled) = match f.speed {
                FanSpeed::Rpm(rpm) => (rpm as i32, false),
                _ => (0, true),
            };
            dict! {
                "index" => f.index as i32,
                "position" => fan_position(&f.name),
                "name" => f.name,
                "rpm" => rpm,
                "stalled" => stalled,
            }
        })
        .collect();

    let throttle = match info.throttle {
        Some(t) => dict! { "known" => true, "soft" => t.soft, "hard" => t.hard },
        None => dict! { "known" => false },
    };

    Ok((sensors, fans, throttle))
}

/// Where an EC temperature sensor sits, as a stable key ("" if unknown;
/// the KCM then shows the raw name).
///
/// EC names look like `<location>_<chip>@<i2c addr>` (e.g. `cpu_f75303@4d`),
/// or just the source for non-I2C sensors (`peci-temp`, `battery_temp@b`).
fn sensor_location(name: &str) -> &'static str {
    match name.split(['_', '@', '-']).next().unwrap_or(name) {
        // Read from the CPU itself over PECI
        "peci" => "cpu",
        // Board sensor placed next to the CPU
        "cpu" | "apu" | "soc" => "near-cpu",
        "ddr" | "memory" | "mem" => "memory",
        // The sensor chip's own on-die diode
        "local" => "mainboard",
        "battery" => "battery",
        "charger" => "charger",
        "skin" => "chassis",
        "ssd" | "nvme" => "ssd",
        "gpu" | "dgpu" => "gpu",
        "wifi" | "wlan" => "wifi",
        _ => "",
    }
}

/// framework_lib only names fans in English; turn those names back into keys.
fn fan_position(name: &str) -> &'static str {
    match name {
        "APU Fan" => "apu",
        "Left Fan" => "left",
        "Right Fan" => "right",
        "Front Fan" => "front",
        "Third Fan" => "third",
        _ => "",
    }
}

fn fan_idx(fan: i32) -> Option<u32> {
    (fan >= 0).then_some(fan as u32)
}

pub fn set_fan_duty(ec: &CrosEc, fan: i32, percent: i32) -> Result<(), String> {
    if !(0..=100).contains(&percent) {
        return Err("Fan duty must be between 0% and 100%".into());
    }
    ok(ec.fan_set_duty(fan_idx(fan), percent as u32))
}

pub fn set_fan_rpm(ec: &CrosEc, fan: i32, rpm: i32) -> Result<(), String> {
    if rpm < 0 {
        return Err("Fan RPM must not be negative".into());
    }
    ok(ec.fan_set_rpm(fan_idx(fan), rpm as u32))
}

pub fn set_auto_fan(ec: &CrosEc, fan: i32) -> Result<(), String> {
    ok(ec.autofanctrl(fan_idx(fan).map(|f| f as u8)))
}

// ----- Input & lighting ---------------------------------------------------

fn fp_level_name(level: &FpLedBrightnessLevel) -> &'static str {
    match level {
        FpLedBrightnessLevel::High => "high",
        FpLedBrightnessLevel::Medium => "medium",
        FpLedBrightnessLevel::Low => "low",
        FpLedBrightnessLevel::UltraLow => "ultra-low",
        FpLedBrightnessLevel::Custom => "custom",
        FpLedBrightnessLevel::Auto => "auto",
    }
}

pub fn fp_led(ec: &CrosEc) -> Result<(u8, String), String> {
    let (percent, level) = ok(ec.get_fp_led_level())?;
    Ok((percent, level.map(|l| fp_level_name(&l).to_string()).unwrap_or_default()))
}

pub fn set_fp_led_level(ec: &CrosEc, level: &str) -> Result<(), String> {
    let level = FpBrightnessArg::from_str(level, false)
        .map_err(|_| format!("Unknown fingerprint LED level '{level}'"))?;
    ok(ec.set_fp_led_level(level.into()))
}

pub fn set_haptic_intensity(value: i32) -> Result<(), String> {
    let value = u8::try_from(value).map_err(|_| "Invalid haptic intensity".to_string())?;
    touchpad::set_haptic_intensity(value).map_err(|e| e.to_string())
}

pub fn set_click_force(force: &str) -> Result<(), String> {
    let force: ClickForce = ClickForceArg::from_str(force, false)
        .map_err(|_| format!("Unknown click force '{force}'"))?
        .into();
    touchpad::set_click_force(force).map_err(|e| e.to_string())
}

// ----- USB-C ports --------------------------------------------------------

pub fn ports(ec: &CrosEc) -> Vec<Dict> {
    // All Framework laptops so far have 4 PD ports; same positions as framework_tool --pdports
    let positions = match smbios::get_family() {
        Some(PlatformFamily::Framework16) => ["right-back", "right-middle", "left-middle", "left-back"],
        _ => ["right-back", "right-front", "left-front", "left-back"],
    };
    power::get_pd_info(ec, positions.len() as u8)
        .into_iter()
        .zip(positions)
        .enumerate()
        .map(|(i, (info, position))| {
            let mut d = dict! { "index" => i as i32, "position" => position };
            match info {
                Ok(p) => d.extend(dict! {
                    "ok" => true,
                    "role" => power_role(&p.role),
                    "chargingType" => charging_type(&p.charging_type),
                    "dualRole" => p.dualrole,
                    "voltageNow" => p.meas.voltage_now as i32,
                    "voltageMax" => p.meas.voltage_max as i32,
                    "currentMax" => p.meas.current_max as i32,
                    "currentLimit" => p.meas.current_lim as i32,
                    "maxPower" => p.max_power as i32,
                }),
                Err(e) => d.extend(dict! { "ok" => false, "error" => err(e) }),
            }
            d
        })
        .collect()
}

fn power_role(role: &UsbPowerRoles) -> &'static str {
    match role {
        UsbPowerRoles::Disconnected => "disconnected",
        UsbPowerRoles::Source => "source",
        UsbPowerRoles::Sink => "sink",
        UsbPowerRoles::SinkNotCharging => "sink-not-charging",
    }
}

fn charging_type(t: &UsbChargingType) -> &'static str {
    match t {
        UsbChargingType::None => "none",
        UsbChargingType::PD => "pd",
        UsbChargingType::TypeC => "type-c",
        UsbChargingType::Proprietary => "proprietary",
        UsbChargingType::Bc12Dcp => "bc12-dcp",
        UsbChargingType::Bc12Cdp => "bc12-cdp",
        UsbChargingType::Bc12Sdp => "bc12-sdp",
        UsbChargingType::Other => "other",
        UsbChargingType::VBus => "vbus",
        UsbChargingType::Unknown => "unknown",
    }
}
