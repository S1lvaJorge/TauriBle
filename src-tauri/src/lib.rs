// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant}; // Importação correta

use lazy_static::lazy_static;

#[derive(Serialize, Clone)]
struct Device {
    name: String,
    address: String,
}

lazy_static! {
    static ref PERIPHERALS: Arc<Mutex<Vec<Device>>> = Arc::new(Mutex::new(Vec::new()));
}

// Existing greet command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// New scan_devices command
#[tauri::command]
async fn scan_devices() -> Result<Vec<Device>, String> {
    println!("Scan command called!");
    let manager = Manager::new().await.map_err(|e| e.to_string())?;
    let adapters = manager.adapters().await.map_err(|e| e.to_string())?;
    let central = adapters.into_iter().nth(0).ok_or("No Bluetooth adapters found")?;

    central.start_scan(ScanFilter::default()).await.map_err(|e| e.to_string())?;
    println!("Scanning started...");

    let scan_duration = Duration::from_secs(10);
    let scan_start = Instant::now();
    let mut events = central.events().await.map_err(|e| e.to_string())?;
    let mut devices = Vec::new();

    while scan_start.elapsed() < scan_duration {
        if let Some(event) = events.next().await {
            if let CentralEvent::DeviceDiscovered(id) = event {
                let peripheral = central.peripheral(&id).await.map_err(|e| e.to_string())?;
                let properties = peripheral.properties().await.map_err(|e| e.to_string())?;

                if let Some(props) = properties {
                    let name = props.local_name.unwrap_or("Unknown".to_string());
                    let address = props.address.to_string();

                    println!("Device found: {} - {}", name, address);
                    devices.push(Device { name, address });
                }
            }
        } else {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    println!("Scan completed, {} devices found", devices.len());
    central.stop_scan().await.map_err(|e| e.to_string())?;

    // Atualiza a lista global de dispositivos encontrados
    let mut peripherals = PERIPHERALS.lock().await;
    *peripherals = devices.clone();

    Ok(devices)
}

#[tauri::command]
async fn connect_to_device(address: String) -> Result<String, String> {
    use btleplug::api::Peripheral as _;

    println!("Tentando conectar ao dispositivo: {}", address);

    let manager = Manager::new().await.map_err(|e| e.to_string())?;
    let adapters = manager.adapters().await.map_err(|e| e.to_string())?;
    let central = adapters.into_iter().nth(0).ok_or("No Bluetooth adapters found")?;

    // Obtém a lista de dispositivos escaneados
    let peripherals = PERIPHERALS.lock().await;

    let device_info = peripherals.iter().find(|p| p.address == address)
        .ok_or("Device not found in scanned list")?;

    println!("Tentando conectar ao dispositivo: {} ({})", device_info.name, address);

    let peripheral = central.peripherals().await.map_err(|e| e.to_string())?
        .into_iter()
        .find(|p| p.address().to_string() == address)
        .ok_or("Device not found")?;

    if let Err(e) = peripheral.connect().await {
        return Err(format!("Failed to connect: {}", e));
    }

    println!("Conectado com sucesso ao dispositivo {}", address);
    Ok(format!("Connected to {}", address))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_macos_permissions::init())
        .invoke_handler(tauri::generate_handler![greet, scan_devices, connect_to_device])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
