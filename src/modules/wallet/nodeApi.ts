import { env } from "../../utils/env"

const base = env.NODE_URL.replace(/\/$/, "")

async function getJSON<T>(path: string): Promise<T> {
  const r = await fetch(`${base}${path}`)
  if (!r.ok) throw new Error(`${path} ${r.status}`)
  return r.json()
}

export async function getStatus() { return getJSON<any>("/status") }
export async function getSupply() { return getJSON<any>("/supply") }
export async function getVault()  { return getJSON<any>("/vault") }
export async function getLatestReceipts() { return getJSON<any>("/receipts/latest") }
