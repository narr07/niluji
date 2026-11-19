package com.narr07.niluji.siswa

import android.net.Uri
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.view.WindowManager
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback
import androidx.activity.enableEdgeToEdge
import androidx.appcompat.app.AlertDialog
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat

/**
 * Mode ujian NILUJI Siswa.
 *
 * - Tangkapan layar dan rekam layar diblokir (FLAG_SECURE) selama aplikasi terbuka.
 * - Layar tetap menyala.
 * - Saat portal ujian dari server terbuka: aplikasi meminta disematkan (screen pinning),
 *   bilah sistem disembunyikan, dan tombol kembali meminta konfirmasi sebelum keluar.
 *
 * Penyematan memakai fitur bawaan Android. Pada HP biasa Android menampilkan konfirmasi,
 * dan siswa yang menolak tetap bisa mengerjakan ujian tanpa disematkan.
 */
class MainActivity : TauriActivity() {
  private var webView: WebView? = null
  private var inExam = false
  private val handler = Handler(Looper.getMainLooper())

  // Tombol kembali diatur sendiri, bukan diteruskan ke riwayat WebView.
  override val handleBackNavigation: Boolean = false

  private val watchUrl = object : Runnable {
    override fun run() {
      syncExamMode()
      handler.postDelayed(this, 700)
    }
  }

  private val backCallback = object : OnBackPressedCallback(true) {
    override fun handleOnBackPressed() {
      if (inExam) {
        confirmLeaveExam()
      } else {
        isEnabled = false
        onBackPressedDispatcher.onBackPressed()
        isEnabled = true
      }
    }
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    window.addFlags(WindowManager.LayoutParams.FLAG_SECURE or WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
    onBackPressedDispatcher.addCallback(this, backCallback)
  }

  override fun onWebViewCreate(webView: WebView) {
    this.webView = webView
  }

  override fun onResume() {
    super.onResume()
    handler.post(watchUrl)
  }

  override fun onPause() {
    handler.removeCallbacks(watchUrl)
    super.onPause()
  }

  override fun onWindowFocusChanged(hasFocus: Boolean) {
    super.onWindowFocusChanged(hasFocus)
    if (hasFocus && inExam) hideSystemBars()
  }

  /** Portal ujian adalah halaman http dari server, bukan halaman bawaan aplikasi. */
  private fun isExamUrl(url: String?): Boolean {
    if (url.isNullOrBlank()) return false
    val uri = Uri.parse(url)
    val host = uri.host ?: return false
    return uri.scheme == "http" && !host.equals(APP_HOST, ignoreCase = true) && host != "localhost"
  }

  private fun syncExamMode() {
    val nowInExam = isExamUrl(webView?.url)
    if (nowInExam == inExam) return
    inExam = nowInExam
    if (inExam) {
      hideSystemBars()
      try {
        startLockTask()
      } catch (_: Exception) {
        // Penyematan tidak tersedia di perangkat ini. Ujian tetap bisa dikerjakan.
      }
    } else {
      leaveLockTask()
      showSystemBars()
    }
  }

  private fun confirmLeaveExam() {
    AlertDialog.Builder(this)
      .setTitle("Keluar dari ujian?")
      .setMessage("Jawaban yang sudah tersimpan tetap aman di server. Anda bisa masuk lagi dengan alamat dan token yang sama selama waktu ujian masih ada.")
      .setNegativeButton("Lanjutkan Ujian", null)
      .setPositiveButton("Keluar") { _, _ ->
        leaveLockTask()
        webView?.loadUrl(APP_HOME)
      }
      .show()
  }

  private fun leaveLockTask() {
    try {
      stopLockTask()
    } catch (_: Exception) {
      // Tidak sedang disematkan.
    }
  }

  private fun hideSystemBars() {
    val controller = WindowCompat.getInsetsController(window, window.decorView)
    controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
    controller.hide(WindowInsetsCompat.Type.systemBars())
  }

  private fun showSystemBars() {
    WindowCompat.getInsetsController(window, window.decorView).show(WindowInsetsCompat.Type.systemBars())
  }

  companion object {
    private const val APP_HOST = "tauri.localhost"
    private const val APP_HOME = "http://tauri.localhost/"
  }
}
