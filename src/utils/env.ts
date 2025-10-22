export const env = {
  NODE_URL: (import.meta as any).env.VITE_VISION_NODE_URL ?? "http://127.0.0.1:7070",
  MARKET_URL: (import.meta as any).env.VITE_VISION_MARKET_URL ?? "http://127.0.0.1:8080",
  FEATURE_DEV_PANEL: ((import.meta as any).env.VITE_FEATURE_DEV_PANEL ?? "false") === "true",
}
