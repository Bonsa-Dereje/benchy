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
// Battery health — shells out to Windows' built-in
// `powercfg /batteryreport`, which dumps an HTML report to disk, then
// scrapes DESIGN CAPACITY / FULL CHARGE CAPACITY (and CYCLE COUNT, when
// present) out of it to compute how much the battery has degraded.
// Windows-only; other platforms get a clear error instead of a fake result.
// ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatteryReport {
    pub has_battery: bool,
    pub design_capacity_mwh: Option<u32>,
    pub full_charge_capacity_mwh: Option<u32>,
    // full_charge / design * 100 — "how much of the original tank is left"
    pub health_pct: Option<f64>,
    // 100 - health_pct — "how much has been lost"
    pub degradation_pct: Option<f64>,
    pub cycle_count: Option<u32>,
}

// Pulls the text content of the <td> immediately following the first <td>
// that contains `label`. The report's rows all look like:
//   <tr><td class="label">DESIGN CAPACITY</td><td align="right">57,420 mWh</td></tr>
// so "find the label, then the next <td>...</td> after it" is enough —
// no need for a full HTML parser for a report this simple/stable.
#[cfg(target_os = "windows")]
fn extract_field(html: &str, label: &str) -> Option<String> {
    let label_idx = html.find(label)?;
    let rest = &html[label_idx + label.len()..];
    let td_open_rel = rest.find("<td")?;
    let after_open = &rest[td_open_rel..];
    let gt_rel = after_open.find('>')?;
    let content_start = gt_rel + 1;
    let close_rel = after_open.find("</td>")?;
    if close_rel < content_start {
        return None;
    }
    Some(after_open[content_start..close_rel].trim().to_string())
}

// "57,420 mWh" -> 57420. Strips commas, "mWh", any stray non-digit chars.
#[cfg(target_os = "windows")]
fn parse_mwh(raw: &str) -> Option<u32> {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<u32>().ok()
    }
}

#[cfg(target_os = "windows")]
fn parse_battery_report(html: &str) -> BatteryReport {
    let design = extract_field(html, "DESIGN CAPACITY").and_then(|s| parse_mwh(&s));
    let full = extract_field(html, "FULL CHARGE CAPACITY").and_then(|s| parse_mwh(&s));
    let cycle_count = extract_field(html, "CYCLE COUNT").and_then(|s| parse_mwh(&s));

    let (health_pct, degradation_pct) = match (design, full) {
        (Some(d), Some(f)) if d > 0 => {
            let health = (f as f64 / d as f64) * 100.0;
            let health = (health * 10.0).round() / 10.0;
            let degraded = ((100.0 - health) * 10.0).round() / 10.0;
            (Some(health.max(0.0)), Some(degraded.max(0.0)))
        }
        _ => (None, None),
    };

    BatteryReport {
        has_battery: design.is_some() && full.is_some(),
        design_capacity_mwh: design,
        full_charge_capacity_mwh: full,
        health_pct,
        degradation_pct,
        cycle_count,
    }
}

#[tauri::command]
fn get_battery_report() -> Result<BatteryReport, String> {
    #[cfg(target_os = "windows")]
    {
        let report_path = std::env::temp_dir().join("battery-report.html");

        eprintln!(
            "[get_battery_report] running: powercfg /batteryreport /output {}",
            report_path.display()
        );

        let output = Command::new("powercfg")
            .args(["/batteryreport", "/output"])
            .arg(&report_path)
            .output()
            .map_err(|e| format!("failed to launch powercfg: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let msg = format!(
                "powercfg exited with an error: {}{}",
                stderr.trim(),
                if stderr.trim().is_empty() {
                    stdout.trim().to_string()
                } else {
                    String::new()
                }
            );
            eprintln!("[get_battery_report] ✗ {}", msg);
            return Err(msg);
        }

        let html = std::fs::read_to_string(&report_path).map_err(|e| {
            format!(
                "powercfg ran but the report at {} couldn't be read: {}",
                report_path.display(),
                e
            )
        })?;

        let report = parse_battery_report(&html);
        if !report.has_battery {
            eprintln!("[get_battery_report] no DESIGN CAPACITY / FULL CHARGE CAPACITY found — likely a desktop with no battery");
        } else {
            eprintln!(
                "[get_battery_report] ✓ design={:?} mWh, full={:?} mWh, health={:?}%",
                report.design_capacity_mwh, report.full_charge_capacity_mwh, report.health_pct
            );
        }
        Ok(report)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Battery report is only available on Windows (uses powercfg).".to_string())
    }
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
    // Human-readable reason the AI estimator wasn't used, when estimator ==
    // "fallback". None when estimator == "ai". Surfaced in the UI and
    // logged to stderr so a broken key / deprecated model / rate limit is
    // obvious instead of silently showing made-up-looking numbers.
    pub error_detail: Option<String>,
}

