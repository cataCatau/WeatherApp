# [🌦️Rust Weather Dashboard](https://d3on2ox9rpesz1.cloudfront.net)

A high-performance, cross-platform Weather Dashboard (Native Desktop + WebAssembly) deployed on AWS Global Infrastructure. Built with **Rust** and **egui**, designed for speed, low resource usage, and responsiveness.

<img width="1919" height="985" alt="image" src="https://github.com/user-attachments/assets/208be9dc-c0a1-41a0-ae18-46bfa8b53aa8" />



## 🚀 Overview

Unlike resource-heavy web or Electron-based weather apps, this dashboard compiles to native machine code, offering instant startup times and a minimal memory footprint. It leverages Rust's asynchronous capabilities to fetch data concurrently, ensuring a fluid user interface without freezing.

## ✨ Key Features

* **📍 Global Location Search:** Geocoding integration to find weather for any city.
* **⚡ Async Data Fetching:** Simultaneously retrieves Weather and Air Quality data using concurrent tasks.
* **📊 Detailed Metrics:**
    * **Current Conditions:** Temperature, Wind, Pressure, Precipitation.
    * **Hourly Forecast:** Horizontal scrollable view for the next 24 hours.
    * **Daily Forecast:** 7-day outlook with Min/Max temps, Sunrise time, and UV Index.
    * **Air Quality Index (AQI):** Visual breakdown of PM2.5, CO, NO2, SO2, and Ozone levels.
* **⭐ Persistent Favorites:** Save your favorite cities locally (stored in `favorites.json`).
* **🎨 Custom UI:** Dark mode aesthetic with semi-transparent cards, dynamic color-coded progress bars, and custom emoji rendering.


## 🛠️ Tech Stack

* **Language:** [Rust](https://www.rust-lang.org/) 🦀
* **GUI Library:** [egui](https://github.com/emilk/egui) (Immediate Mode GUI) via `eframe`.
* **Async Runtime:** [Tokio](https://tokio.rs/) (for non-blocking I/O).
* **WASM Toolchain:** [Trunk](https://trunkrs.dev/) (for building and serving WebAssembly).
* **API Provider:** [Open-Meteo](https://open-meteo.com/) (No API key required).
* **Cloud Infrastructure:** AWS (Amazon S3 for storage, CloudFront for global CDN caching).
* **Infrastructure as Code (IaC):** Terraform.
* **CI/CD:** GitHub Actions (Automated build & deployment).

## ⚙️ Architecture & Implementation Details

The application follows a modular architecture to separate concerns, fully adapted for both native and cloud environments:

* **State Management (`app.rs`):** Implements a state machine (`Search`, `Loading`, `Result`, `Error`) to manage the UI flow.
* **Concurrent Networking (`api.rs`):** Uses `tokio::join!` (on desktop) and `wasm_bindgen_futures` (on web) to parallelize the Geocoding, Weather, and Air Quality requests, effectively halving the wait time for the user.
* **Smart Persistence & Conditional Compilation:** Uses `#[cfg(target_arch = "wasm32")]` to handle environments differently. It uses local file I/O (`favorites.json`) for the desktop app, while falling back gracefully in the sandboxed WebAssembly environment.
* **Asset Optimization:** Large assets (like external emoji fonts) are dynamically managed or excluded from the web build to ensure a lightning-fast download time for the `.wasm` binary.
* **Cloud Architecture & CI/CD:** The web version is continuously deployed using **GitHub Actions**. Upon pushing to the `main` branch, the pipeline automatically compiles the Rust code to WASM, syncs the artifacts to an **AWS S3** bucket, and invalidates the **AWS CloudFront** cache for immediate, zero-downtime global updates.

## 📦 How to Run

Ensure you have Rust and Cargo installed.

**Clone the repository:**
    ```bash
    git clone [https://github.com/yourusername/weather-dashboard.git](https://github.com/yourusername/weather-dashboard.git)
    cd weather-dashboard
    ```

### Option A: Native Desktop App (Recommended for local use)
Run the application natively on your OS for maximum performance and local storage capabilities:
```bash
cargo run --release
```
### Option B: Local Web Server (WASM)
To test the WebAssembly build locally, make sure you have Trunk installed (cargo install trunk), then run:
```bash
trunk serve
```

### Option C: Cloud Deployment (CI/CD)
The project includes a fully automated GitHub Actions workflow (.github/workflows/deploy.yml). Any push to the main branch will automatically:

1.Build the WASM artifacts.
2.Deploy to AWS S3.
3.Invalidate the CloudFront CDN cache.
(Note: Requires AWS credentials configured in GitHub Secrets).

## 📂 Project Structure

```text
src/
├── main.rs        # Entry point, window configuration, and web-runner setup.
├── app.rs         # Application logic, update loop, and screen rendering.
├── api.rs         # Network requests and JSON deserialization models.
├── favorites.rs   # Logic for saving/loading favorite cities.
└── ui.rs          # Custom UI styles, cards, and utility functions.
.github/
└── workflows/
    └── deploy.yml # GitHub Actions CI/CD pipeline for AWS deployment.
```
## 👤 Author
**Catalin Tarca**

**University: Alexandru Ioan Cuza University, Faculty of Computer Science**

**GitHub: cataCatau**
