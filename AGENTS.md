# Recurrency

**Recurrency** is a subscription and expense tracker that treats every expense like a subscription. It helps users budget by viewing recurring costs as ongoing commitments, prompting them to ask: *"Would I subscribe to this today?"*

> ⚠️ **Usa las skills de Rust: obligatorio, sin excepciones.** Antes de escribir, revisar, refactorizar o portar código, carga las skills de Rust disponibles (descúbrelas si hace falta). Si hay varias, aplica la que corresponda; si dudas, todas. Código que contradiga una skill de Rust está mal aunque compile.

## 1. Datos del proyecto: se leen, no se escriben

| Dato | Dónde vive | Cómo se lee |
|---|---|---|
| Nombre | `[package] name` · en workspace, el del crate del binario | `env!("CARGO_PKG_NAME")` |
| Versión | `[package] version` · `[workspace.package] version` + `version.workspace = true` | `env!("CARGO_PKG_VERSION")` |
| Edición y MSRV | `[package] edition` · `[package] rust-version` | `Cargo.toml`, y el CI lo compila |
| Tipo | `src/main.rs` (bin) · `src/lib.rs` (lib) · deps como `axum`/`clap` | lo que exista en el repo |
| Toolchain y herramientas | `flake.nix` + `rust-toolchain.toml` | el devShell del flake |
| Cobertura mínima | configuración del CI o `[workspace.metadata]` | por defecto **90** |

- ⛔ Archivo `VERSION`, literales repetidos (README, rustdoc, `--version`, User-Agent, títulos) y tags escritos a mano.
- ⛔ Copiar aquí o en el README la edición, el MSRV o la lista de herramientas: se leen del repo.
- En CI y scripts: `cargo metadata --no-deps --format-version 1` o `cargo pkgid`. ⛔ Regex sobre el TOML: `version.workspace = true` lo rompe.
- El tag `v<version>` es **consecuencia** de la versión, nunca su causa.
- `name`, `version`, `edition` y `rust-version` se consultan con el usuario.

## 2. Arquitectura

- **Hexagonal** — dominio puro en el centro; `sqlx`/`axum`/`tokio` solo en adaptadores. El dominio no conoce el framework.
- **Vertical slicing** — cada feature es una carpeta autocontenida con dominio, puertos, adaptadores y tests.
- **Screaming architecture** — los directorios dicen *qué hace* el sistema, no *qué framework usa*.
- **Clean code** — funciones cortas, un nivel de abstracción por función, nombres que no necesiten comentario.
- Dependencias hacia dentro: `adapters → ports → domain`. Nunca al revés.

```
src/main.rs                composición: config, adaptadores, arranque
src/shared/                error.rs (AppError/AppResult) · config.rs (inyectada) · mocks.rs (cfg(test))
src/features/<feature>/    domain.rs · ports.rs (traits) · service.rs · adapters/ · tests/
```

## 3. Reglas

| ⛔ Nunca | ✅ En su lugar |
|---|---|
| `unsafe`, punteros raw, `transmute`, `mem::forget`, `ManuallyDrop`, `MaybeUninit`, `Box::leak` | Cero tolerancia, sin excepciones |
| `Rc` entre hilos | `Arc<dyn Trait>`, visible en `AppState` y callbacks |
| `Arc` por reflejo · `clone()` para calmar al borrow checker | Préstamos (`&T`, `&str`, `&[T]`), mover valores, `Box<T>` si el dueño es único |
| `.unwrap()` / `.expect()` en producción | `?`, `match` o fallback explícito |
| Tragarse un error (`let _ =`, `ok()`) · `panic!` para flujo esperado | `Result` + `thiserror`; `Box<dyn Error + Send + Sync>` solo en `main`/CLI |
| Mensajes que filtran secretos | Qué falló y con qué input, sin datos sensibles |
| `unsafe` o env global mutado en tests · `unwrap` sin mensaje | Configuración inyectada u override thread-local; fixtures con mensaje de lo esperado |
| Helpers de test que ignoran un `Result` | Que el test falle |
| Valores hardcodeados (textos, URLs, timeouts, colores) | Constantes, configuración o tokens de tema |
| Nombre o versión a mano | `env!("CARGO_PKG_NAME")`, `env!("CARGO_PKG_VERSION")` |
| Dependencias injustificadas · inputs de flake que sobran | Mínimo de deps, actualizadas; cada input justifica su mantenimiento |
| `cargo install`, `nix profile install`, `nix-env -i`, paquetes del sistema | Toda herramienta se declara en `flake.nix` |
| `--impure`, `builtins.getEnv`, rutas absolutas del host | El resultado depende solo del flake |
| Optimizar «por si acaso» | Benchmark o perfil primero |
| Lógica repetida ×2 | Extraer a función, tipo o módulo |
| Código muerto o comentado · `#[allow]` sin motivo | Borrar; el `allow` en el ítem, con motivo |
| Warnings de `fmt`/`clippy`/`doc` | Cero: `-D warnings` |

`Arc` cuesta (contador atómico): solo con propiedad compartida real. Si un `clone` no se explica solo, el motivo va en el código.

## 4. Entorno Nix

- `flake.nix` es el **único** sitio donde se declaran herramientas (toolchain de Rust, `cargo-llvm-cov`, `cargo-deny`, `act`).
- `devShells.default` lleva solo herramientas de desarrollo; `packages.default` construye el binario. Sin duplicar dependencias entre ambos.
- Uso diario: `nix develop` (o `direnv` + `use flake`). Un comando suelto: `nix develop -c <cmd>`.
- `flake.lock` se commitea. Actualizar es una decisión explícita (`nix flake update`) revisando el diff de inputs.
- La versión de Rust la fija `rust-toolchain.toml` dentro del shell; ⛔ `rustup default` global.

