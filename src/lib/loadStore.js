import { writable } from 'svelte/store'

// Populated by src/routes/loadingScreen/+page.svelte once the real hardware
// specs + general-readiness benchmark have both come back. The root page
// (src/routes/+page.svelte) just reads these back out — it never calls
// invoke() itself anymore.
export const specsStore = writable(null)
export const specsErrorStore = writable('')
export const generalStore = writable({ stage: 'idle', results: {}, notes: {}, error: false, errorDetail: '' })
