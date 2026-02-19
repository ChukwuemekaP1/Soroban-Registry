use anyhow::{Context, Result};
use colored::Colorize;
use serde_json::json;
use std::path::Path;
use crate::profiler;

pub async fn search(
    api_url: &str,
    query: &str,
    network: Option<&str>,
    verified_only: bool,
) -> Result<()> {
    let client = reqwest::Client::new();
    let mut url = format!("{}/api/contracts?query={}", api_url, query);

    if let Some(net) = network {
        url.push_str(&format!("&network={}", net));
    }
    if verified_only {
        url.push_str("&verified_only=true");
    }

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to search contracts")?;

    let data: serde_json::Value = response.json().await?;
    let items = data["items"].as_array().context("Invalid response")?;

    println!("\n{}", "Search Results:".bold().cyan());
    println!("{}", "=".repeat(80).cyan());

    if items.is_empty() {
        println!("{}", "No contracts found.".yellow());
        return Ok(());
    }

    for contract in items {
        let name = contract["name"].as_str().unwrap_or("Unknown");
        let contract_id = contract["contract_id"].as_str().unwrap_or("");
        let is_verified = contract["is_verified"].as_bool().unwrap_or(false);
        let network = contract["network"].as_str().unwrap_or("");

        println!("\n{} {}", "●".green(), name.bold());
        println!("  ID: {}", contract_id.bright_black());
        println!(
            "  Status: {} | Network: {}",
            if is_verified {
                "✓ Verified".green()
            } else {
                "○ Unverified".yellow()
            },
            network.bright_blue()
        );

        if let Some(desc) = contract["description"].as_str() {
            println!("  {}", desc.bright_black());
        }
    }

    println!("\n{}", "=".repeat(80).cyan());
    println!("Found {} contract(s)\n", items.len());

    Ok(())
}

pub async fn info(api_url: &str, contract_id: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/contracts/{}", api_url, contract_id);

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to fetch contract info")?;

    if !response.status().is_success() {
        anyhow::bail!("Contract not found");
    }

    let contract: serde_json::Value = response.json().await?;

    println!("\n{}", "Contract Information:".bold().cyan());
    println!("{}", "=".repeat(80).cyan());

    println!("\n{}: {}", "Name".bold(), contract["name"].as_str().unwrap_or("Unknown"));
    println!("{}: {}", "Contract ID".bold(), contract["contract_id"].as_str().unwrap_or(""));
    println!("{}: {}", "Network".bold(), contract["network"].as_str().unwrap_or("").bright_blue());
    
    let is_verified = contract["is_verified"].as_bool().unwrap_or(false);
    println!(
        "{}: {}",
        "Verified".bold(),
        if is_verified {
            "✓ Yes".green()
        } else {
            "○ No".yellow()
        }
    );

    if let Some(desc) = contract["description"].as_str() {
        println!("\n{}: {}", "Description".bold(), desc);
    }

    if let Some(tags) = contract["tags"].as_array() {
        if !tags.is_empty() {
            print!("\n{}: ", "Tags".bold());
            for (i, tag) in tags.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{}", tag.as_str().unwrap_or("").bright_magenta());
            }
            println!();
        }
    }

    println!("\n{}", "=".repeat(80).cyan());
    println!();

    Ok(())
}

pub async fn publish(
    api_url: &str,
    contract_id: &str,
    name: &str,
    description: Option<&str>,
    network: &str,
    category: Option<&str>,
    tags: Vec<String>,
    publisher: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/contracts", api_url);

    let payload = json!({
        "contract_id": contract_id,
        "name": name,
        "description": description,
        "network": network,
        "category": category,
        "tags": tags,
        "publisher_address": publisher,
    });

    println!("\n{}", "Publishing contract...".bold().cyan());

    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .context("Failed to publish contract")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to publish: {}", error_text);
    }

    let contract: serde_json::Value = response.json().await?;

    println!("{}", "✓ Contract published successfully!".green().bold());
    println!("\n{}: {}", "Name".bold(), contract["name"].as_str().unwrap_or(""));
    println!("{}: {}", "ID".bold(), contract["contract_id"].as_str().unwrap_or(""));
    println!("{}: {}", "Network".bold(), contract["network"].as_str().unwrap_or("").bright_blue());
    println!();

    Ok(())
}

