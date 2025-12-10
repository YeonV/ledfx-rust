// android-native/PermissionPlugin.kt
package com.blade.ledfx_rust

import android.Manifest
import android.app.Activity
import android.content.pm.PackageManager
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import app.tauri.plugin.JSObject

@TauriPlugin
class PermissionsPlugin(private val activity: Activity) : Plugin(activity) {
    private val permissionLauncher: ActivityResultLauncher<String>
    private var pendingInvoke: Invoke? = null

    init {
        if (activity is androidx.activity.ComponentActivity) {
            permissionLauncher = activity.registerForActivityResult(
                ActivityResultContracts.RequestPermission()
            ) { isGranted ->
                pendingInvoke?.let {
                    if (isGranted) {
                        it.resolve()
                    } else {
                        it.reject("Permission was denied by the user.")
                    }
                    pendingInvoke = null
                }
            }
        } else {
            throw IllegalStateException("Activity must be ComponentActivity")
        }
    }

    @Command
    fun requestRecordAudioPermission(invoke: Invoke) {
        if (ContextCompat.checkSelfPermission(activity, Manifest.permission.RECORD_AUDIO)
            == PackageManager.PERMISSION_GRANTED
        ) {
            invoke.resolve()
        } else {
            pendingInvoke = invoke
            permissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
        }
    }
}