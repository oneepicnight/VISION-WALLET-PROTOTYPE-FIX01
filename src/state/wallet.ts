import { create } from 'zustand'
import { persist } from 'zustand/middleware'

interface Profile {
  handle: string
  address: string
  createdAt: number
}

interface Balances {
  LAND: number
  GAME: number
  CASH: number
}

interface NodeStatus {
  baseUrl: string
  status: 'down' | 'degraded' | 'up'
  lastSeen: number
}

interface WalletState {
  profile: Profile | null
  balances: Balances
  node: NodeStatus
  setProfile: (profile: Profile) => void
  setBalances: (balances: Partial<Balances>) => void
  setNode: (node: Partial<NodeStatus>) => void
  reset: () => void
}

const initialState = {
  profile: null,
  balances: { LAND: 0, GAME: 0, CASH: 0 },
  node: { 
    baseUrl: 'http://127.0.0.1:7070', 
    status: 'down' as const, 
    lastSeen: 0 
  }
}

export const useWalletStore = create<WalletState>()(
  persist(
    (set, get) => ({
      ...initialState,
      setProfile: (profile) => set({ profile }),
      setBalances: (newBalances) => set({ 
        balances: { ...get().balances, ...newBalances } 
      }),
      setNode: (nodeUpdate) => set({ 
        node: { ...get().node, ...nodeUpdate } 
      }),
      reset: () => set(initialState)
    }),
    {
      name: 'vision-wallet'
    }
  )
)