// Routed through our own Vercel proxy (api/chat.js) instead of calling Groq
// directly. The proxy holds GROQ_API_KEY server-side, so the client no
// longer needs a local key at all — it just forwards the same OpenAI-shaped
// chat-completions body and Vercel attaches the Groq Authorization header.
const GROQ_ENDPOINT: &str = "https://groq-api-sand.vercel.app/api/chat";
// NOTE: llama-3.3-70b-versatile and llama-3.1-8b-instant were deprecated by
// Groq (announced 2026-06-17) and now return a `model_decommissioned` error
// on every call — that's what was making this look like "AI unavailable"
// even with a valid key. Pointed at the models Groq recommends instead:
// https://console.groq.com/docs/deprecations
const GROQ_MODEL: &str = "openai/gpt-oss-120b";
const GROQ_FALLBACKS: [&str; 2] = ["openai/gpt-oss-20b", "qwen/qwen3.6-27b"];

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

async fn call_groq(model: &str, user_text: &str) -> Result<String, String> {
    eprintln!("[benchmark_apps] → calling proxy model={}", model);

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
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            let msg = format!("network error calling proxy: {}", e);
            eprintln!("[benchmark_apps] ✗ {} ({})", msg, model);
            msg
        })?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        let msg = format!("Groq HTTP {} ({}): {}", status.as_u16(), model, text);
        eprintln!("[benchmark_apps] ✗ {}", msg);
        return Err(msg);
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| {
        let msg = format!("couldn't parse Groq response as JSON: {}", e);
        eprintln!("[benchmark_apps] ✗ {} ({})", msg, model);
        msg
    })?;

    match json["choices"][0]["message"]["content"].as_str() {
        Some(s) => {
            eprintln!("[benchmark_apps] ✓ {} responded ({} chars)", model, s.len());
            Ok(s.to_string())
        }
        None => {
            let msg = format!("unexpected Groq response shape: {}", json);
            eprintln!("[benchmark_apps] ✗ {} ({})", msg, model);
            Err(msg)
        }
    }
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

    eprintln!(
        "[benchmark_apps] evaluating {} app(s) via proxy",
        apps.len()
    );

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
        match call_groq(model, &user_text).await {
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
                                eprintln!(
                                    "[benchmark_apps] ✓ using AI results from {} for {} app(s)",
                                    model,
                                    apps.len()
                                );
                                return Ok(BenchmarkResponse {
                                    results,
                                    estimator: "ai".to_string(),
                                    error_detail: None,
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

    let reason = last_err.unwrap_or_else(|| "all Groq models failed for an unknown reason".to_string());
    eprintln!("[benchmark_apps] ✗ all Groq models failed, falling back: {}", reason);
    Ok(BenchmarkResponse {
        results: local_heuristic(&specs, &apps),
        estimator: "fallback".to_string(),
        error_detail: Some(reason),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpgradeItem {
    pub title: String,
    pub component: String,
    pub reason: String,
    pub estimated_boost: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpgradeAdviceResponse {
    pub category: String,
    pub summary: String,
    pub upgrades: Vec<UpgradeItem>,
    pub estimator: String,
    pub error_detail: Option<String>,
}

const UPGRADE_SYSTEM_PROMPT: &str = r#"You are a PC hardware optimization expert.
Given a user's machine specs and a target use-case category to improve (e.g. Graphic Design, Video Editing, Gaming, etc.), analyze the system hardware bottlenecks and recommend 2 to 4 concrete, realistic hardware upgrades.

You MUST reply with ONLY a JSON object (no markdown formatting, no markdown code blocks) matching this exact schema:
{
  "summary": "<1-2 sentences explaining why current specs bottleneck performance for this category>",
  "upgrades": [
    {
      "title": "<Actionable title e.g. 'Add 8GB RAM (Upgrade to 16GB)'>",
      "component": "<RAM | GPU | CPU | Storage>",
      "reason": "<1 concise sentence on how this upgrade helps>",
      "estimated_boost": "<Estimated score or speed boost, e.g. '+25% speed'>"
    }
  ]
}
"#;

fn local_upgrade_heuristic(specs: &SystemSpecs, category: &str) -> UpgradeAdviceResponse {
    let ram = specs.ram_total_gb;
    let gpu_lower = specs.gpu_name.to_lowercase();
    let has_dedicated_gpu = !gpu_lower.contains("intel(r) uhd")
        && !gpu_lower.contains("intel iris")
        && !gpu_lower.contains("integrated")
        && !gpu_lower.contains("unknown");

    let mut upgrades = Vec::new();

    if ram < 16.0 {
        upgrades.push(UpgradeItem {
            title: "Add 8GB RAM (Upgrade to 16GB)".to_string(),
            component: "RAM".to_string(),
            reason: format!("Your current {:.1}GB RAM limits multitasking and project asset caching.", ram),
            estimated_boost: "+30% speed & stability".to_string(),
        });
    } else if ram < 32.0 && (category.contains("Video") || category.contains("3D") || category.contains("LLM") || category.contains("Graphic")) {
        upgrades.push(UpgradeItem {
            title: "Upgrade to 32GB High-Speed RAM".to_string(),
            component: "RAM".to_string(),
            reason: "High-resolution media editing and 3D scenes perform significantly smoother with 32GB RAM.".to_string(),
            estimated_boost: "+20% render responsiveness".to_string(),
        });
    }

    if !has_dedicated_gpu {
        upgrades.push(UpgradeItem {
            title: "Upgrade to Dedicated GPU (6GB+ VRAM)".to_string(),
            component: "GPU".to_string(),
            reason: format!("Integrated graphics ({}) lacks hardware acceleration for heavy workflows.", specs.gpu_name),
            estimated_boost: "+45% graphics rendering score".to_string(),
        });
    } else if category.contains("LLM") || category.contains("3D") || category.contains("Gaming") {
        upgrades.push(UpgradeItem {
            title: "Upgrade to High-VRAM GPU (12GB+ VRAM)".to_string(),
            component: "GPU".to_string(),
            reason: "Demanding 3D rendering and local AI models require expanded GPU VRAM memory.".to_string(),
            estimated_boost: "+35% AI & 3D speed".to_string(),
        });
    }

    if let Some(disk) = specs.disks.first() {
        if disk.kind.to_lowercase().contains("hdd") || disk.available_gb < 50.0 {
            upgrades.push(UpgradeItem {
                title: "Add 1TB High-Speed NVMe SSD".to_string(),
                component: "Storage".to_string(),
                reason: "Faster disk read/write speeds eliminate file loading & project cache lag.".to_string(),
                estimated_boost: "+40% asset load speed".to_string(),
            });
        }
    }

    if upgrades.is_empty() || specs.cpu_cores < 8 {
        upgrades.push(UpgradeItem {
            title: "Upgrade CPU (8+ Cores / 16 Threads)".to_string(),
            component: "CPU".to_string(),
            reason: format!("Your {} CPU ({} cores) handles multi-threaded processing.", specs.cpu_brand, specs.cpu_cores),
            estimated_boost: "+25% multi-threaded speed".to_string(),
        });
    }

    UpgradeAdviceResponse {
        category: category.to_string(),
        summary: format!("Based on your specs ({:.0}GB RAM, {}), here are the recommended upgrades to improve {} performance.", 
            ram, specs.gpu_name, category),
        upgrades,
        estimator: "fallback".to_string(),
        error_detail: None,
    }
}

#[tauri::command]
async fn get_upgrade_advice(
    specs: SystemSpecs,
    category: String,
) -> Result<UpgradeAdviceResponse, String> {
    let spec_summary = build_spec_summary(&specs);
    let user_text = format!("Machine specs:\n{}\n\nTarget category to improve: {}\nProvide hardware upgrade recommendations.", spec_summary, category);

    let models: Vec<&str> = std::iter::once(GROQ_MODEL)
        .chain(GROQ_FALLBACKS.iter().copied())
        .collect();

    for model in models {
        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "model": model,
            "max_tokens": 800,
            "response_format": { "type": "json_object" },
            "messages": [
                { "role": "system", "content": UPGRADE_SYSTEM_PROMPT },
                { "role": "user", "content": user_text }
            ]
        });

        if let Ok(resp) = client.post(GROQ_ENDPOINT).json(&body).send().await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                        let cleaned = strip_to_json(content);
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&cleaned) {
                            let summary = parsed.get("summary").and_then(|s| s.as_str()).unwrap_or("").to_string();
                            if let Some(upgrades_arr) = parsed.get("upgrades").and_then(|u| u.as_array()) {
                                let mut upgrades = Vec::new();
                                for item in upgrades_arr {
                                    if let (Some(t), Some(c), Some(r), Some(b)) = (
                                        item.get("title").and_then(|v| v.as_str()),
                                        item.get("component").and_then(|v| v.as_str()),
                                        item.get("reason").and_then(|v| v.as_str()),
                                        item.get("estimated_boost").and_then(|v| v.as_str()),
                                    ) {
                                        upgrades.push(UpgradeItem {
                                            title: t.to_string(),
                                            component: c.to_string(),
                                            reason: r.to_string(),
                                            estimated_boost: b.to_string(),
                                        });
                                    }
                                }
                                if !upgrades.is_empty() {
                                    return Ok(UpgradeAdviceResponse {
                                        category,
                                        summary,
                                        upgrades,
                                        estimator: "ai".to_string(),
                                        error_detail: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(local_upgrade_heuristic(&specs, &category))
}

fn load_env() {
    let _ = dotenvy::dotenv();
    let _ = dotenvy::from_filename("src-tauri/.env");
    let _ = dotenvy::from_filename(".env");

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(mut dir) = exe_path.parent() {
            loop {
                let env_file = dir.join(".env");
                if env_file.exists() {
                    let _ = dotenvy::from_path(&env_file);
                }
                let src_tauri_env = dir.join("src-tauri").join(".env");
                if src_tauri_env.exists() {
                    let _ = dotenvy::from_path(&src_tauri_env);
                }
                match dir.parent() {
                    Some(parent) => dir = parent,
                    None => break,
                }
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    load_env();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_specs,
            benchmark_apps,
            get_battery_report,
            get_upgrade_advice
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}