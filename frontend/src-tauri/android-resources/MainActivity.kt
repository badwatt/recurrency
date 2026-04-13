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
            injectSafeAreaInsets(webView, insets)
            insets
        }

        // Force initial inset dispatch after layout
        webView.post {
            ViewCompat.requestApplyInsets(webView)
        }
    }

    override fun onResume() {
        super.onResume()
        // Re-dispatch insets when returning to app
        val webView = findWebView()
        webView?.post {
            ViewCompat.requestApplyInsets(webView)
        }
    }

    private fun injectSafeAreaInsets(webView: WebView, insets: WindowInsetsCompat): WindowInsetsCompat {
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
        return insets
    }

    private fun findWebView(): WebView? {
        // Walk the view tree to find the WebView
        val rootView = window.decorView.rootView
        return findWebViewInView(rootView)
    }

    private fun findWebViewInView(view: android.view.View): WebView? {
        if (view is WebView) return view
        if (view is android.view.ViewGroup) {
            for (i in 0 until view.childCount) {
                val found = findWebViewInView(view.getChildAt(i))
                if (found != null) return found
            }
        }
        return null
    }
}