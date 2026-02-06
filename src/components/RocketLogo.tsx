export function RocketLogo() {
  return (
    <svg viewBox="0 0 48 48" fill="none" className="h-full w-full">
      <defs>
        <linearGradient id="logoGradient" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#818cf8" />
          <stop offset="50%" stopColor="#a855f7" />
          <stop offset="100%" stopColor="#ec4899" />
        </linearGradient>
      </defs>
      <rect width="48" height="48" rx="12" fill="url(#logoGradient)" />
      {/* 火箭主体 */}
      <path d="M24 10c-2 4-3 8-3 12 0 4 1 7 3 10 2-3 3-6 3-10 0-4-1-8-3-12z" fill="white" fillOpacity="0.95" />
      {/* 火箭左翼 */}
      <path d="M21 22c-3 1-5 3-6 5 2 1 4 1 6 0v-5z" fill="white" fillOpacity="0.85" />
      {/* 火箭右翼 */}
      <path d="M27 22c3 1 5 3 6 5-2 1-4 1-6 0v-5z" fill="white" fillOpacity="0.85" />
      {/* 火箭窗户 */}
      <circle cx="24" cy="18" r="2.5" fill="url(#logoGradient)" />
      {/* 火焰 */}
      <path d="M22 32c0 2 1 4 2 5 1-1 2-3 2-5-0.5 0.5-1.5 1-2 1s-1.5-0.5-2-1z" fill="#fbbf24" />
      <path d="M23 32c0 1.5 0.5 3 1 4 0.5-1 1-2.5 1-4-0.3 0.3-0.7 0.5-1 0.5s-0.7-0.2-1-0.5z" fill="#f97316" />
    </svg>
  );
}
