import axios from 'axios'

// Configure axios with default timeout
const api = axios.create({
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json'
  }
})

// Base URL management
export function getBaseUrl(): string {
  return localStorage.getItem('vision.node.url') || 'http://127.0.0.1:7070'
}

export function setBaseUrl(url: string): void {
  localStorage.setItem('vision.node.url', url)
}

// API endpoints
export async function pingStatus(): Promise<{ up: boolean; info: any }> {
  try {
    const response = await api.get(`${getBaseUrl()}/status`)
    return {
      up: true,
      info: response.data
    }
  } catch (error) {
    return {
      up: false,
      info: { error: error instanceof Error ? error.message : 'Network error' }
    }
  }
}

export async function getSupply(): Promise<{ total: string | number }> {
  try {
    const response = await api.get(`${getBaseUrl()}/supply`)
    return response.data
  } catch (error) {
    throw new Error(`Failed to get supply: ${error instanceof Error ? error.message : 'Network error'}`)
  }
}

export async function getLatestReceipts(): Promise<any[]> {
  try {
    const response = await api.get(`${getBaseUrl()}/receipts/latest`)
    return Array.isArray(response.data) ? response.data : []
  } catch (error) {
    throw new Error(`Failed to get receipts: ${error instanceof Error ? error.message : 'Network error'}`)
  }
}

export async function getBalance(address: string): Promise<{ LAND: number; GAME: number; CASH: number }> {
  try {
    const response = await api.get(`${getBaseUrl()}/balance/${address}`)
    return {
      LAND: response.data.LAND || 0,
      GAME: response.data.GAME || 0,
      CASH: response.data.CASH || 0
    }
  } catch (error) {
    // For now, return zeros if balance endpoint fails (stub until wired)
    console.warn('Balance fetch failed:', error)
    return { LAND: 0, GAME: 0, CASH: 0 }
  }
}

export interface Transaction {
  token: string
  to: string
  amount: number
  from: string
  nonce: number
}

export interface SignedTransaction {
  tx: Transaction
  sig: string
}

export async function submitTx(payload: SignedTransaction): Promise<{ ok: boolean; txid: string }> {
  try {
    const response = await api.post(`${getBaseUrl()}/tx/submit`, payload)
    return {
      ok: true,
      txid: response.data.txid || response.data.id || 'unknown'
    }
  } catch (error) {
    return {
      ok: false,
      txid: `error:${error instanceof Error ? error.message : 'Unknown error'}`
    }
  }
}