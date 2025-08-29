// src/tauri.d.ts

// This tells TypeScript that the global Window interface has an additional property.
interface Window {
	__TAURI_METADATA__: {
		__TAURI_PLATFORM__: 'windows' | 'macos' | 'linux' | 'android' | 'ios'
	}
}
