package com.alvaro17f.recurrency

import android.os.Bundle
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import kotlin.math.roundToInt

class MainActivity : TauriActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
    }

    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)

        ViewCompat.setOnApplyWindowInsetsListener(webView) { _, insets ->
            val systemBars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
            val displayCutout = insets.getInsets(WindowInsetsCompat.Type.displayCutout())
            val imeVisible = insets.isVisible(WindowInsetsCompat.Type.ime())
            val imeHeight = insets.getInsets(WindowInsetsCompat.Type.ime()).bottom

            val density = resources.displayMetrics.density

            val safeTop = maxOf(systemBars.top, displayCutout.top)
            val safeBottom = maxOf(systemBars.bottom, displayCutout.bottom)
            val safeLeft = maxOf(systemBars.left, displayCutout.left)
            val safeRight = maxOf(systemBars.right, displayCutout.right)

            val computedBottom = if (imeVisible) imeHeight else safeBottom

            val script = """
                document.documentElement.style.setProperty('--safe-area-inset-top', '${(safeTop / density).roundToInt()}px');
                document.documentElement.style.setProperty('--safe-area-inset-bottom', '${(safeBottom / density).roundToInt()}px');
                document.documentElement.style.setProperty('--safe-area-inset-left', '${(safeLeft / density).roundToInt()}px');
                document.documentElement.style.setProperty('--safe-area-inset-right', '${(safeRight / density).roundToInt()}px');
                document.documentElement.style.setProperty('--safe-area-bottom-computed', '${(computedBottom / density).roundToInt()}px');
                document.documentElement.style.setProperty('--keyboard-visible', '${if (imeVisible) "1" else "0"}');
            """.trimIndent()

            webView.evaluateJavascript(script, null)
            insets
        }
    }
}