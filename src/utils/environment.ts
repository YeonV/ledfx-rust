import { invoke } from '@tauri-apps/api/core'

let isDevelopment = false
let hasChecked = false

// This async function checks the backend and caches the result.
export const checkEnvironment = async () => {
	if (hasChecked) return isDevelopment
	isDevelopment = await invoke('is_dev')
	hasChecked = true
	return isDevelopment
}

// This provides synchronous access after the initial check.
export const isDev = () => {
	if (!hasChecked) {
		console.warn('isDev() called before checkEnvironment() has completed. Assuming production.')
		return false
	}
	return isDevelopment
}
