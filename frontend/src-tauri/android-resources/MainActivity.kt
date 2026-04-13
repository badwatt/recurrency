package com.alvaro17f.recurrency

import android.os.Bundle
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import kotlin.math.roundToInt

class MainActivity : TauriActivity() {
    private var webViewRef: WebView? = null
    private var lastInsets: WindowInsetsCompat? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
    }

    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        webViewRef = webView

        // Set a WebViewClient wrapper to re-inject on every page load
        val existingClient = webView.webViewClient
        webView.webViewClient = object : WebViewClient() {
            override fun onPageFinished(view: WebView?, url: String?) {
                super.onPageFinished(view, url)
                // Re-inject CSS vars after page finishes loading
                lastInsets?.let { insets ->
                    view?.let { wv -> injectSafeAreaInsets(wv, insets) }
                }
            }
        }

        // Listen for inset changes and apply as native padding + CSS vars
        ViewCompat.setOnApplyWindowInsetsListener(webView) { v, insets ->
            lastInsets = insets
            applyNativePadding(v as WebView, insets)
            injectSafeAreaInsets(v, insets)
            insets
        }

        // Force initial inset dispatch after layout
        webView.post {
            ViewCompat.requestApplyInsets(webView)
        }
    }

    override fun onResume() {
        super.onResume()
        // Re-apply insets when returning to app
        webViewRef?.post {
            webViewRef?.let { webView ->
                lastInsets?.let { insets ->
                    applyNativePadding(webView, insets)
                    injectSafeAreaInsets(webView, insets)
                }
                // Also request fresh insets
                ViewCompat.requestApplyInsets(webView)
            }
        }
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (hasFocus) {
            webViewRef?.post {
                webViewRef?.let { webView ->
                    lastInsets?.let { insets ->
                        applyNativePadding(webView, insets)
                        injectSafeAreaInsets(webView, insets)
                    }
                    ViewCompat.requestApplyInsets(webView)
                }
            }
        }
    }

    /**
     * Apply insets as native Android padding on the WebView.
     * This GUARANTEES content respects safe areas even if JS injection fails.
     * The WebView content area is pushed in by the exact inset amounts.
     */
    private fun applyNativePadding(webView: WebView, insets: WindowInsetsCompat) {
        val systemBars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
        val displayCutout = insets.getInsets(WindowInsetsCompat.Type.displayCutout())
        val imeVisible = insets.isVisible(WindowInsetsCompat.Type.ime())
        val imeHeight = insets.getInsets(WindowInsetsCompat.Type.ime()).bottom

        val safeTop = maxOf(systemBars.top, displayCutout.top)
        val safeLeft = maxOf(systemBars.left, displayCutout.left)
        val safeRight = maxOf(systemBars.right, displayCutout.right)
        val safeBottom = if (imeVisible) imeHeight else maxOf(systemBars.bottom, displayCutout.bottom)

        webView.setPadding(safeLeft, safeTop, safeRight, safeBottom)
    }

    /**
     * Inject safe area CSS vars into the WebView.
     * This provides granular control within the web page (e.g., individual
     * sidebar sections can have their own safe area padding).
     */
    private fun injectSafeAreaInsets(webView: WebView, insets: WindowInsetsCompat) {
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
    }
}