#!/usr/bin/env python3
"""Patch RustWebView.kt with FOUC prevention and native loader."""
import sys, re

path = sys.argv[1]
with open(path, 'r') as f:
    content = f.read()

# Remove any existing FOUC/patch block
content = re.sub(
    r'(?s)        // Prevent FOUC:.*?(?=\n        (?:if \(WebViewFeature|isDocumentStartScriptEnabled|settings\.|fun |\}))',
    '', content
)

# Remove any existing system-bars bridge (keeps the patch idempotent)
BRIDGE_MARKER = '// --- Recurrency system bars bridge ---'
content = re.sub(r'(?s)\n*' + re.escape(BRIDGE_MARKER) + r'.*$', '\n', content)
content = re.sub(r'\n\s*addJavascriptInterface\(SystemBarsBridge\(this\), "RecurrencyAndroid"\)', '', content)

CSS = (
    'body{visibility:hidden}'
    'html{background:#1e1e2e;color:#cdd6f4}'
    'html.light{background:#FAF9F2;color:#212121}'
    'html.dark{background:#0d0d11;color:#ededed}'
    'html.mocha{background:#1e1e2e;color:#cdd6f4}'
    '#dxloader{visibility:visible!important;display:flex;flex-direction:column;align-items:center;justify-content:center;'
    'position:fixed;inset:0;z-index:99999;font-family:system-ui,sans-serif}'
    '@keyframes dxspin{to{transform:rotate(360deg)}}'
    '@keyframes dxpulse{0%,100%{opacity:1}50%{opacity:.5}}'
    '@keyframes dxdot{0%,60%,100%{opacity:0}30%{opacity:1}}'
    '#dxloader .spin-outer{position:relative;width:48px;height:48px}'
    '#dxloader .spin-ring{position:absolute;inset:0;border-radius:50%;border:4px solid #3b4261}'
    '#dxloader .spin-ring.top{border-top-color:#6366f1;animation:dxspin .9s linear infinite}'
    '#dxloader .spin-ring.inner{inset:6px;border-bottom-color:#818cf8;animation:dxspin 1.4s linear infinite reverse}'
    '#dxloader .brand{margin-top:24px;font-size:20px;font-weight:700;letter-spacing:-0.02em}'
    '#dxloader .tagline{margin-top:6px;font-size:13px;animation:dxpulse 2s ease-in-out infinite}'
    '#dxloader .tagline .dot{animation:dxdot 1.2s infinite;opacity:0}'
    '#dxloader .tagline .dot:nth-child(2){animation-delay:.2s}'
    '#dxloader .tagline .dot:nth-child(3){animation-delay:.4s}'
    '#dxloader.light{background:#FAF9F2;color:#212121}'
    '#dxloader.light .spin-ring{border-color:#d4d4d8}'
    '#dxloader.light .spin-ring.top{border-top-color:#6366f1}'
    '#dxloader.light .spin-ring.inner{border-bottom-color:#818cf8}'
    '#dxloader.light .brand{color:#212121}'
    '#dxloader.light .tagline{color:#71717a}'
    '#dxloader.dark{background:#0d0d11;color:#ededed}'
    '#dxloader.dark .spin-ring{border-color:#27272a}'
    '#dxloader.dark .spin-ring.top{border-top-color:#a78bfa}'
    '#dxloader.dark .spin-ring.inner{border-bottom-color:#7c3aed}'
    '#dxloader.dark .brand{color:#ededed}'
    '#dxloader.dark .tagline{color:#a1a1aa}'
)

inject_lines = [
    '        // Expose the system-bars bridge so the theme can paint status/nav bars.',
    '        addJavascriptInterface(SystemBarsBridge(this), "RecurrencyAndroid")',
    '        // Prevent FOUC: mocha background + themed native loader.',
    '        // setBackgroundColor paints mocha behind WebView — eliminates white flash.',
    '        // addDocumentStartJavaScript injects CSS + loader HTML before WASM loads.',
    '        // Reads localStorage for theme preference to match loader colors to theme.',
    '        setBackgroundColor(android.graphics.Color.parseColor("#1e1e2e"))',
    '        if (WebViewFeature.isFeatureSupported(WebViewFeature.DOCUMENT_START_SCRIPT)) {',
    '            WebViewCompat.addDocumentStartJavaScript(this, """',
    '(function(){',
    "var S=document.createElement('style');",
    f"S.textContent='{CSS}';",
    "document.documentElement.appendChild(S);",
    # Read theme from localStorage
    "var t='mocha';",
    "try{var s=localStorage.getItem('recurrency-theme');if(s&&['light','dark','mocha'].indexOf(s)>=0)t=s;}catch(e){}",
    # Resolve system theme
    "if(t==='system'){try{if(window.matchMedia('(prefers-color-scheme: dark)').matches)t='mocha';else t='light';}catch(e){t='mocha';}}",
    # Create loader div with theme class
    "var D=document.createElement('div');",
    "D.id='dxloader';",
    # Use JS template literal so double quotes in HTML don't need escaping
    'D.innerHTML=`<div class="spin-outer"><div class="spin-ring top"></div><div class="spin-ring inner"></div></div><div class="brand">Recurrency</div><div class="tagline">Every expense is a subscription<span class="dot">.</span><span class="dot">.</span><span class="dot">.</span></div>`;',
    "D.className=t;",
    "document.documentElement.appendChild(D);",
    # Set theme class on html element for CSS vars
    "var h=document.documentElement;",
    "if(t==='light')h.className='light';",
    "else h.className=t+' dark';",
    '})();',
    '            """, setOf("*"))',
    '        }',
    '',
]

inject = '\n'.join(inject_lines)

content = content.replace(
    "settings.javaScriptCanOpenWindowsAutomatically = true",
    "settings.javaScriptCanOpenWindowsAutomatically = true\n" + inject
)

BRIDGE_CLASS = '''
// --- Recurrency system bars bridge ---
private class SystemBarsBridge(private val view: android.webkit.WebView) {
    @android.webkit.JavascriptInterface
    fun setSystemBars(color: String, light: Boolean) {
        val activity = view.context as? android.app.Activity ?: return
        activity.runOnUiThread {
            val window = activity.window
            val parsed = try {
                android.graphics.Color.parseColor(color)
            } catch (e: IllegalArgumentException) {
                return@runOnUiThread
            }
            window.statusBarColor = parsed
            window.navigationBarColor = parsed
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.Q) {
                window.isStatusBarContrastEnforced = false
                window.isNavigationBarContrastEnforced = false
            }
            androidx.core.view.WindowInsetsControllerCompat(window, window.decorView).apply {
                isAppearanceLightStatusBars = light
                isAppearanceLightNavigationBars = light
            }
            view.setBackgroundColor(parsed)
        }
    }
}
'''

content = content.rstrip() + '\n' + BRIDGE_CLASS

with open(path, 'w') as f:
    f.write(content)
print('Patched RustWebView.kt: mocha background + themed native loader')