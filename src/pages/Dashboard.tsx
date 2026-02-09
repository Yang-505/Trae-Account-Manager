import { memo } from 'react';
import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer } from 'recharts';
import type { UsageSummary } from "../types";
import { UsageEvents } from "../components/UsageEvents";

interface DashboardProps {
  accounts: Array<{
    id: string;
    name: string;
    email: string;
    usage?: UsageSummary | null;
    is_current?: boolean;
  }>;
}

export const Dashboard = memo(function Dashboard({ accounts }: DashboardProps) {
  const totalAccounts = accounts.length;

  // 合并所有统计计算为一次遍历，提升性能
  const stats = accounts.reduce((acc, a) => {
    if (a.usage) {
      // 统计活跃账号
      if (a.usage.fast_request_left > 0) {
        acc.activeAccounts++;
      }

      // 累加使用量、配额和剩余量
      acc.totalUsed += a.usage.fast_request_used + a.usage.extra_fast_request_used;
      acc.totalLimit += a.usage.fast_request_limit + a.usage.extra_fast_request_limit;
      acc.totalLeft += a.usage.fast_request_left + a.usage.extra_fast_request_left;

      // 统计套餐分布
      const planType = a.usage.plan_type || 'Free';
      acc.quotaMap.set(planType, (acc.quotaMap.get(planType) || 0) + 1);
    }
    return acc;
  }, {
    activeAccounts: 0,
    totalUsed: 0,
    totalLimit: 0,
    totalLeft: 0,
    quotaMap: new Map<string, number>()
  });

  const { activeAccounts, totalUsed, totalLimit, totalLeft, quotaMap } = stats;
  const usagePercent = totalLimit > 0 ? Math.round((totalUsed / totalLimit) * 100) : 0;

  const pieData = [
    { name: '已使用', value: totalUsed, color: '#0ea5e9' },
    { name: '剩余', value: totalLeft, color: '#e5e7eb' },
  ];

  // 将 Map 转换为数组
  const quotaData = Array.from(quotaMap.entries()).map(([name, value]) => ({ name, value }));

  const COLORS = ['#0ea5e9', '#06b6d4', '#22d3ee', '#f97316'];

  return (
    <div className="dashboard">
      <div className="dashboard-header">
        <div className="welcome-section">
          <h1>欢迎回来 👋</h1>
          <p>这是您的账号使用概览</p>
        </div>
        <div className="header-stats">
          <div className="quick-stat">
            <span className="quick-stat-value">{totalAccounts}</span>
            <span className="quick-stat-label">账号总数</span>
          </div>
          <div className="quick-stat">
            <span className="quick-stat-value success">{activeAccounts}</span>
            <span className="quick-stat-label">可用账号</span>
          </div>
        </div>
      </div>

      <div className="stats-row">
        <div className="stat-card gradient-purple">
          <div className="stat-card-content">
            <div className="stat-card-info">
              <span className="stat-card-label">总配额</span>
              <span className="stat-card-value">{totalLimit}</span>
              <span className="stat-card-change">Fast Requests</span>
            </div>
            <div className="stat-card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
              </svg>
            </div>
          </div>
        </div>

        <div className="stat-card gradient-blue">
          <div className="stat-card-content">
            <div className="stat-card-info">
              <span className="stat-card-label">已使用</span>
              <span className="stat-card-value">{Math.round(totalUsed)}</span>
              <span className="stat-card-change">{usagePercent}% 使用率</span>
            </div>
            <div className="stat-card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
              </svg>
            </div>
          </div>
        </div>

        <div className="stat-card gradient-green">
          <div className="stat-card-content">
            <div className="stat-card-info">
              <span className="stat-card-label">剩余可用</span>
              <span className="stat-card-value">{Math.round(totalLeft)}</span>
              <span className="stat-card-change">{100 - usagePercent}% 剩余</span>
            </div>
            <div className="stat-card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M20 6L9 17l-5-5"/>
              </svg>
            </div>
          </div>
        </div>

        <div className="stat-card gradient-orange">
          <div className="stat-card-content">
            <div className="stat-card-info">
              <span className="stat-card-label">平均使用</span>
              <span className="stat-card-value">{totalAccounts > 0 ? Math.round(totalUsed / totalAccounts) : 0}</span>
              <span className="stat-card-change">每账号</span>
            </div>
            <div className="stat-card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="10"/>
                <path d="M12 6v6l4 2"/>
              </svg>
            </div>
          </div>
        </div>
      </div>

      <div className="charts-grid-2col">
        <div className="chart-card">
          <div className="chart-header">
            <h3>使用量分布</h3>
            <span className="chart-badge">{usagePercent}%</span>
          </div>
          <div className="chart-body pie-chart-container">
            <ResponsiveContainer width="100%" height={220}>
              <PieChart>
                <Pie
                  data={pieData}
                  cx="50%"
                  cy="50%"
                  innerRadius={70}
                  outerRadius={90}
                  paddingAngle={2}
                  dataKey="value"
                >
                  {pieData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip />
              </PieChart>
            </ResponsiveContainer>
            <div className="pie-center-text">
              <span className="pie-value">{Math.round(totalLeft)}</span>
              <span className="pie-label">剩余</span>
            </div>
          </div>
          <div className="chart-legend">
            <div className="legend-item">
              <span className="legend-dot" style={{ background: '#0ea5e9' }}></span>
              <span>已使用 ({Math.round(totalUsed)})</span>
            </div>
            <div className="legend-item">
              <span className="legend-dot" style={{ background: '#e5e7eb' }}></span>
              <span>剩余 ({Math.round(totalLeft)})</span>
            </div>
          </div>
        </div>

        <div className="chart-card">
          <div className="chart-header">
            <h3>套餐分布</h3>
          </div>
          <div className="chart-body">
            {quotaData.length > 0 ? (
              <ResponsiveContainer width="100%" height={220}>
                <PieChart>
                  <Pie
                    data={quotaData}
                    cx="50%"
                    cy="50%"
                    outerRadius={80}
                    dataKey="value"
                    label={({ name, percent }) => `${name || ''} ${((percent || 0) * 100).toFixed(0)}%`}
                  >
                    {quotaData.map((_, index) => (
                      <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                    ))}
                  </Pie>
                  <Tooltip />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <div className="chart-empty">暂无数据</div>
            )}
          </div>
        </div>
      </div>

      {accounts.length > 0 && (
        <>
          <UsageEvents accountId={accounts.find(a => a.is_current)?.id || accounts[0]?.id || ''} />

          <div className="accounts-preview">
            <div className="preview-header">
              <h3>账号概览</h3>
              <span className="preview-count">{accounts.length} 个账号</span>
            </div>
            <div className="preview-list">
              {accounts.slice(0, 4).map((account) => {
                const used = account.usage ? account.usage.fast_request_used + account.usage.extra_fast_request_used : 0;
                const limit = account.usage ? account.usage.fast_request_limit + account.usage.extra_fast_request_limit : 0;
                const percent = limit > 0 ? Math.round((used / limit) * 100) : 0;

                return (
                  <div key={account.id} className="preview-item">
                    <div className="preview-avatar">
                      {(account.email || account.name || '?').charAt(0).toUpperCase()}
                    </div>
                    <div className="preview-info">
                      <span className="preview-name">{account.email || account.name || 'Unknown'}</span>
                      <span className="preview-plan">{account.usage?.plan_type || 'Free'}</span>
                    </div>
                    <div className="preview-usage">
                      <div className="preview-progress">
                        <div
                          className="preview-progress-fill"
                          style={{
                            width: `${percent}%`,
                            background: percent > 80 ? '#ef4444' : percent > 50 ? '#f59e0b' : '#10b981'
                          }}
                        />
                      </div>
                      <span className="preview-percent">{percent}%</span>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </>
      )}

      {accounts.length === 0 && (
        <div className="dashboard-empty">
          <div className="empty-icon">📊</div>
          <h3>暂无账号数据</h3>
          <p>请先在"账号管理"中添加账号</p>
        </div>
      )}
    </div>
  );
});
