export function LoadingScreen() {
  return (
    <div className="flex h-screen w-full items-center justify-center bg-background">
      <div className="flex flex-col items-center gap-6">
        <div className="relative">
          {/* Glow ring */}
          <div className="absolute inset-0 rounded-full bg-gradient-to-br from-blue-500 to-violet-500 opacity-20 blur-xl animate-pulse" />

          {/* Spinning ring */}
          <div className="relative h-16 w-16">
            <svg viewBox="0 0 64 64" className="h-full w-full animate-spin" style={{ animationDuration: "1.5s" }}>
              <circle
                cx="32"
                cy="32"
                r="28"
                fill="none"
                stroke="url(#loader-grad)"
                strokeWidth="4"
                strokeLinecap="round"
                strokeDasharray="120 60"
              />
              <defs>
                <linearGradient id="loader-grad" x1="0" y1="0" x2="1" y2="1">
                  <stop offset="0%" stopColor="#3B82F6" />
                  <stop offset="100%" stopColor="#8B5CF6" />
                </linearGradient>
              </defs>
            </svg>
          </div>
        </div>

        {/* Logo mark */}
        <div className="flex items-center gap-2 text-muted-foreground">
          <svg width="20" height="20" viewBox="0 0 512 512" className="opacity-50">
            <path
              d="M 256 90 A 166 166 0 1 1 138.6 138.6"
              fill="none"
              stroke="currentColor"
              strokeWidth="54"
              strokeLinecap="round"
            />
          </svg>
          <span className="text-sm font-medium tracking-wide">Recurrency</span>
        </div>
      </div>
    </div>
  );
}