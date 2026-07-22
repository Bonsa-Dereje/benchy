// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use sysinfo::{Disks, System};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ─────────────────────────────────────────────────────────────
// Reading the real machine's specs
// ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiskInfo {
    pub name: String,
    pub total_gb: f64,
    pub available_gb: f64,
    pub kind: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemSpecs {
    pub hostname: String,
    pub os: String,
    pub os_version: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub ram_total_gb: f64,
    pub ram_available_gb: f64,
    pub gpu_name: String,
    pub disks: Vec<DiskInfo>,
}

// Best-effort GPU detection. sysinfo doesn't expose GPU info, so we shell
// out to whatever each OS already has installed. This can come back empty
// on unusual setups (e.g. headless Linux with no lspci) — that's fine, the
// UI just shows "Unknown" and the estimator leans on CPU/RAM instead.
fn detect_gpu() -> String {
    #[cfg(target_os = "windows")]
    {
        if let Ok(out) = Command::new("wmic")
            .args(["path", "win32_VideoController", "get", "name"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(name) = text
                .lines()
                .map(|l| l.trim())
                .find(|l| !l.is_empty() && !l.eq_ignore_ascii_case("name"))
            {
                return name.to_string();
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = Command::new("system_profiler")
            .arg("SPDisplaysDataType")
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if let Some(rest) = line.trim().strip_prefix("Chipset Model:") {
                    return rest.trim().to_string();
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(out) = Command::new("lspci").output() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = text
                .lines()
                .find(|l| l.contains("VGA") || l.contains("3D controller"))
            {
                if let Some(idx) = line.find(':') {
                    let after = &line[idx + 1..];
                    if let Some(idx2) = after.find(':') {
                        return after[idx2 + 1..].trim().to_string();
                    }
                }
                return line.trim().to_string();
            }
        }
    }

    "Unknown / integrated graphics".to_string()
}

#[tauri::command]
fn get_system_specs() -> Result<SystemSpecs, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_brand = sys
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown CPU".to_string());

    let cpu_threads = sys.cpus().len().max(1);
    let cpu_cores = num_cpus::get_physical().max(1);

    let ram_total_gb = sys.total_memory() as f64 / 1_073_741_824.0;
    let ram_available_gb = sys.available_memory() as f64 / 1_073_741_824.0;

    let disks_list = Disks::new_with_refreshed_list();
    let disks = disks_list
        .iter()
        .map(|d| DiskInfo {
            name: d.name().to_string_lossy().to_string(),
            total_gb: (d.total_space() as f64 / 1_073_741_824.0 * 10.0).round() / 10.0,
            available_gb: (d.available_space() as f64 / 1_073_741_824.0 * 10.0).round() / 10.0,
            kind: format!("{:?}", d.kind()),
        })
        .collect();

    Ok(SystemSpecs {
        hostname: System::host_name().unwrap_or_else(|| "This PC".to_string()),
        os: System::name().unwrap_or_else(|| "Unknown OS".to_string()),
        os_version: System::os_version().unwrap_or_default(),
        cpu_brand,
        cpu_cores,
        cpu_threads,
        ram_total_gb: (ram_total_gb * 10.0).round() / 10.0,
        ram_available_gb: (ram_available_gb * 10.0).round() / 10.0,
        gpu_name: detect_gpu(),
        disks,
    })
}

// ─────────────────────────────────────────────────────────────
// Benchmark estimator — same idea as api/products.js's
// test-performance handler, ported to Rust and pointed at this
// machine's real specs instead of a marketplace listing.
// ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppScore {
    pub score: i32,
    pub verdict: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BenchmarkResponse {
    pub results: HashMap<String, AppScore>,
    // "ai" if Groq answered, "fallback" if we used the local heuristic
    // (no key set, or every model call failed)
    pub estimator: String,
}

const GROQ_ENDPOINT: &str = "https://api.groq.com/openai/v1/chat/completions";
const GROQ_MODEL: &str = "llama-3.3-70b-versatile";
const GROQ_FALLBACKS: [&str; 2] = ["llama-3.1-8b-instant", "openai/gpt-oss-20b"];

const PERF_SYSTEM_PROMPT: &str = "\
You are a PC performance estimator. Given one machine's real hardware specs \
and a list of apps/games/software the user wants to run, estimate — for EACH \
app — roughly what percentage of that software's performance headroom this \
machine can deliver, based on typical published system requirements and how \
this hardware generally performs in the real world.

Scoring guide (as a rough anchor, use your judgement in between):
- 0-20:  won't meet minimum requirements / unusably slow or won't run at all
- 21-45: runs, but below the software's comfortable/recommended requirements
  (choppy, laggy, forced to lowest settings, long render/export times)
- 46-70: meets recommended requirements — usable, decent everyday experience
- 71-90: comfortably exceeds recommended requirements — smooth
- 91-100: overkill for this software, maxes it out effortlessly

Consider CPU generation/class and thread count, RAM amount, storage type, and \
GPU (integrated vs dedicated, VRAM) together — a single weak component (e.g. \
integrated graphics for 3D rendering, or 8GB RAM for local LLMs) should \
meaningfully cap the score even if other specs are strong.

Return ONLY a valid JSON object, no markdown fences, no commentary, in \
exactly this shape:
{
  \"results\": {
    \"<app name exactly as given>\": {
      \"score\": <integer 0-100>,
      \"verdict\": \"<one short sentence, under 12 words, plain and specific>\"
    }
  }
}
Include every app that was given, and nothing else.";

