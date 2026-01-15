import { invoke } from "@tauri-apps/api/core";
import type { Account, AccountBrief, UsageSummary, UsageEventsResponse } from "./types";

// 添加账号（通过 Cookies）
export async function addAccount(cookies: string): Promise<Account> {
  return invoke("add_account", { cookies });
}

// 添加账号（通过 Token，可选 Cookies）
export async function addAccountByToken(token: string, cookies?: string): Promise<Account> {
  return invoke("add_account_by_token", { token, cookies });
}

// 删除账号
export async function removeAccount(accountId: string): Promise<void> {
  return invoke("remove_account", { accountId });
}

// 获取所有账号
export async function getAccounts(): Promise<AccountBrief[]> {
  return invoke("get_accounts");
}

// 获取单个账号详情（包含 token）
export async function getAccount(accountId: string): Promise<Account> {
  return invoke("get_account", { accountId });
}

// 设置活跃账号
export async function setActiveAccount(accountId: string): Promise<void> {
  return invoke("switch_account", { accountId });
}

// 切换账号（设置活跃账号并更新机器码）
export async function switchAccount(accountId: string): Promise<void> {
  return invoke("switch_account", { accountId });
}

// 获取账号使用量
export async function getAccountUsage(accountId: string): Promise<UsageSummary> {
  return invoke("get_account_usage", { accountId });
}

// 更新账号 Token
export async function updateAccountToken(accountId: string, token: string): Promise<UsageSummary> {
  return invoke("update_account_token", { accountId, token });
}

// 刷新 Token
export async function refreshToken(accountId: string): Promise<void> {
  return invoke("refresh_token", { accountId });
}

// 更新 Cookies
export async function updateCookies(accountId: string, cookies: string): Promise<void> {
  return invoke("update_cookies", { accountId, cookies });
}

// 导出账号
export async function exportAccounts(): Promise<string> {
  return invoke("export_accounts");
}

// 导入账号
export async function importAccounts(data: string): Promise<number> {
  return invoke("import_accounts", { data });
}

// 获取使用事件
export async function getUsageEvents(
  accountId: string,
  startTime: number,
  endTime: number,
  pageNum: number = 1,
  pageSize: number = 20
): Promise<UsageEventsResponse> {
  return invoke("get_usage_events", {
    accountId,
    startTime,
    endTime,
    pageNum,
    pageSize
  });
}

// 从 Trae IDE 读取当前登录账号
export async function readTraeAccount(): Promise<Account | null> {
  return invoke("read_trae_account");
}

// ============ 机器码相关 API ============

// 获取当前系统机器码
export async function getMachineId(): Promise<string> {
  return invoke("get_machine_id");
}

// 重置系统机器码（生成新的随机机器码）
export async function resetMachineId(): Promise<string> {
  return invoke("reset_machine_id");
}

// 设置系统机器码为指定值
export async function setMachineId(machineId: string): Promise<void> {
  return invoke("set_machine_id", { machineId });
}

// 绑定账号机器码（保存当前系统机器码到账号）
export async function bindAccountMachineId(accountId: string): Promise<string> {
  return invoke("bind_account_machine_id", { accountId });
}

// ============ Trae IDE 机器码相关 API ============

// 获取 Trae IDE 的机器码
export async function getTraeMachineId(): Promise<string> {
  return invoke("get_trae_machine_id");
}

// 设置 Trae IDE 的机器码
export async function setTraeMachineId(machineId: string): Promise<void> {
  return invoke("set_trae_machine_id", { machineId });
}

// 清除 Trae IDE 登录状态（让 IDE 变成全新安装状态）
export async function clearTraeLoginState(): Promise<void> {
  return invoke("clear_trae_login_state");
}

// ============ Trae IDE 路径相关 API ============

// 获取保存的 Trae IDE 路径
export async function getTraePath(): Promise<string> {
  return invoke("get_trae_path");
}

// 设置 Trae IDE 路径
export async function setTraePath(path: string): Promise<void> {
  return invoke("set_trae_path", { path });
}

// 自动扫描 Trae IDE 路径
export async function scanTraePath(): Promise<string> {
  return invoke("scan_trae_path");
}

// ============ 礼包相关 API ============

// 领取礼包
export async function claimGift(accountId: string): Promise<void> {
  return invoke("claim_gift", { accountId });
}
