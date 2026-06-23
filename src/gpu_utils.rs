use std::process::{Command, Stdio};

pub struct StaticGpuInfo {
    pub name: String,
    pub driver_version: String,
    pub memory_total: u64, // in MB
}

pub struct DynamicGpuStats {
    pub gpu_util: u32,
    pub mem_util: u32,
    pub dec_util: u32,
    pub enc_util: u32,
    pub mem_used: u64, // in MB
    pub temp: u32,
    pub power: f32, // in Watts
}

pub fn query_static_gpu_info() -> Result<StaticGpuInfo, String> {
    let output = Command::new("nvidia-smi")
        .args(&[
            "--query-gpu=name,driver_version,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .map_err(|e| format!("Failed to run nvidia-smi: {}", e))?;

    if !output.status.success() {
        return Err("nvidia-smi exited with error status".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().next().ok_or("Empty nvidia-smi output")?;
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 3 {
        return Err("Unexpected nvidia-smi static output format".to_string());
    }

    let memory_total = parts[parts.len() - 1]
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse memory: {}", e))?;

    let driver_version = parts[parts.len() - 2].trim().to_string();
    let name = parts[..parts.len() - 2].join(",").trim().to_string();

    Ok(StaticGpuInfo {
        name,
        driver_version,
        memory_total,
    })
}

pub fn query_dynamic_gpu_stats() -> Result<DynamicGpuStats, String> {
    let output = Command::new("nvidia-smi")
        .args(&[
            "--query-gpu=utilization.gpu,utilization.memory,utilization.decoder,utilization.encoder,memory.used,temperature.gpu,power.draw",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .map_err(|e| format!("Failed to run nvidia-smi: {}", e))?;

    if !output.status.success() {
        return Err("nvidia-smi exited with error status".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let line = text
        .lines()
        .next()
        .ok_or("Empty dynamic nvidia-smi output")?;
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 7 {
        return Err("Unexpected nvidia-smi dynamic output format".to_string());
    }

    let gpu_util = parts[0].trim().parse::<u32>().unwrap_or(0);
    let mem_util = parts[1].trim().parse::<u32>().unwrap_or(0);
    let dec_util = parts[2].trim().parse::<u32>().unwrap_or(0);
    let enc_util = parts[3].trim().parse::<u32>().unwrap_or(0);
    let mem_used = parts[4].trim().parse::<u64>().unwrap_or(0);
    let temp = parts[5].trim().parse::<u32>().unwrap_or(0);
    let power = parts[6].trim().parse::<f32>().unwrap_or(0.0);

    Ok(DynamicGpuStats {
        gpu_util,
        mem_util,
        dec_util,
        enc_util,
        mem_used,
        temp,
        power,
    })
}

pub fn check_nvidia_gpu() -> bool {
    #[cfg(windows)]
    {
        if unsafe { libloading::Library::new("nvcuda.dll") }.is_ok() {
            return true;
        }
    }

    let status = Command::new("nvidia-smi")
        .arg("-L")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if let Ok(s) = status {
        if s.success() {
            return true;
        }
    }

    false
}
