# Auditoría Rust Best Practices — Recurrency

Fecha: 2026-05-12 | Skill: `rust-best-practices` (Apollo GraphQL)

---

## 🔴 Crítico

### 1. `unwrap()` en código de producción — API client

**Archivo:** `frontend/src/api/preferences.rs:30,50,63,74,85`

```rust
// L30 — producción
let json = serde_json::to_string(&req).unwrap();
```

**Violación:** `unwrap()`/`expect()` fuera de tests. Si la serialización falla, la app WASM entra en pánico sin recuperación.

**Fix:** Retornar `Result` o usar `?` con tipo de error adecuado.

**Impacto:** 5 llamadas en `preferences.rs`, patrón repetido en `settings.rs:30,49,73,87,98` y otros módulos api/. ~30+ instancias en código de producción del frontend.

---

### 2. `unwrap()` en construcción de fechas — componentes de página

**Archivo:** `frontend/src/pages/calendar_page.rs:57,74,76,514,549`

```rust
// L57 — componente, no test
let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
```

**Violación:** `from_ymd_opt` devuelve `Option<NaiveDate>`. Fechas inválidas (ej. 2024-02-30) causan panic.

**Fix:** Usar `unwrap_or` con fallback, o propagar `Option` al renderizado.

---

## 🟡 Moderado

### 3. Clippy warnings acumulados

**Frontend (~40 warnings):**
| Tipo | Count | Ejemplo |
|------|-------|---------|
| Unused imports | 2 | `Input`, `Tooltip` |
| Unused variables | 5+ | `body_json` x2, `i` x2, `show_filters` |
| Redundant clone on Copy | 2 | `Navigator.clone()` |
| Manual `!Range::contains` | 3 | Reemplazar con `range.contains()` |
| Cannot test inner items | 22+ | `subscriptions_page.rs` — mover tests fuera |

**Backend (26+ warnings + 1 error):**
| Tipo | Count | Ejemplo |
|------|-------|---------|
| Always-true assertions | 12+ | `assert_eq!(true, ...)` |
| `assert_eq!` with literal bool | 3 | Usar `assert!` |
| `unwrap()` on `Ok`/`None` | 5 | Tests con valores conocidos |
| **ERROR** approx PI + logic bug | 1 | Bloquea compilación |

### 4. `.clone()` en tipos `Copy`

**Archivos:** `frontend/src/layout/app_layout.rs` (2x)

```rust
let nav = navigator().clone(); // Navigator implementa Copy
```

**Fix:** Eliminar `.clone()`, usar directamente. Clippy ya lo detecta.

### 5. Redundant closure

**Archivo:** frontend (1 instancia)

```rust
// Clippy sugiere pasar la función directamente
.map(|x| foo(x)) → .map(foo)
```

---

## 🟢 Buenas Prácticas

| Práctica | Estado |
|----------|--------|
| `thiserror` para errores de librería | ✅ Backend usa `thiserror` |
| `anyhow` solo en binarios | ✅ Solo en `backend/src/main.rs` |
| Sin `#[allow(clippy::...)]` | ✅ Cero usos |
| Trait injection para testabilidad | ✅ Todos los servicios backend |
| Tests pasan | ✅ 1533/1533 frontend, backend pendiente |
| Cobertura >80% | ✅ Backend 86.88%, Frontend 80.22% |
| `//` para *why*, `///` para *what* | ✅ Consistente |

---

## 🔧 Recomendaciones

### Inmediatas (esta sesión)
1. Eliminar imports no usados: `Input` en `components/logo_search.rs`, `Tooltip` en `components/mod.rs`
2. Eliminar variables no usadas: `body_json` x2 en `api/client.rs`, `show_filters` en `transactions_page.rs`
3. Reemplazar `.clone()` en `Navigator` (Copy) por uso directo

### Corto plazo (próximo PR)
4. Reemplazar `unwrap()` en `api/preferences.rs` y `api/settings.rs` por `?` con manejo de error
5. Reemplazar `from_ymd_opt(...).unwrap()` en `calendar_page.rs` por `unwrap_or` con fallback seguro
6. Corregir error de clippy en backend ("approximate PI" + "logic bug")

### Medio plazo
7. Mover 22+ tests anidados de `subscriptions_page.rs` a módulo `tests` separado
8. Estandarizar nombres de tests: `fn subject_scenario_expected()`
9. Agregar `cargo clippy -- -D warnings` al pre-commit hook (junto con `cargo test --lib`)
