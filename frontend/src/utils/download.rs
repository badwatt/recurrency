//! Cross-platform file exports. Desktop writes a real file through the native
//! save dialog; web and Android use the browser download / print bridges.

/// Result of an export action, used to show a precise toast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportResult {
    /// The file was written to this path.
    Saved(String),
    /// The browser started a download.
    Downloaded,
    /// A print dialog was opened (the user can save it as PDF).
    Printed,
    /// The user dismissed the save dialog.
    Cancelled,
}

/// Build a stable report filename for the selected period.
pub fn export_filename(year: &str, month: &str, extension: &str) -> String {
    let extension = extension.trim_start_matches('.');
    format!(
        "recurrency-report-{}-{}.{}",
        year.trim(),
        month.trim(),
        extension
    )
}

/// Export a CSV report.
pub async fn export_csv(filename: String, csv: String) -> Result<ExportResult, String> {
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "desktop"
    ))]
    {
        save_with_dialog(&filename, "CSV", &["csv"], csv.as_bytes()).await
    }
    #[cfg(not(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "desktop"
    )))]
    {
        save_text(&filename, "text/csv", &csv);
        Ok(ExportResult::Downloaded)
    }
}

/// Export a PDF report. Desktop writes the generated PDF bytes; other
/// platforms open the print dialog with the HTML report.
pub async fn export_pdf(
    filename: String,
    pdf: Vec<u8>,
    html: String,
) -> Result<ExportResult, String> {
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "desktop"
    ))]
    {
        let _ = html;
        save_with_dialog(&filename, "PDF", &["pdf"], &pdf).await
    }
    #[cfg(not(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "desktop"
    )))]
    {
        let _ = (filename, pdf);
        print_html(&html);
        Ok(ExportResult::Printed)
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "android"),
    feature = "desktop"
))]
async fn save_with_dialog(
    filename: &str,
    kind: &str,
    extensions: &[&str],
    bytes: &[u8],
) -> Result<ExportResult, String> {
    let Some(file) = rfd::AsyncFileDialog::new()
        .set_title("Save report")
        .set_file_name(filename)
        .add_filter(kind, extensions)
        .save_file()
        .await
    else {
        return Ok(ExportResult::Cancelled);
    };
    let path = file.path().to_path_buf();
    std::fs::write(&path, bytes).map_err(|error| error.to_string())?;
    Ok(ExportResult::Saved(path.display().to_string()))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_includes_period_and_extension() {
        assert_eq!(
            export_filename("2026", "11", "csv"),
            "recurrency-report-2026-11.csv"
        );
        assert_eq!(
            export_filename(" 2026 ", " 11 ", ".pdf"),
            "recurrency-report-2026-11.pdf"
        );
    }

    #[test]
    fn export_result_is_comparable() {
        assert_eq!(
            ExportResult::Saved("/tmp/report.pdf".to_string()),
            ExportResult::Saved("/tmp/report.pdf".to_string())
        );
        assert_ne!(ExportResult::Downloaded, ExportResult::Printed);
    }

    #[test]
    fn js_string_escapes_quotes() {
        assert_eq!(js_string("a\"b"), "\"a\\\"b\"");
    }
}