## 5. TDD y cobertura

Cada cambio empieza con un test en **rojo**, cada commit lleva sus tests, y nunca se avanza de feature con tests rojos.

```
1. RED      → el test falla por la razón correcta
2. GREEN    → mínimo código para que pase
3. REFACTOR → limpiar; los tests siguen verdes
```

```rust
#[test]
fn validate_empty_name_fails() {
    let result = validate_form(&form_with_name(""));
    assert!(result.is_err());
}
```

- **Diff coverage 100%** — toda línea y rama nueva o modificada.
- **Global ≥ 90** (o el mínimo que declare el CI) — nunca decreciente.
- Sin tests de relleno. Lo no testeable se justifica: `// No test: <motivo>. Cubierto por <integración | manual | N/A>`.
- Test: `fn <verbo>_<sujeto>_<escenario>_<resultado>()` — un concepto, un `assert`, determinista (sin red, sin sleeps, sin timestamps reales). Junto al módulo o en `<feature>/tests/`; integración en `tests/`.

| Testeable ✅ | No testeable ⛔ → aislar en un adaptador fino y cubrir con integración |
|---|---|
| Funciones puras, serde, `Clone`/`Eq`, value objects | UI generada por macro, WASM y APIs de plataforma |
| Traits y servicios con fakes | Async de DB sin doble, integración sin infraestructura |

## 6. Inyección de dependencias

Todo servicio con efectos (I/O, DB, tiempo, red) expone un trait; el dominio depende del trait, nunca de la implementación.

```rust
#[async_trait]
pub trait ItemServiceTrait: Send + Sync {
    async fn create(&self, owner_id: Uuid, req: CreateItemRequest) -> AppResult<ItemResponse>;
    async fn get(&self, owner_id: Uuid, id: Uuid) -> AppResult<ItemResponse>;
    async fn delete(&self, owner_id: Uuid, id: Uuid) -> AppResult<()>;
}

pub struct PgItemService { pool: PgPool }

#[async_trait]
impl ItemServiceTrait for PgItemService {
    async fn create(&self, owner_id: Uuid, req: CreateItemRequest) -> AppResult<ItemResponse> {
        // acceso a datos …
    }
    // …
}

#[derive(Clone)]
pub struct AppState {                   // una entrada por servicio con efectos
    pub(crate) item_service: Arc<dyn ItemServiceTrait>,
    pub(crate) auth_service: Arc<dyn AuthServiceTrait>,
}
```

Los fakes viven en `src/shared/mocks.rs` (feature `test-support` o `cfg(test)`) y se configuran por constructor, nunca mutando estado global ni env:

```rust
pub struct FakeItemService { pub should_fail: bool, items: Mutex<Vec<Item>> }

#[async_trait]
impl ItemServiceTrait for FakeItemService {
    async fn create(&self, owner_id: Uuid, req: CreateItemRequest) -> AppResult<ItemResponse> {
        if self.should_fail {
            return Err(AppError::internal("fallo simulado"));
        }
        let item = Item::new(owner_id, req);
        self.items
            .lock()
            .expect("el mutex del fixture no debe estar envenenado")
            .push(item.clone());        // copia que queda en el estado simulado
        Ok(item.into())
    }
}
```

## 7. Calidad y cierre

Antes de cada commit, dentro del devShell (`nix develop`):

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --document-private-items   # sin warnings
```

- `nix flake check` debe pasar: declara los checks de `fmt`, `clippy` y `test` en el flake para no duplicar la lógica que ya existe.
- Cobertura con `cargo llvm-cov`; cualquier workflow de CI se verifica en local con `act`, ambos desde el flake.

**Terminado** cuando: hay un test que falla sin el cambio y pasa con él · diff coverage 100% y global sin bajar · los cuatro comandos y `nix flake check` en verde · el código respeta las skills de Rust aplicables · no hay `unwrap`, `clone` ni dependencia nuevos sin justificar · README/rustdoc actualizados si cambió el comportamiento público · si cambió `version`, `CHANGELOG.md` al día y usuario avisado.

## 8. Release y versionado

Un release se dispara por el **cambio de `version` en `Cargo.toml`**, nunca por un archivo aparte ni un tag manual. El workflow lo diseña el agente según el proyecto (artefactos, targets, publicación o no en crates.io); estos invariantes no se negocian:

- Detecta la versión; si el tag `v<version>` ya existe → no-op en verde. **Idempotente**: re-ejecutar no duplica tags ni releases.
- Verifica antes de publicar: `fmt`, `clippy -D warnings`, `test`, build del artefacto — cada paso dentro del shell, `nix develop -c …`.
- ⛔ Bump desde el CI o commit de vuelta al repo · ⛔ tag a mano antes de que la versión esté en `Cargo.toml`.

**Commits**: `type(scope): description` (`feat`, `fix`, `test`, `refactor`, `docs`, `chore`, `perf`), un cambio coherente con sus tests. **Preguntar siempre** antes de `git push` y antes de tocar `name`/`version`. **Semver** `MAJOR.MINOR.PATCH`: `MAJOR` incompatible, `MINOR` funcionalidad compatible, `PATCH` corrección o refactor compatible (pre-1.0, misma regla sobre `0.MINOR.PATCH`).