fn build_spec_summary(specs: &SystemSpecs) -> String {
    let mut lines = vec![
        format!("Machine: {}", specs.hostname),
        format!("OS: {} {}", specs.os, specs.os_version),
        format!(
            "CPU: {} ({} physical cores / {} threads)",
            specs.cpu_brand, specs.cpu_cores, specs.cpu_threads
        ),
        format!(
            "RAM: {:.1} GB total, {:.1} GB currently available",
            specs.ram_total_gb, specs.ram_available_gb
        ),
        format!("GPU: {}", specs.gpu_name),
    ];
    if let Some(d) = specs.disks.first() {
        lines.push(format!("Primary storage: {:.0} GB ({})", d.total_gb, d.kind));
    }
    lines.join("\n")
}

fn strip_to_json(content: &str) -> String {
    let mut text = content.trim().to_string();
    if text.starts_with("```") {
        if let Some(rest) = text.splitn(3, "```").nth(1) {
            text = rest.trim().to_string();
            if let Some(stripped) = text.strip_prefix("json") {
                text = stripped.trim().to_string();
            }
        }
    }
    text
}

async fn call_groq(model: &str, api_key: &str, user_text: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 900,
        "response_format": { "type": "json_object" },
        "messages": [
            { "role": "system", "content": PERF_SYSTEM_PROMPT },
            { "role": "user", "content": user_text }
        ]
    });

    let resp = client
        .post(GROQ_ENDPOINT)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Groq HTTP {}: {}", status, text));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Unexpected Groq response shape: {}", json))
}

// Crude no-AI fallback so the UI always shows *something* if the key is
// missing or Groq is unreachable, instead of a hard error.
fn local_heuristic(specs: &SystemSpecs, apps: &[String]) -> HashMap<String, AppScore> {
    let ram = specs.ram_total_gb;
    let gpu_lower = specs.gpu_name.to_lowercase();
    let has_dedicated = !gpu_lower.contains("intel(r) uhd")
        && !gpu_lower.contains("intel iris")
        && !gpu_lower.contains("integrated")
        && !gpu_lower.contains("unknown");
    let base = ((ram / 32.0) * 45.0
        + (specs.cpu_threads as f64 / 24.0) * 35.0
        + if has_dedicated { 20.0 } else { 5.0 })
    .round()
    .clamp(10.0, 95.0) as i32;

    apps.iter()
        .map(|a| {
            (
                a.clone(),
                AppScore {
                    score: base,
                    verdict: "Estimated from specs only — AI estimator unavailable.".to_string(),
                },
            )
        })
        .collect()
}

#[tauri::command]
async fn benchmark_apps(
    specs: SystemSpecs,
    dept: Option<String>,
    apps: Vec<String>,
) -> Result<BenchmarkResponse, String> {
    if apps.is_empty() {
        return Err("apps must not be empty".to_string());
    }

    let api_key = std::env::var("GROQ_API_KEY").unwrap_or_default();
    if api_key.trim().is_empty() {
        return Ok(BenchmarkResponse {
            results: local_heuristic(&specs, &apps),
            estimator: "fallback".to_string(),
        });
    }

    let spec_summary = build_spec_summary(&specs);
    let mut user_text = format!("Machine specs:\n{}\n", spec_summary);
    if let Some(d) = &dept {
        user_text.push_str(&format!("\nUser's use case category: {}\n", d));
    }
    user_text.push_str("\nApps/software to evaluate:\n");
    for a in &apps {
        user_text.push_str(&format!("- {}\n", a));
    }

    let models: Vec<&str> = std::iter::once(GROQ_MODEL)
        .chain(GROQ_FALLBACKS.iter().copied())
        .collect();
    let mut last_err: Option<String> = None;

    for model in models {
        match call_groq(model, &api_key, &user_text).await {
            Ok(raw) => {
                let cleaned = strip_to_json(&raw);
                match serde_json::from_str::<serde_json::Value>(&cleaned) {
                    Ok(parsed) => {
                        if let Some(results_val) = parsed.get("results") {
                            if let Ok(mut results) =
                                serde_json::from_value::<HashMap<String, AppScore>>(
                                    results_val.clone(),
                                )
                            {
                                let missing: Vec<String> = apps
                                    .iter()
                                    .filter(|a| !results.contains_key(*a))
                                    .cloned()
                                    .collect();
                                if !missing.is_empty() {
                                    for (k, v) in local_heuristic(&specs, &missing) {
                                        results.insert(k, v);
                                    }
                                }
                                return Ok(BenchmarkResponse {
                                    results,
                                    estimator: "ai".to_string(),
                                });
                            }
                        }
                        last_err = Some("Missing \"results\" object in AI response".to_string());
                    }
                    Err(e) => last_err = Some(e.to_string()),
                }
            }
            Err(e) => {
                eprintln!("[benchmark_apps] model {} failed: {}", model, e);
                last_err = Some(e);
                continue;
            }
        }
    }

    eprintln!("[benchmark_apps] all Groq models failed: {:?}", last_err);
    Ok(BenchmarkResponse {
        results: local_heuristic(&specs, &apps),
        estimator: "fallback".to_string(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Loads GROQ_API_KEY from a .env file next to the app during dev.
    // In a packaged build there's no guaranteed cwd, so for production set
    // GROQ_API_KEY as a real environment variable at launch, or bundle a
    // resource file and point dotenvy at it with `dotenvy::from_path(..)`.
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_specs,
            benchmark_apps
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
