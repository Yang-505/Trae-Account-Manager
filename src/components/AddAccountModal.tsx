import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import * as api from "../api";

interface AddAccountModalProps {
  isOpen: boolean;
  onClose: () => void;
  onAdd: (token: string, cookies?: string) => Promise<void>;
  onToast?: (type: "success" | "error" | "warning" | "info", message: string) => void;
  onAccountAdded?: () => void;
}

type AddMode = "manual" | "trae-ide" | "browser";

export function AddAccountModal({ isOpen, onClose, onAdd, onToast, onAccountAdded }: AddAccountModalProps) {
  const [mode, setMode] = useState<AddMode>("trae-ide");
  const [tokenInput, setTokenInput] = useState("");
  const [cookiesInput, setCookiesInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [browserLoginStarted, setBrowserLoginStarted] = useState(false);

  // 监听浏览器登录事件
  useEffect(() => {
    const unlistenSuccess = listen<string>("login-success", (event) => {
      onToast?.("success", `浏览器登录成功: ${event.payload}`);
      onAccountAdded?.();
      setBrowserLoginStarted(false);
      handleCloseInternal();
    });

    const unlistenFailed = listen<string>("login-failed", (event) => {
      setError(event.payload || "登录失败");
      setBrowserLoginStarted(false);
    });

    const unlistenCancelled = listen("login-cancelled", () => {
      setBrowserLoginStarted(false);
    });

    return () => {
      unlistenSuccess.then((fn) => fn());
      unlistenFailed.then((fn) => fn());
      unlistenCancelled.then((fn) => fn());
    };
  }, []);

  if (!isOpen) return null;

  // 从输入中提取 Token（优化：增强验证和清理）
  const extractToken = (input: string): string | null => {
    // 清理输入：移除首尾空白和潜在的危险字符
    const trimmed = input.trim().replace(/[\r\n\t]/g, '');

    // 情况1: 直接是 JWT Token (以 eyJ 开头)
    if (trimmed.startsWith("eyJ")) {
      // 验证 JWT 格式：必须有三个部分，用点分隔
      const parts = trimmed.split('.');
      if (parts.length === 3 && parts.every(part => /^[A-Za-z0-9_-]+$/.test(part))) {
        return trimmed;
      }
    }

    // 情况2: 是 JSON 响应，尝试解析
    try {
      const json = JSON.parse(trimmed);

      // GetUserToken 接口的响应格式
      if (json.Result?.Token && typeof json.Result.Token === 'string') {
        return validateAndCleanToken(json.Result.Token);
      }

      // 可能是其他格式
      if (json.token && typeof json.token === 'string') {
        return validateAndCleanToken(json.token);
      }
      if (json.Token && typeof json.Token === 'string') {
        return validateAndCleanToken(json.Token);
      }
    } catch {
      // 不是有效的 JSON，继续尝试其他方式
    }

    // 情况3: 尝试用正则提取 Token
    const tokenMatch = trimmed.match(/"Token"\s*:\s*"(eyJ[^"]+)"/);
    if (tokenMatch && tokenMatch[1]) {
      return validateAndCleanToken(tokenMatch[1]);
    }

    // 情况4: 尝试提取任何 eyJ 开头的字符串
    const jwtMatch = trimmed.match(/eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+/);
    if (jwtMatch && jwtMatch[0]) {
      return validateAndCleanToken(jwtMatch[0]);
    }

    return null;
  };

  // 验证并清理 Token
  const validateAndCleanToken = (token: string): string | null => {
    const cleaned = token.trim();
    const parts = cleaned.split('.');

    // JWT 必须有三个部分
    if (parts.length !== 3) {
      return null;
    }

    // 每个部分必须是有效的 Base64URL 字符
    if (!parts.every(part => /^[A-Za-z0-9_-]+$/.test(part))) {
      return null;
    }

    return cleaned;
  };

  // 读取 Trae IDE 账号
  const handleReadTraeAccount = async () => {
    setLoading(true);
    setError("");

    try {
      const account = await api.readTraeAccount();
      if (account) {
        onToast?.("success", `成功从 Trae IDE 读取账号: ${account.email}`);
        onAccountAdded?.();
        handleCloseInternal();
      } else {
        setError("未找到 Trae IDE 登录账号或账号已存在");
      }
    } catch (err: any) {
      setError(err.message || "读取 Trae IDE 账号失败");
    } finally {
      setLoading(false);
    }
  };

  // 手动添加账号
  const handleManualSubmit = async (e?: React.FormEvent) => {
    e?.preventDefault();
    if (!tokenInput.trim()) {
      setError("请输入 Token 或 API 响应");
      return;
    }

    setLoading(true);
    setError("");

    try {
      const token = extractToken(tokenInput);

      if (!token) {
        setError("无法识别 Token，请确保输入正确的 Token 或 GetUserToken 接口响应");
        setLoading(false);
        return;
      }

      // 清理 Cookies（如果有）
      const cookies = cookiesInput.trim() || undefined;
      await onAdd(token, cookies);
      setTokenInput("");
      setCookiesInput("");
      onClose();
    } catch (err: any) {
      setError(err.message || "添加账号失败");
    } finally {
      setLoading(false);
    }
  };

  // 浏览器登录
  const handleBrowserLogin = async () => {
    setLoading(true);
    setError("");
    setBrowserLoginStarted(true);

    try {
      await api.startBrowserLogin();
    } catch (err: any) {
      setError(err.message || "打开登录窗口失败");
      setBrowserLoginStarted(false);
    } finally {
      setLoading(false);
    }
  };

  const handleCloseInternal = () => {
    setError("");
    setTokenInput("");
    setCookiesInput("");
    setBrowserLoginStarted(false);
    setMode("trae-ide");
    onClose();
  };

  return (
    <div className="modal-overlay" onClick={handleCloseInternal}>
      <div className="modal-content add-account-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header-fixed">
          <h2>添加账号</h2>
          <button className="modal-close-btn" onClick={handleCloseInternal}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" width="20" height="20">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>

        <div className="modal-body-scrollable">
          {/* 添加方式选择 */}
          <div className="add-mode-tabs">
            <button
              className={`mode-tab ${mode === "trae-ide" ? "active" : ""}`}
              onClick={() => setMode("trae-ide")}
              disabled={loading}
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
              </svg>
              从 Trae IDE 读取
            </button>
            <button
              className={`mode-tab ${mode === "browser" ? "active" : ""}`}
              onClick={() => setMode("browser")}
              disabled={loading}
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="10"/>
                <line x1="2" y1="12" x2="22" y2="12"/>
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
              </svg>
              浏览器登录
            </button>
            <button
              className={`mode-tab ${mode === "manual" ? "active" : ""}`}
              onClick={() => setMode("manual")}
              disabled={loading}
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
              </svg>
              手动输入 Token
            </button>
          </div>

          {mode === "trae-ide" ? (
            /* Trae IDE 读取模式 */
            <div className="trae-ide-mode">
              <div className="mode-description-simple">
                <div className="mode-icon">
                  <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                    <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
                    <polyline points="3.27 6.96 12 12.01 20.73 6.96"/>
                    <line x1="12" y1="22.08" x2="12" y2="12"/>
                  </svg>
                </div>
                <h3>自动读取本地 Trae IDE 账号</h3>
                <p>系统将自动读取本地 Trae IDE 客户端当前登录的账号信息</p>
              </div>

              {error && <div className="error-message">{error}</div>}
            </div>
          ) : mode === "browser" ? (
            /* 浏览器登录模式 */
            <div className="trae-ide-mode">
              <div className="mode-description-simple">
                <div className="mode-icon">
                  <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                    <circle cx="12" cy="12" r="10"/>
                    <line x1="2" y1="12" x2="22" y2="12"/>
                    <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                  </svg>
                </div>
                <h3>浏览器授权登录</h3>
                <p>将打开一个登录窗口，在其中登录 trae.ai 账号，系统将自动提取 Cookies 并添加账号</p>
                {browserLoginStarted && (
                  <p style={{ color: "var(--color-warning, #f0a030)", marginTop: "8px" }}>
                    登录窗口已打开，请在窗口中完成登录...
                  </p>
                )}
              </div>

              {error && <div className="error-message">{error}</div>}
            </div>
          ) : (
            /* 手动输入模式 */
            <div className="manual-mode">
              {/* Token 输入 */}
              <div className="form-section">
                <label className="form-label">
                  Token <span className="required">*</span>
                </label>
                <textarea
                  value={tokenInput}
                  onChange={(e) => setTokenInput(e.target.value)}
                  placeholder='粘贴 Token 或 GetUserToken 接口响应...'
                  rows={4}
                  disabled={loading}
                />
                <div className="form-help">
                  <details>
                    <summary>如何获取 Token？</summary>
                    <ol>
                      <li>打开 <a href="https://www.trae.ai/account-setting#usage" target="_blank" rel="noopener noreferrer">trae.ai 账号设置页面</a> 并登录</li>
                      <li>按 <kbd>F12</kbd> 打开开发者工具，切换到 <strong>Network</strong> 标签</li>
                      <li>刷新页面，在请求列表中找到 <code>GetUserToken</code></li>
                      <li>点击该请求，在右侧 <strong>Response</strong> 标签中复制整个响应内容</li>
                    </ol>
                  </details>
                </div>
              </div>

              {/* Cookies 输入（可选） */}
              <div className="form-section">
                <label className="form-label">
                  Cookies <span className="optional">（可选）</span>
                </label>
                <textarea
                  value={cookiesInput}
                  onChange={(e) => setCookiesInput(e.target.value)}
                  placeholder='粘贴 Cookie 值（可选）...'
                  rows={3}
                  disabled={loading}
                />
                <div className="form-help">
                  <details>
                    <summary>如何获取 Cookies？</summary>
                    <ol>
                      <li>在上面获取 Token 的同一个页面</li>
                      <li>在 <strong>Network</strong> 标签中点击任意请求</li>
                      <li>在右侧 <strong>Headers</strong> 中找到 <code>Cookie</code> 字段</li>
                      <li>复制整个 Cookie 值（很长的一串）</li>
                    </ol>
                  </details>
                </div>
              </div>

              {error && <div className="error-message">{error}</div>}
            </div>
          )}
        </div>

        <div className="modal-actions-fixed">
          <button type="button" onClick={handleCloseInternal} disabled={loading}>
            取消
          </button>
          {mode === "trae-ide" ? (
            <button
              type="button"
              className="primary"
              onClick={handleReadTraeAccount}
              disabled={loading}
            >
              {loading ? "读取中..." : "读取本地账号"}
            </button>
          ) : mode === "browser" ? (
            <button
              type="button"
              className="primary"
              onClick={handleBrowserLogin}
              disabled={loading || browserLoginStarted}
            >
              {browserLoginStarted ? "等待登录中..." : loading ? "打开中..." : "打开登录窗口"}
            </button>
          ) : (
            <button
              type="button"
              className="primary"
              onClick={() => handleManualSubmit()}
              disabled={loading}
            >
              {loading ? "添加中..." : "添加账号"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
