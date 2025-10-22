import React from 'react'
import { getVault } from '../modules/wallet'

export function VaultCard() {
  const [vault, setVault] = React.useState<any>(null)
  const [err, setErr] = React.useState<string | null>(null)

  React.useEffect(() => {
    let alive = true
    async function tick() {
      try {
        const v = await getVault()
        if (alive) setVault(v)
      } catch (e:any) { if (alive) setErr(String(e?.message ?? e)) }
    }
    tick()
    const id = setInterval(tick, 3000)
    return () => { alive = false; clearInterval(id) }
  }, [])

  if (err) return <div className="p-4 border rounded">Vault error: {err}</div>
  if (!vault) return <div className="p-4 border rounded">Loading vault…</div>

  return (
    <div className="p-4 border rounded shadow-sm">
      <div className="font-semibold mb-1">Vision Vault</div>
      <div className="text-sm opacity-80">Auto-updates every 3s</div>
      <pre className="text-xs mt-2 bg-black/5 p-2 rounded overflow-auto">{JSON.stringify(vault, null, 2)}</pre>
    </div>
  )
}
