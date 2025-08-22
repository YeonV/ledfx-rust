// android-native/PermissionPlugin.kt
package com.blade.ledfx_rust

import android.Manifest
import android.content.pm.PackageManager
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

class PermissionsPlugin(private val activity: MainActivity) : Plugin(activity) {
    private val permissionLauncher: ActivityResultLauncher<String>
    private var pendingInvoke: Invoke? = null

    init {
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