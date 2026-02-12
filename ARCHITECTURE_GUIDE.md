# Architecture Guide (Boilerplate)

This project uses a layered Axum + Askama architecture. Use this file as the source of truth when creating new projects from this boilerplate.

## 1. Core Layers

1. `routes/`:
- Owns URL structure only.
- Delegates to controller handlers.

2. `controllers/`:
- Owns HTTP concerns: extractors, session/CSRF checks, validation, response mapping.
- Calls repository functions for DB operations.
- Maps model data into template view models.

3. `repository/`:
- Owns SQL queries and DB persistence with `sqlx`.
- No HTTP/session/template logic.

4. `models/entities/`:
- Owns data contracts: DB entities, form DTOs, query params, datatable payloads.

5. `views/templates/` + `templates/`:
- Rust `Template` structs and Askama HTML files.
- Template structs are strongly typed and live in Rust.

6. `state.rs`:
- Global app dependencies (`db`, `redis`) in `AppState`.

## 2. Current Route Domains

- `public`: landing, login, register, logout, admin login.
- `user`: authenticated user pages.
- `admin`: authenticated admin pages, user management, geo management.
- `api/v1`, `api/v2`: JSON endpoints.

All domains are composed in `src/routes/mod.rs` and mounted in `main.rs`.

## 3. Request Flow Pattern

Follow this pipeline for every feature:

1. Add route in `src/routes/<domain>.rs`.
2. Add handler in `src/controllers/page_controller/<domain>.rs`.
3. Add/extend DTOs in `src/models/entities/*`.
4. Add DB function in `src/repository/<domain>_repository.rs`.
5. Add template struct in `src/views/templates/<domain>.rs`.
6. Add Askama HTML file in `templates/...`.
7. Re-export from `mod.rs` files where needed.

## 4. Naming Conventions

- Handler names:
- Page render: `<feature>_page` or `<resource>_list`/`<resource>_detail`
- Form submit: `<feature>_submit`
- Delete action: `<resource>_delete`

- Repository names:
- `find_<resource>_by_id`, `get_<resource>s`, `create_<resource>`, `update_<resource>`, `delete_<resource>`

- Template structs:
- `<Area><Action>Template` (for example `AdminEditUserTemplate`)

## 5. Styling and Template Change Policy

Before applying visual/UI changes (`templates/*.html`, `static/css/*`, `static/js/*` affecting presentation), ask the user first and wait for approval.

Required workflow:

1. Propose the styling/template change scope first.
2. Wait for explicit user confirmation.
3. Apply changes only after approval.

Exception:
- If the user explicitly requests styling/template edits in the current task, proceed directly.

## 6. Security and Session Rules

1. Auth:
- Use extractor types from `src/controllers/auth_controller.rs`:
- `AuthUser` / `OptionalAuthUser`
- `AdminUser` / `OptionalAdminUser`

2. CSRF:
- For all mutating form endpoints, validate via helpers in `src/controllers/page_controller/shared.rs`:
- `ensure_csrf_token(session)` for GET pages
- `validate_csrf(session, token)` for POST actions

3. Passwords:
- Hash/verify through `src/utils.rs` helpers.
- Before implementing password features, ask the user to confirm:
- hash algorithm (for example bcrypt/argon2 and parameters)
- password policy (length, complexity, expiry, reuse rules)
- reset/change flow (email reset, OTP, admin reset, in-session change)
- Proceed only after explicit user decision on these items.

4. SQL safety:
- Use parameterized `sqlx` queries only.

## 7. Caching Rules

- Redis is used for:
- Session storage (`tower-sessions` + Redis store)
- Lightweight query cache (example: geo data in `shared.rs`)
- When changing cached data, invalidate related keys in the same request path (example: `invalidate_geo_cache`).

## 8. Environment and Runtime

Required env vars (see `.env.example`):

- `DATABASE_URL`
- `REDIS_URL`
- `APP_HOST`
- `APP_PORT`
- `SESSION_TIMEOUT`
- `APP_ENV`
- `LOG_DIR` (optional override)

Runtime wiring lives in `src/main.rs`:
- Logging init
- DB pool init
- Redis/session init
- Router/app state assembly

## 9. Boilerplate Checklist for New Projects

Before copying this project to a new app:

1. Rename package and app title (`Cargo.toml`, template text).
2. Replace domain models under `src/models/entities/`.
3. Replace repository SQL for your schema.
4. Keep route/controller/repository separation unchanged.
5. Keep auth + CSRF patterns unchanged unless intentionally redesigning.
6. Keep `AppState` as the single dependency container.
7. Remove unused templates/static assets after first feature pass.
8. Validate with:
- `cargo check`
- `cargo clippy`
- manual login/register/admin flow smoke test

## 10. Definition of Done for Any New Feature

A feature is done only when all are true:

1. Route is registered and reachable.
2. Handler validates input and auth requirements.
3. Repository query is isolated and parameterized.
4. Template renders with typed data.
5. CSRF is enforced for mutating forms.
6. Related cache keys are invalidated (if applicable).
7. Errors return explicit user-facing responses or error template.
