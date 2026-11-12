//! Cross-platform file downloads. Android WebView cannot download Blob URLs,
//! so the generated APK exposes a small DownloadBridge instead.

fn js_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

pub fn save_text(filename: &str, mime: &str, text: &str) {
    let filename = js_string(filename);
    let mime = js_string(mime);
    let text = js_string(text);
    let _ = dioxus::document::eval(&format!(
        r#"(function() {{
            const text = {text};
            if (window.RecurrencyDownload && window.RecurrencyDownload.saveFile) {{
                window.RecurrencyDownload.saveFile({filename}, {mime}, text);
                return;
            }}
            const blob = new Blob([text], {{type: {mime}}});
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = {filename};
            a.click();
            setTimeout(() => URL.revokeObjectURL(url), 1000);
        }})();"#
    ));
}

pub fn print_html(html: &str) {
    let html = js_string(html);
    let _ = dioxus::document::eval(&format!(
        r#"(function() {{
            if (window.RecurrencyDownload && window.RecurrencyDownload.printHtml) {{
                window.RecurrencyDownload.printHtml({html});
                return;
            }}
            const w = window.open('', '_blank');
            if (w) {{
                w.document.write({html});
                w.document.close();
                setTimeout(() => w.print(), 400);
            }}
        }})();"#
    ));
}
