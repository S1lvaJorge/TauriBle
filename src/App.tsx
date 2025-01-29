import { useState } from "react";
//import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [devices, setDevices] = useState<{ name: string; address: string }[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<{ [key: string]: string }>({});

  async function scanForDevices() {
    setIsScanning(true);
    setError(null);

    try {
      console.log("Scanning...");
      const scannedDevices = await invoke<{ name: string; address: string }[]>("scan_devices");
      setDevices(scannedDevices);
    } catch (err) {
      console.error("Error scanning devices:", err);
      setError("Failed to scan devices. Make sure Bluetooth is enabled.");
    } finally {
      setIsScanning(false);
    }
  }
  
  async function connectToDevice(address: string) {
    setConnectionStatus((prev) => ({ ...prev, [address]: "Connecting..." }));
  
    try {
      const result = await invoke<string>("connect_to_device", { address });
      setConnectionStatus((prev) => ({ ...prev, [address]: "Connected!" }));
      console.log("Success:", result);
    } catch (err) {
      console.error("Error connecting to device:", err);
      setConnectionStatus((prev) => ({ ...prev, [address]: `Failed: ${err}` }));
    }
  }  

  return (
    <div>
      <h1>Bluetooth Device Scanner</h1>

      <button onClick={scanForDevices} disabled={isScanning}>
        {isScanning ? "Scanning..." : "Scan for Devices"}
      </button>

      {error && <p style={{ color: "red" }}>{error}</p>}

      <h2>Discovered Devices:</h2>

      {/* Tabela de dispositivos */}
      <table style={{ width: "100%", borderCollapse: "collapse", marginTop: "20px" }}>
        <thead>
          <tr style={{ backgroundColor: "#000000" }}>
            <th style={{ padding: "10px", textAlign: "left" }}>Device Name</th>
            <th style={{ padding: "10px", textAlign: "left" }}>Address</th>
            <th style={{ padding: "10px", textAlign: "center" }}>Action</th>
          </tr>
        </thead>
        <tbody>
          {devices.length > 0 ? (
            devices.map((device, index) => (
              <tr key={index} style={{ borderBottom: "1px solid #ddd" }}>
                <td style={{ padding: "10px" }}>
                  <strong>{device.name}</strong>
                </td>
                <td style={{ padding: "10px" }}>{device.address}</td>
                <td style={{ padding: "10px", textAlign: "center" }}>
                  <button
                    onClick={() => connectToDevice(device.address)}
                    style={{
                      padding: "6px 12px",
                      cursor: "pointer",
                      borderRadius: "4px",
                      backgroundColor: "#4CAF50",
                      color: "white",
                      border: "none",
                      transition: "background-color 0.3s",
                    }}
                    onMouseOver={(e) => (e.currentTarget.style.backgroundColor = "#45a049")}
                    onMouseOut={(e) => (e.currentTarget.style.backgroundColor = "#4CAF50")}
                  >
                    {connectionStatus[device.address] || "Connect"}
                  </button>
                </td>
              </tr>
            ))
          ) : (
            <tr>
              <td colSpan={3} style={{ padding: "10px", textAlign: "center" }}>
                {isScanning ? "Scanning for devices..." : "No devices found."}
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

export default App;