use anyhow::{anyhow, Result};
use uuid::Uuid;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

/// Windows 注册表中 MachineGuid 的路径
#[cfg(target_os = "windows")]
const MACHINE_GUID_PATH: &str = r"SOFTWARE\Microsoft\Cryptography";
#[cfg(target_os = "windows")]
const MACHINE_GUID_KEY: &str = "MachineGuid";

/// 读取当前系统的 MachineGuid
#[cfg(target_os = "windows")]
pub fn get_machine_guid() -> Result<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey(MACHINE_GUID_PATH)
        .map_err(|e| anyhow!("无法打开注册表: {}", e))?;

    let guid: String = key.get_value(MACHINE_GUID_KEY)
        .map_err(|e| anyhow!("无法读取 MachineGuid: {}", e))?;

    Ok(guid)
}

/// 设置系统的 MachineGuid（需要管理员权限）
#[cfg(target_os = "windows")]
pub fn set_machine_guid(new_guid: &str) -> Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey_with_flags(MACHINE_GUID_PATH, KEY_SET_VALUE)
        .map_err(|e| anyhow!("无法打开注册表（需要管理员权限）: {}", e))?;

    key.set_value(MACHINE_GUID_KEY, &new_guid)
        .map_err(|e| anyhow!("无法设置 MachineGuid: {}", e))?;

    Ok(())
}

/// 生成新的 MachineGuid
pub fn generate_machine_guid() -> String {
    Uuid::new_v4().to_string()
}

/// 重置 MachineGuid 为新的随机值
#[cfg(target_os = "windows")]
pub fn reset_machine_guid() -> Result<String> {
    let new_guid = generate_machine_guid();
    set_machine_guid(&new_guid)?;
    Ok(new_guid)
}

/// 获取 Trae IDE 数据目录路径
#[cfg(target_os = "windows")]
fn get_trae_data_path() -> Result<PathBuf> {
    let appdata = std::env::var("APPDATA")
        .map_err(|_| anyhow!("无法获取 APPDATA 环境变量"))?;
    Ok(PathBuf::from(appdata).join("Trae"))
}

