import axios from 'axios'

const DEFAULT_BASE = (import.meta as any).env?.VITE_VISION_NODE_URL || 'http://127.0.0.1:7070'

function baseUrl() {
  return (localStorage.getItem('vision.node.url') || DEFAULT_BASE)
}

async function withRetry<T>(fn: () => Promise<T>, retries = 1): Promise<T> {
  try {
    return await fn()
  } catch (err) {
    if (retries <= 0) throw err
    // retry once after short delay for 500/timeouts
    await new Promise((res) => setTimeout(res, 300))
    return withRetry(fn, retries - 1)
  }
}

export async function getStatus(): Promise<{ up: boolean; info?: any }> {
  return withRetry(async () => {
    const res = await axios.get(`${baseUrl()}/status`, { timeout: 3000 })
    return { up: true, info: res.data }
  }, 1).catch(() => ({ up: false }))
}

export async function getSupply(): Promise<any> {
  return withRetry(async () => {
    const res = await axios.get(`${baseUrl()}/supply`, { timeout: 3000 })
    return res.data
  }, 1)
}

export async function getVault(): Promise<any> {
  return withRetry(async () => {
    const res = await axios.get(`${baseUrl()}/vault`, { timeout: 3000 })
    return res.data
  }, 1)
}

export async function getLatestReceipts(): Promise<any[]> {
  return withRetry(async () => {
    const res = await axios.get(`${baseUrl()}/receipts/latest`, { timeout: 3000 })
    return Array.isArray(res.data) ? res.data : []
  }, 1)
}
