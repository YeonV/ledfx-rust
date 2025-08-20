// android-native/PermissionsPlugin.kt
package com.blade.ledfx_rust

import android.Manifest
import android.content.pm.PackageManager
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat
import app.tauri.plugin.Plugin
import app.tauri.annotation.Command
import app.tauri.plugin.Invoke

class PermissionsPlugin(private val activity: MainActivity) : Plugin(activity) {
    @Command
    fun requestRecordAudioPermission(invoke: Invoke) {
        if (ContextCompat.checkSelfPermission(activity, Manifest.permission.RECORD_AUDIO)
            == PackageManager.PERMISSION_GRANTED
        ) {
            invoke.resolve()
        } else {
            val launcher = activity.registerForActivityResult(
                ActivityResultContracts.RequestPermission()
            ) { isGranted ->
                if (isGranted) {
                    invoke.resolve()
                } else {
                    invoke.reject("Permission was denied by the user.")
                }
            }
            launcher.launch(Manifest.permission.RECORD_AUDIO)
        }
    }
}