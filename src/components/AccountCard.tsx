import type { UsageSummary } from "../types";

interface AccountCardProps {
  account: {
    id: string;
    name: string;
    email: string;
    avatar_url: string;
    plan_type: string;
    created_at: number;
    is_current?: boolean;
    token_expired_at?: string | null;
  };
  usage: UsageSummary | null;
  selected: boolean;
  onSelect: (id: string) => void;
  onContextMenu: (e: React.MouseEvent, id: string) => void;
}

export function AccountCard({ account, usage, selected, onSelect, onContextMenu }: AccountCardProps) {
  const getUsageLevel = (used: number, limit: number) => {
    if (limit === 0) return "low";
    const percent = (used / limit) * 100;
    if (percent >= 80) return "high";
    if (percent >= 50) return "medium";
    return "low";
  };

  const formatDate = (timestamp: number) => {
    if (!timestamp) return "-";
    const date = new Date(timestamp * 1000);
    const year = date.getFullYear();
    const month = date.getMonth() + 1;
    const day = date.getDate();
    return `${year}/${month}/${day}`;
  };

  const formatCreatedDate = (timestamp: number) => {
    if (!timestamp) return "-";
    const date = new Date(timestamp * 1000);
    const year = date.getFullYear();
    const month = date.getMonth() + 1;
    const day = date.getDate();
    return `${year}/${month}/${day}`;
  };

  const totalUsed = usage ? usage.fast_request_used + usage.extra_fast_request_used : 0;
  const totalLimit = usage ? usage.fast_request_limit + usage.extra_fast_request_limit : 0;
  const totalLeft = usage ? usage.fast_request_left + usage.extra_fast_request_left : 0;
  const usagePercent = totalLimit > 0 ? Math.round((totalUsed / totalLimit) * 100) : 0;
  const usageLevel = getUsageLevel(totalUsed, totalLimit);

  const getTokenStatus = (): "normal" | "expiring" | "expired" | "unknown" => {
    if (!account.token_expired_at) return "unknown";
    const expiry = new Date(account.token_expired_at).getTime();
    if (isNaN(expiry)) return "unknown";
    const now = Date.now();
    if (expiry < now) return "expired";
    if (expiry - now < 3600000) return "expiring"; // < 1小时
    return "normal";
  };

  const tokenStatus = getTokenStatus();

  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(account.email || account.name);
  };

  return (
    <div
      className={`account-card ${selected ? "selected" : ""} ${account.is_current ? "current" : ""}`}
      onClick={() => onSelect(account.id)}
      onContextMenu={(e) => onContextMenu(e, account.id)}
    >
      <div className="card-header">
        <div className="card-checkbox" onClick={(e) => e.stopPropagation()}>
          <input
            type="checkbox"
            checked={selected}
            onChange={() => onSelect(account.id)}
          />
        </div>

        <div className="card-avatar">
          {account.avatar_url ? (
            <img src={account.avatar_url} alt={account.name} />
          ) : (
            <div className="avatar-placeholder">
              {(account.email || account.name).charAt(0).toUpperCase()}
            </div>
          )}
        </div>

        <div className="card-info">
          <div className="card-email">
            <span className="email-text">{account.email || account.name}</span>
            <button
              className="copy-btn"
              onClick={handleCopy}
              title="复制邮箱"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
              </svg>
            </button>
          </div>
          <div className="card-name">Trae 账号</div>
        </div>

        <div className={`card-status ${tokenStatus === "expired" ? "expired" : tokenStatus === "expiring" ? "expiring" : "normal"}`}>
          <span className="status-indicator"></span>
          {tokenStatus === "expired" ? "已过期" : tokenStatus === "expiring" ? "即将过期" : "正常"}
        </div>
      </div>

      <div className="card-tags">
        <span className="tag plan">{usage?.plan_type || account.plan_type || "Free"}</span>
        {usage && usage.extra_fast_request_limit > 0 && (
          <span className="tag extra">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 12l-1.41-1.41L13 16.17V4h-2v12.17l-5.58-5.59L4 12l8 8 8-8z" transform="rotate(180 12 12)"/>
            </svg>
            礼包
          </span>
        )}
        {account.is_current && (
          <span className="tag current">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
              <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
            </svg>
            当前使用
          </span>
        )}
      </div>

      <div className="card-usage">
        <div className="usage-header">
          <span className="usage-label">Fast Requests</span>
          <span className={`usage-percent ${usageLevel}`}>{usagePercent}%</span>
        </div>
        <div className="usage-bar">
          <div
            className={`usage-bar-fill ${usageLevel}`}
            style={{ width: `${Math.min(usagePercent, 100)}%` }}
          />
        </div>
        <div className="usage-numbers">
          <span className="usage-used">
            <strong>{Math.round(totalUsed)}</strong> / {totalLimit}
          </span>
          <span className="usage-left">剩余 {Math.round(totalLeft)}</span>
        </div>
      </div>

      <div className="card-meta">
        <span className="meta-item">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
            <line x1="16" y1="2" x2="16" y2="6"/>
            <line x1="8" y1="2" x2="8" y2="6"/>
            <line x1="3" y1="10" x2="21" y2="10"/>
          </svg>
          添加于 {formatCreatedDate(account.created_at)}
        </span>
        <span className="meta-item">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
          重置 {usage ? formatDate(usage.reset_time) : "-"}
        </span>
        {usage && usage.extra_expire_time > 0 && (
          <span className="meta-item warning">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M20 12v6a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-6M12 2v10M8 6l4-4 4 4"/>
            </svg>
            礼包到期 {formatDate(usage.extra_expire_time)}
          </span>
        )}
      </div>

      <div className="card-footer">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>
        </svg>
        右键查看更多操作
      </div>
    </div>
  );
}
