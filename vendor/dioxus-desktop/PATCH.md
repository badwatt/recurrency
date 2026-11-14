# Recurrency dioxus-desktop patch

This is dioxus-desktop 0.7.10.

On Android only, the internal WebView origin is changed from the Dioxus default
`https://dioxus.index.html/` to `https://recurrency.app/`. Android autofill
services read that origin as the app's web domain; without the change Bitwarden
lists the app as `dioxus.index.html`.

Desktop platforms keep the upstream `dioxus://index.html/` origin.
