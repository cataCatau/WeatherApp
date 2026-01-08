# 🌦️ Rust Weather Dashboard

A high-performance, native desktop application for monitoring real-time weather and air quality. Built with **Rust** and **egui**, designed for speed, low resource usage, and responsiveness.

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
* **HTTP Client:** [Reqwest](https://docs.rs/reqwest/) (HTTPS requests).
* **Serialization:** [Serde](https://serde.rs/) & `serde_json`.
* **API Provider:** [Open-Meteo](https://open-meteo.com/) (No API key required).

## ⚙️ Architecture & Implementation Details

The application follows a modular architecture to separate concerns:

* **State Management (`app.rs`):** Implements a state machine (`Search`, `Loading`, `Result`, `Error`) to manage the UI flow.
* **Concurrent Networking (`api.rs`):** Uses `tokio::join!` to parallelize the Geocoding, Weather, and Air Quality requests, effectively halving the wait time for the user.
* **Persistent Storage (`favorites.rs`):** Handles file I/O to save user preferences on the disk.
* **UI Components (`ui.rs`):** Contains reusable widgets and helper functions for determining metric colors (e.g., changing AQI color from Green to Red based on pollution levels).
* **Font Embedding:** The application embeds the `seguiemj.ttf` font binary directly into the executable to ensure consistent Emoji rendering across different Windows systems.

## 📦 How to Run

Ensure you have Rust and Cargo installed.

1.  **Clone the repository:**
    ```bash
    git clone [https://github.com/yourusername/weather-dashboard.git](https://github.com/yourusername/weather-dashboard.git)
    cd weather-dashboard
    ```

2.  **Run in release mode (Recommended for performance):**
    ```bash
    cargo run --release
    ```

## 📂 Project Structure

```text
src/
├── main.rs       # Entry point, window configuration, and font loading.
├── app.rs        # Application logic, update loop, and screen rendering.
├── api.rs        # Network requests and JSON deserialization models.
├── favorites.rs  # Logic for saving/loading favorite cities to JSON.
└── ui.rs         # Custom UI styles, cards, and utility functions.