pub async fn list(api_url: &str, limit: usize, network: Option<&str>) -> Result<()> {
    let client = reqwest::Client::new();
    let mut url = format!("{}/api/contracts?page_size={}", api_url, limit);

    if let Some(net) = network {
        url.push_str(&format!("&network={}", net));
    }

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to list contracts")?;

    let data: serde_json::Value = response.json().await?;
    let items = data["items"].as_array().context("Invalid response")?;

    println!("\n{}", "Recent Contracts:".bold().cyan());
    println!("{}", "=".repeat(80).cyan());

    if items.is_empty() {
        println!("{}", "No contracts found.".yellow());
        return Ok(());
    }

    for (i, contract) in items.iter().enumerate() {
        let name = contract["name"].as_str().unwrap_or("Unknown");
        let contract_id = contract["contract_id"].as_str().unwrap_or("");
        let is_verified = contract["is_verified"].as_bool().unwrap_or(false);
        let network = contract["network"].as_str().unwrap_or("");

        println!(
            "\n{}. {} {}",
            i + 1,
            name.bold(),
            if is_verified { "✓".green() } else { "".normal() }
        );
        println!("   {} | {}", contract_id.bright_black(), network.bright_blue());
    }

    println!("\n{}", "=".repeat(80).cyan());
    println!();

    Ok(())
}

pub async fn profile(
    contract_path: &str,
    method: Option<&str>,
    output: Option<&str>,
    flamegraph: Option<&str>,
    compare: Option<&str>,
    show_recommendations: bool,
) -> Result<()> {
    let path = Path::new(contract_path);
    if !path.exists() {
        anyhow::bail!("Contract file not found: {}", contract_path);
    }

    println!("\n{}", "Profiling contract...".bold().cyan());
    println!("{}", "=".repeat(80).cyan());

    let mut profiler = profiler::Profiler::new();
    profiler::simulate_execution(path, method, &mut profiler)?;
    let profile_data = profiler.finish(contract_path.to_string(), method.map(|s| s.to_string()));

    println!("\n{}", "Profile Results:".bold().green());
    println!("Total Duration: {:.2}ms", profile_data.total_duration.as_secs_f64() * 1000.0);
    println!("Overhead: {:.2}%", profile_data.overhead_percent);
    println!("Functions Profiled: {}", profile_data.functions.len());

    let mut sorted_functions: Vec<_> = profile_data.functions.values().collect();
    sorted_functions.sort_by(|a, b| b.total_time.cmp(&a.total_time));

    println!("\n{}", "Top Functions:".bold());
    for (i, func) in sorted_functions.iter().take(10).enumerate() {
        println!(
            "{}. {} - {:.2}ms ({} calls, avg: {:.2}μs)",
            i + 1,
            func.name.bold(),
            func.total_time.as_secs_f64() * 1000.0,
            func.call_count,
            func.avg_time.as_secs_f64() * 1_000_000.0
        );
    }

    if let Some(output_path) = output {
        let json = serde_json::to_string_pretty(&profile_data)?;
        std::fs::write(output_path, json)
            .with_context(|| format!("Failed to write profile to: {}", output_path))?;
        println!("\n{} Profile exported to: {}", "✓".green(), output_path);
    }

    if let Some(flame_path) = flamegraph {
        profiler::generate_flame_graph(&profile_data, Path::new(flame_path))?;
        println!("{} Flame graph generated: {}", "✓".green(), flame_path);
    }

    if let Some(baseline_path) = compare {
        let baseline_json = std::fs::read_to_string(baseline_path)
            .with_context(|| format!("Failed to read baseline: {}", baseline_path))?;
        let baseline: profiler::ProfileData = serde_json::from_str(&baseline_json)?;

        let comparisons = profiler::compare_profiles(&baseline, &profile_data);

        println!("\n{}", "Comparison Results:".bold().yellow());
        for comp in comparisons.iter().take(10) {
            let sign = if comp.time_diff_ns > 0 { "+" } else { "" };
            println!(
                "{}: {} ({}{:.2}%, {:.2}ms → {:.2}ms)",
                comp.function.bold(),
                comp.status,
                sign,
                comp.time_diff_percent,
                comp.baseline_time.as_secs_f64() * 1000.0,
                comp.current_time.as_secs_f64() * 1000.0
            );
        }
    }

    if show_recommendations {
        let recommendations = profiler::generate_recommendations(&profile_data);
        println!("\n{}", "Recommendations:".bold().magenta());
        for (i, rec) in recommendations.iter().enumerate() {
            println!("{}. {}", i + 1, rec);
        }
    }

    println!("\n{}", "=".repeat(80).cyan());
    println!();

    Ok(())
}
