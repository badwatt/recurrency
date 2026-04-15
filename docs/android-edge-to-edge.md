# Android Edge-to-Edge Setup

After running `npx tauri android init`, apply these changes for transparent status bar and navigation bar:

## 1. MainActivity.kt — `gen/android/app/src/main/java/com/alvaro17f/recurrency/app/MainActivity.kt`

```kotlin
package com.alvaro17f.recurrency.app

import android.content.res.Configuration
import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.core.view.WindowInsetsControllerCompat
import com.alvaro17f.recurrency.app.TauriActivity

class MainActivity : TauriActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        applySystemBarIconMode()
    }

    override fun onResume() {
        super.onResume()
        applySystemBarIconMode()
    }

    override fun onConfigurationChanged(newConfig: Configuration) {
        super.onConfigurationChanged(newConfig)
        applySystemBarIconMode()
    }

    private fun applySystemBarIconMode() {
        val isNight = (resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK) == Configuration.UI_MODE_NIGHT_YES
        val controller = WindowInsetsControllerCompat(window, window.decorView)
        controller.isAppearanceLightStatusBars = !isNight
        controller.isAppearanceLightNavigationBars = !isNight
    }
}
```

## 2. themes.xml — `gen/android/app/src/main/res/values/themes.xml`

```xml
<resources>
    <style name="Theme.Recurrency" parent="Theme.MaterialComponents.DayNight.NoActionBar">
        <item name="android:statusBarColor">@android:color/transparent</item>
        <item name="android:navigationBarColor">@android:color/transparent</item>
        <item name="android:windowLightStatusBar">true</item>
        <item name="android:windowLightNavigationBar">true</item>
    </style>
</resources>
```

## 3. themes.xml (night) — `gen/android/app/src/main/res/values-night/themes.xml`

```xml
<resources>
    <style name="Theme.Recurrency" parent="Theme.MaterialComponents.DayNight.NoActionBar">
        <item name="android:statusBarColor">@android:color/transparent</item>
        <item name="android:navigationBarColor">@android:color/transparent</item>
        <item name="android:windowLightStatusBar">false</item>
        <item name="android:windowLightNavigationBar">false</item>
    </style>
</resources>
```

## 4. build.gradle.kts — add activity-ktx dependency

In `gen/android/app/build.gradle.kts`, add to `dependencies`:

```kotlin
implementation("androidx.activity:activity-ktx:1.9.3")
```

## CSS already configured

- `viewport-fit=cover` in meta viewport
- `env(safe-area-inset-*)` padding on body in `index.css`
- `theme-color` meta updated dynamically via inline script
- Swipe-to-open sidebar in `useSwipeSidebar` hook