#[cfg(target_os = "macos")]
fn get_trae_data_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| anyhow!("无法获取 HOME 环境变量"))?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("Trae"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn get_trae_data_path() -> Result<PathBuf> {
    Err(anyhow!("此功能仅支持 Windows 和 macOS 系统"))
}

/// 读取 Trae IDE 的机器码
pub fn get_trae_machine_id() -> Result<String> {
    let trae_path = get_trae_data_path()?;
    let machine_id_path = trae_path.join("machineid");

    if !machine_id_path.exists() {
        return Err(anyhow!("Trae IDE 机器码文件不存在"));
    }

    let content = fs::read_to_string(&machine_id_path)
        .map_err(|e| anyhow!("读取 Trae 机器码失败: {}", e))?;

    Ok(content.trim().to_string())
}

/// 设置 Trae IDE 的机器码
pub fn set_trae_machine_id(new_id: &str) -> Result<()> {
    let trae_path = get_trae_data_path()?;
    let machine_id_path = trae_path.join("machineid");

    fs::write(&machine_id_path, new_id)
        .map_err(|e| anyhow!("写入 Trae 机器码失败: {}", e))?;

    Ok(())
}

/// 检查 Trae IDE 是否正在运行
#[cfg(target_os = "windows")]
pub fn is_trae_running() -> bool {
    let output = Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq Trae.exe", "/NH"])
        .output();

    match output {
        Ok(out) => {
            let result = String::from_utf8_lossy(&out.stdout);
            result.contains("Trae.exe")
        }
        Err(_) => false,
    }
}

#[cfg(target_os = "macos")]
pub fn is_trae_running() -> bool {
    // 使用 pgrep -f 匹配进程路径中包含 "Trae.app" 的进程
    Command::new("pgrep")
        .args(["-f", "Trae.app/Contents/MacOS"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// 关闭 Trae IDE 进程
#[cfg(target_os = "windows")]
pub fn kill_trae() -> Result<()> {
    if !is_trae_running() {
        println!("[INFO] Trae IDE 未运行");
        return Ok(());
    }

    println!("[INFO] 正在关闭 Trae IDE...");

    // 先尝试优雅关闭
    let _ = Command::new("taskkill")
        .args(["/IM", "Trae.exe"])
        .output();

    // 等待一小段时间
    std::thread::sleep(std::time::Duration::from_millis(500));

    // 如果还在运行，强制关闭
    if is_trae_running() {
        let output = Command::new("taskkill")
            .args(["/F", "/IM", "Trae.exe"])
            .output()
            .map_err(|e| anyhow!("关闭 Trae IDE 失败: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            if !err.is_empty() {
                return Err(anyhow!("关闭 Trae IDE 失败: {}", err));
            }
        }
    }

    // 等待进程完全退出
    std::thread::sleep(std::time::Duration::from_millis(1000));

    println!("[INFO] Trae IDE 已关闭");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn kill_trae() -> Result<()> {
    if !is_trae_running() {
        println!("[INFO] Trae IDE 未运行");
        return Ok(());
    }

    println!("[INFO] 正在关闭 Trae IDE...");

    // 使用 osascript 优雅关闭 Trae 应用
    let _ = Command::new("osascript")
        .args(["-e", "tell application \"Trae\" to quit"])
        .output();

    // 等待一小段时间
    std::thread::sleep(std::time::Duration::from_millis(1500));

    // 如果还在运行，使用 pkill 强制关闭
    if is_trae_running() {
        println!("[INFO] 优雅关闭失败，正在强制关闭...");
        let _ = Command::new("pkill")
            .args(["-9", "-f", "Trae.app/Contents/MacOS"])
            .output();
        
        // 再等待一下
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    if is_trae_running() {
        return Err(anyhow!("无法关闭 Trae IDE，请手动关闭后重试"));
    }

    println!("[INFO] Trae IDE 已关闭");
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn is_trae_running() -> bool {
    false
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn kill_trae() -> Result<()> {
    Err(anyhow!("此功能仅支持 Windows 和 macOS 系统"))
}

/// 获取 Trae IDE 配置文件路径
fn get_trae_config_path() -> Result<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("com", "sauce", "trae-auto")
        .ok_or_else(|| anyhow!("无法获取应用数据目录"))?;
    let config_dir = proj_dirs.config_dir();
    fs::create_dir_all(config_dir)?;
    Ok(config_dir.join("trae_path.txt"))
}

/// 获取保存的 Trae IDE 路径
pub fn get_saved_trae_path() -> Result<String> {
    let config_path = get_trae_config_path()?;
    if config_path.exists() {
        let path = fs::read_to_string(&config_path)?;
        let path = path.trim().to_string();
        if !path.is_empty() && PathBuf::from(&path).exists() {
            return Ok(path);
        }
    }
    Err(anyhow!("未设置 Trae IDE 路径"))
}

/// 保存 Trae IDE 路径
#[cfg(target_os = "windows")]
pub fn save_trae_path(path: &str) -> Result<()> {
    let exe_path = PathBuf::from(path);
    if !exe_path.exists() {
        return Err(anyhow!("指定的路径不存在"));
    }
    if !path.to_lowercase().ends_with(".exe") {
        return Err(anyhow!("请选择 Trae.exe 文件"));
    }
    let config_path = get_trae_config_path()?;
    fs::write(&config_path, path)?;
    println!("[INFO] 已保存 Trae IDE 路径: {}", path);
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn save_trae_path(path: &str) -> Result<()> {
    let app_path = PathBuf::from(path);
    if !app_path.exists() {
        return Err(anyhow!("指定的路径不存在"));
    }
    // macOS 应用是 .app bundle 目录
    if !path.to_lowercase().ends_with(".app") {
        return Err(anyhow!("请选择 Trae.app 应用程序"));
    }
    let config_path = get_trae_config_path()?;
    fs::write(&config_path, path)?;
    println!("[INFO] 已保存 Trae IDE 路径: {}", path);
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn save_trae_path(_path: &str) -> Result<()> {
    Err(anyhow!("此功能仅支持 Windows 和 macOS 系统"))
}

/// 自动扫描 Trae IDE 安装路径
#[cfg(target_os = "windows")]
pub fn scan_trae_path() -> Result<String> {
    Err(anyhow!("请手动设置 Trae IDE 路径"))
}

#[cfg(target_os = "macos")]
pub fn scan_trae_path() -> Result<String> {
    // 常见的 macOS 应用安装位置
    let possible_paths = [
        "/Applications/Trae.app",
        &format!("{}/Applications/Trae.app", std::env::var("HOME").unwrap_or_default()),
    ];
    
    for path in possible_paths {
        if PathBuf::from(path).exists() {
            return Ok(path.to_string());
        }
    }
    
    Err(anyhow!("未找到 Trae IDE，请手动设置路径"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn scan_trae_path() -> Result<String> {
    Err(anyhow!("此功能仅支持 Windows 和 macOS 系统"))
}

/// 打开 Trae IDE
#[cfg(target_os = "windows")]
pub fn open_trae() -> Result<()> {
    let trae_exe = match get_saved_trae_path() {
        Ok(path) => PathBuf::from(path),
        Err(_) => return Err(anyhow!("未设置 Trae IDE 路径，请在设置中配置")),
    };

    if !trae_exe.exists() {
        return Err(anyhow!("Trae IDE 路径无效，请在设置中重新配置"));
    }

    println!("[INFO] 正在启动 Trae IDE: {}", trae_exe.display());

    Command::new(&trae_exe)
        .spawn()
        .map_err(|e| anyhow!("启动 Trae IDE 失败: {}", e))?;

    println!("[INFO] Trae IDE 已启动");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn open_trae() -> Result<()> {
    let trae_app = match get_saved_trae_path() {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            // 尝试自动扫描
            match scan_trae_path() {
                Ok(path) => PathBuf::from(path),
                Err(_) => return Err(anyhow!("未设置 Trae IDE 路径，请在设置中配置")),
            }
        }
    };

    if !trae_app.exists() {
        return Err(anyhow!("Trae IDE 路径无效，请在设置中重新配置"));
    }

    println!("[INFO] 正在启动 Trae IDE: {}", trae_app.display());

    Command::new("open")
        .arg("-a")
        .arg(&trae_app)
        .spawn()
        .map_err(|e| anyhow!("启动 Trae IDE 失败: {}", e))?;

    println!("[INFO] Trae IDE 已启动");
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn open_trae() -> Result<()> {
    Err(anyhow!("此功能仅支持 Windows 和 macOS 系统"))
}

/// 账号登录信息结构（用于写入 Trae IDE）
#[derive(Debug, Clone)]
pub struct TraeLoginInfo {
    pub token: String,
    pub refresh_token: Option<String>,
    pub user_id: String,
    pub email: String,
    pub username: String,
    pub avatar_url: String,
    pub host: String,
    pub region: String,
}

/// 将账号登录信息写入 Trae IDE
pub fn write_trae_login_info(info: &TraeLoginInfo) -> Result<()> {
    let trae_path = get_trae_data_path()?;

    // 确保目录存在
    let storage_dir = trae_path.join("User").join("globalStorage");
    fs::create_dir_all(&storage_dir)
        .map_err(|e| anyhow!("创建目录失败: {}", e))?;

    let storage_path = storage_dir.join("storage.json");

    // 读取现有配置或创建新的
    let mut json: serde_json::Value = if storage_path.exists() {
        let content = fs::read_to_string(&storage_path)
            .map_err(|e| anyhow!("读取 storage.json 失败: {}", e))?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let obj = json.as_object_mut()
        .ok_or_else(|| anyhow!("storage.json 格式错误"))?;

    // 计算过期时间（14天后）
    let now = chrono::Utc::now();
    let expired_at = now + chrono::Duration::days(14);
    let refresh_expired_at = now + chrono::Duration::days(180);

    // 构建 host URL
    let host = if info.host.is_empty() {
        match info.region.to_uppercase().as_str() {
            "SG" => "https://api-sg-central.trae.ai",
            "CN" => "https://api.trae.com.cn",
            _ => "https://api-sg-central.trae.ai",
        }
    } else {
        &info.host
    };

    // 构建 iCubeAuthInfo
    let auth_info = serde_json::json!({
        "token": info.token,
        "refreshToken": info.refresh_token.clone().unwrap_or_default(),
        "expiredAt": expired_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        "refreshExpiredAt": refresh_expired_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        "tokenReleaseAt": now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        "userId": info.user_id,
        "host": host,
        "userRegion": {
            "region": info.region.to_uppercase(),
            "_aiRegion": info.region.to_uppercase()
        },
        "account": {
            "username": info.username,
            "iss": "",
            "iat": 0,
            "organization": "",
            "work_country": "",
            "email": info.email,
            "avatar_url": info.avatar_url,
            "description": "",
            "scope": "marscode",
            "loginScope": "trae",
            "storeCountryCode": "cn",
            "storeCountrySrc": "uid",
            "storeRegion": info.region.to_uppercase(),
            "userTag": "row"
        }
    });

    // 构建 iCubeEntitlementInfo
    let entitlement_info = serde_json::json!({
        "identityStr": "Free",
        "identity": 0,
        "isPayFreshman": false,
        "isSupportCommercialization": true,
        "hasPackage": false,
        "enableEntitlement": true,
        "detail": {
            "can_gen_solo_code": false,
            "fast_request_per": 1,
            "in_wait": false,
            "permission": 1,
            "toast_read": false,
            "toastRead": false,
            "canGenSoloCode": false,
            "fastRequestPer": 1,
            "inWaitlist": false
        }
    });

    // 写入登录信息
    obj.insert(
        "iCubeAuthInfo://icube.cloudide".to_string(),
        serde_json::Value::String(serde_json::to_string(&auth_info).unwrap())
    );
    obj.insert(
        "iCubeEntitlementInfo://icube.cloudide".to_string(),
        serde_json::Value::String(serde_json::to_string(&entitlement_info).unwrap())
    );

    // 写回文件
    let new_content = serde_json::to_string_pretty(&json)
        .map_err(|e| anyhow!("序列化 JSON 失败: {}", e))?;
    fs::write(&storage_path, new_content)
        .map_err(|e| anyhow!("写入 storage.json 失败: {}", e))?;

    println!("[INFO] 已写入 Trae IDE 登录信息: {}", info.email);
    Ok(())
}

/// 切换 Trae IDE 到指定账号（清除旧登录状态并写入新账号信息）
pub fn switch_trae_account(info: &TraeLoginInfo, machine_id: Option<&str>) -> Result<()> {
    // 0. 先关闭 Trae IDE
    kill_trae()?;

    let trae_path = get_trae_data_path()?;

    // 1. 设置机器码（如果提供则使用，否则生成新的）
    let new_machine_id = match machine_id {
        Some(mid) => mid.to_string(),
        None => generate_machine_guid(),
    };
    let machine_id_path = trae_path.join("machineid");
    fs::write(&machine_id_path, &new_machine_id)
        .map_err(|e| anyhow!("写入 Trae 机器码失败: {}", e))?;
    println!("[INFO] 已设置 Trae 机器码: {}", new_machine_id);

    // 2. 删除 state.vscdb 数据库（清除旧的登录缓存）
    let state_db_path = trae_path.join("User").join("globalStorage").join("state.vscdb");
    if state_db_path.exists() {
        let _ = fs::remove_file(&state_db_path);
        println!("[INFO] 已删除 state.vscdb");
    }

    // 3. 删除 state.vscdb.backup
    let state_db_backup_path = trae_path.join("User").join("globalStorage").join("state.vscdb.backup");
    if state_db_backup_path.exists() {
        let _ = fs::remove_file(&state_db_backup_path);
    }

    // 4. 清除 Local State
    let local_state_path = trae_path.join("Local State");
    if local_state_path.exists() {
        let _ = fs::remove_file(&local_state_path);
    }

    // 5. 清除 IndexedDB
    let indexed_db_path = trae_path.join("IndexedDB");
    if indexed_db_path.exists() {
        let _ = fs::remove_dir_all(&indexed_db_path);
    }

    // 6. 清除 Local Storage
    let local_storage_path = trae_path.join("Local Storage");
    if local_storage_path.exists() {
        let _ = fs::remove_dir_all(&local_storage_path);
    }

    // 7. 清除 Session Storage
    let session_storage_path = trae_path.join("Session Storage");
    if session_storage_path.exists() {
        let _ = fs::remove_dir_all(&session_storage_path);
    }

    // 8. 清除 Cookies
    let cookies_path = trae_path.join("Network").join("Cookies");
    if cookies_path.exists() {
        let _ = fs::remove_file(&cookies_path);
        println!("[INFO] 已清除 Cookies");
    }

    // 9. 清除 Cookies-journal
    let cookies_journal_path = trae_path.join("Network").join("Cookies-journal");
    if cookies_journal_path.exists() {
        let _ = fs::remove_file(&cookies_journal_path);
    }

    // 10. 更新 storage.json 中的 telemetry ID 并写入登录信息
    let storage_dir = trae_path.join("User").join("globalStorage");
    fs::create_dir_all(&storage_dir)
        .map_err(|e| anyhow!("创建目录失败: {}", e))?;
    let storage_path = storage_dir.join("storage.json");

    // 读取现有配置或创建新的
    let mut json: serde_json::Value = if storage_path.exists() {
        let content = fs::read_to_string(&storage_path)
            .map_err(|e| anyhow!("读取 storage.json 失败: {}", e))?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let obj = json.as_object_mut()
        .ok_or_else(|| anyhow!("storage.json 格式错误"))?;

    // 移除旧的登录信息
    obj.remove("iCubeAuthInfo://icube.cloudide");
    obj.remove("iCubeEntitlementInfo://icube.cloudide");
    obj.remove("iCubeServerData://icube.cloudide");
    obj.remove("iCubeAuthInfo://usertag");

    // 更新 telemetry ID
    let new_telemetry_id = format!("{:x}", md5_hash(&new_machine_id));
    obj.insert("telemetry.machineId".to_string(), serde_json::Value::String(new_telemetry_id));
    obj.insert("telemetry.sqmId".to_string(), serde_json::Value::String(format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase())));
    obj.insert("telemetry.devDeviceId".to_string(), serde_json::Value::String(Uuid::new_v4().to_string()));

    // 写回文件
    let new_content = serde_json::to_string_pretty(&json)
        .map_err(|e| anyhow!("序列化 JSON 失败: {}", e))?;
    fs::write(&storage_path, new_content)
        .map_err(|e| anyhow!("写入 storage.json 失败: {}", e))?;

    // 11. 写入新的登录信息
    write_trae_login_info(info)?;

    println!("[INFO] 已切换 Trae IDE 到账号: {}", info.email);

    // 12. 自动打开 Trae IDE
    if let Err(e) = open_trae() {
        println!("[WARN] 自动打开 Trae IDE 失败: {}", e);
    }

    Ok(())
}

/// 清除 Trae IDE 的登录状态（让 IDE 变成全新安装状态）
pub fn clear_trae_login_state() -> Result<()> {
    let trae_path = get_trae_data_path()?;

    // 1. 生成新的机器码
    let new_machine_id = generate_machine_guid();
    let machine_id_path = trae_path.join("machineid");
    fs::write(&machine_id_path, &new_machine_id)
        .map_err(|e| anyhow!("重置 Trae 机器码失败: {}", e))?;
    println!("[INFO] 已重置 Trae 机器码: {}", new_machine_id);

    // 2. 清除 storage.json 中的登录信息
    let storage_path = trae_path.join("User").join("globalStorage").join("storage.json");
    if storage_path.exists() {
        let content = fs::read_to_string(&storage_path)
            .map_err(|e| anyhow!("读取 storage.json 失败: {}", e))?;

        // 解析 JSON 并移除登录相关字段
        if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(obj) = json.as_object_mut() {
                // 移除登录相关字段
                obj.remove("iCubeAuthInfo://icube.cloudide");
                obj.remove("iCubeEntitlementInfo://icube.cloudide");
                obj.remove("iCubeServerData://icube.cloudide");
                obj.remove("iCubeAuthInfo://usertag");

                // 重置遥测 ID
                let new_telemetry_id = format!("{:x}", md5_hash(&new_machine_id));
                obj.insert("telemetry.machineId".to_string(), serde_json::Value::String(new_telemetry_id));
                obj.insert("telemetry.sqmId".to_string(), serde_json::Value::String(format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase())));
                obj.insert("telemetry.devDeviceId".to_string(), serde_json::Value::String(Uuid::new_v4().to_string()));

                // 写回文件
                let new_content = serde_json::to_string_pretty(&json)
                    .map_err(|e| anyhow!("序列化 JSON 失败: {}", e))?;
                fs::write(&storage_path, new_content)
                    .map_err(|e| anyhow!("写入 storage.json 失败: {}", e))?;
                println!("[INFO] 已清除 storage.json 中的登录信息");
            }
        }
    }

    // 3. 删除 state.vscdb 数据库（包含更多登录状态）
    let state_db_path = trae_path.join("User").join("globalStorage").join("state.vscdb");
    if state_db_path.exists() {
        fs::remove_file(&state_db_path)
            .map_err(|e| anyhow!("删除 state.vscdb 失败: {}", e))?;
        println!("[INFO] 已删除 state.vscdb");
    }

    // 4. 删除 state.vscdb.backup
    let state_db_backup_path = trae_path.join("User").join("globalStorage").join("state.vscdb.backup");
    if state_db_backup_path.exists() {
        let _ = fs::remove_file(&state_db_backup_path);
        println!("[INFO] 已删除 state.vscdb.backup");
    }

    // 5. 清除 Local State 中的加密密钥
    let local_state_path = trae_path.join("Local State");
    if local_state_path.exists() {
        let _ = fs::remove_file(&local_state_path);
        println!("[INFO] 已删除 Local State");
    }

    // 6. 清除 IndexedDB（可能包含登录缓存）
    let indexed_db_path = trae_path.join("IndexedDB");
    if indexed_db_path.exists() {
        let _ = fs::remove_dir_all(&indexed_db_path);
        println!("[INFO] 已清除 IndexedDB");
    }

    // 7. 清除 Local Storage
    let local_storage_path = trae_path.join("Local Storage");
    if local_storage_path.exists() {
        let _ = fs::remove_dir_all(&local_storage_path);
        println!("[INFO] 已清除 Local Storage");
    }

    // 8. 清除 Session Storage
    let session_storage_path = trae_path.join("Session Storage");
    if session_storage_path.exists() {
        let _ = fs::remove_dir_all(&session_storage_path);
        println!("[INFO] 已清除 Session Storage");
    }

    // 9. 清除 Cookies
    let cookies_path = trae_path.join("Network").join("Cookies");
    if cookies_path.exists() {
        let _ = fs::remove_file(&cookies_path);
        println!("[INFO] 已清除 Cookies");
    }

    Ok(())
}

/// 简单的 MD5 哈希（用于生成 telemetry.machineId 格式）
fn md5_hash(input: &str) -> u128 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let h1 = hasher.finish();

    let mut hasher2 = DefaultHasher::new();
    format!("{}{}", input, h1).hash(&mut hasher2);
    let h2 = hasher2.finish();

    ((h1 as u128) << 64) | (h2 as u128)
}

// macOS 平台实现
#[cfg(target_os = "macos")]
pub fn get_machine_guid() -> Result<String> {
    // 使用 ioreg 命令读取 IOPlatformUUID
    let output = Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
        .map_err(|e| anyhow!("执行 ioreg 失败: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // 解析 IOPlatformUUID
    for line in stdout.lines() {
        if line.contains("IOPlatformUUID") {
            // 格式: "IOPlatformUUID" = "XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX"
            if let Some(uuid) = line.split('"').nth(3) {
                return Ok(uuid.to_string());
            }
        }
    }
    
    Err(anyhow!("无法获取 IOPlatformUUID"))
}

#[cfg(target_os = "macos")]
pub fn set_machine_guid(_new_guid: &str) -> Result<()> {
    // macOS 无法修改系统 UUID
    Err(anyhow!("macOS 不支持修改系统机器码"))
}

#[cfg(target_os = "macos")]
pub fn reset_machine_guid() -> Result<String> {
    // macOS 无法重置系统 UUID
    Err(anyhow!("macOS 不支持重置系统机器码"))
}

// 非 Windows/macOS 平台的占位实现
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn get_machine_guid() -> Result<String> {
    Err(anyhow!("此功能仅支持 Windows 和 macOS 系统"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn set_machine_guid(_new_guid: &str) -> Result<()> {
    Err(anyhow!("此功能仅支持 Windows 和 macOS 系统"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn reset_machine_guid() -> Result<String> {
    Err(anyhow!("此功能仅支持 Windows 和 macOS 系统"))
}
