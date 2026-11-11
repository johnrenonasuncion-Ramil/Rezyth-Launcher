# Harvest from pull request #2 (Volt Launcher) — ledger

**Analysis target updated 2026-11-08**: the user named **PR #2** the branch to harvest. PR #1
(still open, untouched) points at an older snapshot of the same fork; PR #2's head is the newer
one and is where the in-game **Volt Settings** hub and **Volt Boost** live.

- Head: `entitybrian69-bit/Volt-launcher`, branch `arena/0cae2b73-volt-launcher`, `9e855468`
  (2026-11-08, "Volt Replay: Replay Studio").
- Base: this repository's `android` @ `7c38f33c1` (Starling 0.2.3); merge base `c3905131`.
- Head-only: **339 commits**, 695 files, +162,084/−; dry-run merge: **418 conflicted paths**
  (87 content conflicts under `crates/`, 11 add/add, 10 rename/delete), same structural
  collision as PR #1 (old monolithic `lib.rs`, its own launcher frontend, workflow renames,
  wasm SDK) — so the verdict stands: **harvest per feature, never merge**.

The head's launcher frontend, its Gradle host, the wasm mod SDK and the vendored winit patch stay
**out** (denied by instruction, or removed by design — see [SYNC.md](SYNC.md)). Everything below is
harvested from `9e855468`.

## Where the requested features live in the head

- **"Volt Settings" button and screen (in-game)**: `options_ui.rs` (`Page::VoltSettings`,
  a frosted-glass hub on the Options screen), `options_volt_pages.rs` (789 lines: `VoltVisuals`,
  `VoltQol`, Volt Boost, Volt Waypoints, Volt Minimap, Volt Shaders pages), `graphics_style.rs`
  (`volt_settings`, `volt_mods` styles), and ~85 switches as `extras::Item` variants in
  `extras.rs` (3,195 lines). The launcher's own copy of the same page (`launcher_pages.rs`
  "Launch settings"/Boost) is the denied frontend and is not ported.
- **Volt Boost ("the optimizer with profiles")**: `volt_features.rs` §4 — `VoltBoostPreset`
  (`Off`, `Balanced`, `Competitive PvP`, `Ultra FPS`, `Battery Saver`, `Low-RAM Rescue`) plus the
  switches behind it (ADPF, big.LITTLE thread boost with timer slack, Volt Reflex, pipeline
  prewarm, dynamic menu/idle FPS, chunk-upload cap, entity/item/block-entity caps, LOD boost,
  particle budget, RAM purge, thermal guard, keep-awake...). `horizon-threads` gains
  `set_thread_boost` (Android `PR_SET_TIMERSLACK`).
- Also in `volt_features.rs` (1,563 lines): **Volt Waypoints** (store, death waypoints,
  Overworld↔Nether sync, projection) and **Volt Minimap** (frames, terrain modes, entity radar,
  world clock) — engines first, their in-game pages after.
- **The native mods (30+)**: not in-repo sources — **33 official registry mods** woven into the
  APK at build time (`bundled_native.rs` `WOVEN`: jei, horizon_physics, jade, veinminer,
  appleskin, status_effect_hud, toggle_sprint, inventory_profiles, shulker_peek, mousetricks,
  itemnew, eatinganimation, enchantmenthints, light_overlay, right_click_crops, smartf3,
  armor_status, crosshair_indicator, hitbox_plus, invisible_frames, freelook, voicechat,
  worldedit, horizonmatica, visuality_horizon, held_item_info, carryblock, chatheads, fast_items,
  customhud, coords_hud, biomecompass, skins_3d), plus 4 compiled-in engine mods (`BUNDLED`:
  zoom, fps_overlay, keystrokes, minimap). The head's `weave` job downloads the pinned `.hnmod`
  bundles from `horizon-rust/horizon-mods` releases, verifies each against the registry snapshot
  (`docs/native-mods/horizon-registry-63daa89b…json`, 799 lines), and weaves all of them into the
  engine library in **one** horizon-mixins pass (`tools/native_mods/compat.py` 543 lines,
  `build_set.py` 201, their tests 444). The woven mods register on the in-game Mods page through
  the `native_mods` hook point and are on until the player turns them off.

## Ported (verified by the `android` workflow on `feat/volt-harvest`)

| Feature | Commit | Notes |
|---|---|---|
| F2 screenshots into the system gallery | `4f9df3e54` | `android/gallery.rs`: MediaStore copy into `Pictures/HorizonAndroid` (Android 10+), legacy `Pictures` with the runtime write permission (Android 9-), background thread, chat report through `horizon.gallery.*` keys. The game keeps its own copy in `screenshots/`. |
| Local diagnostics | `e8189768d` | `android/diagnostics.rs`: bounded, redacted log copy and session state in the game folder's `performance/`; clean-exit mark; next start warns if the last run died. Logger gained a `set_sink` hook (horizon-client-render). |
| Android exit history | `e8189768d` | `android/exit_reports.rs`: `ActivityManager.getHistoricalProcessExitReasons` (Android 11+) into `performance/android-exits.json`, with bounded system traces for crashes and ANRs and the last log's and crash reports' tails beside them. |
| Exit-history borrow + unused imports | `a848813fc` | Caught by this branch's own Android build (run #42456739633). |
| Harvest ledger | `b0e7112e3` | This file, first written for PR #1, now aimed at PR #2. |

CI: `android.yml` run **42457716326** built the APK green at `a848813fc`.

### Follow-ups from the newer head

- `gallery.rs` evolved after PR #1's snapshot: `save(file, copy_image)` also puts the MediaStore
  **content URI on the clipboard** (`ClipData.newUri`) after publishing. Fold into our port as an
  optional extra step behind the same background copy.

## Already covered by this code base (checked in `9e855468`, not ported)

- Touch F3/F5 buttons, editable custom keys, perspective: this tree's `touch.rs` already has
  `Button::Debug`, perspective and the editable `touch-layout.json` editor.
- Winit Android mouse/wheel/modifier forwarding: this tree's input hook (`android.rs`'s `input`
  module) takes mouse sources and forwards keys itself; no vendored winit patch.
- Mobile GUI-atlas packing: this tree packs within the GPU's real limit (`hud.rs`
  `atlas_limit`/`set_gpu_texture_side`) and halves sprites under pressure.
- Render scale/aspect preferences: `graphics.rs` `resolution_scale` (25–200%).
- Accounts/Keystore, IME (TextMirror), gamepad haptics, DNS, secure store, prepare/install,
  import inbox, update installer, native-mod build service: all exist here.

## Candidates that need a decision (not ported)

- **GLES terrain fallback + optional Vulkan** (head's largest renderer change): lets devices
  without Vulkan install and play (`terrain_gles.wgsl.inc` variant, manifest `uses-feature`
  change, wgpu backend selection). Must be tested on a GL device; gates ARMv7/TV reach.
- **SAF DocumentsProvider**: exposes the game folder to Android's file pickers; the head's
  allowlist models its launcher's profiles, ours would need a redesign.
- **TV / ARMv7 packaging**: leanback banner/category (manifest + drawable), 32-bit ABI; ARMv7
  doubles the CI build.
- **Performance recorder / export ZIP / tombstone summariser**: triggers live in the head's
  launcher pages; an in-game trigger must be designed here first.

## Plan for the named priorities (Volt Boost first)

1. **Engines** — DONE: `volt_features.rs` ported into `horizon-client-render` and re-exported as
   `horizon_client::volt_features` (`b68b91738`, module declaration fixed `28582b3d0`; its
   dependencies exist here unchanged, and the minimap projection inverts this tree's own
   `compose`). Off by default; engines carry their own tests.
   **Volt Boost's big.LITTLE thread tuning** also landed (`28582b3d0`): `horizon-threads`' Android
   `favour` arm (50 us `PR_SET_TIMERSLACK`, on until `set_thread_boost`), the persisted
   `extras.threadBoost` switch, applied at the Android start. CI: run **42463514240** green.
2. **Volt Boost switches and effects** — the named set is complete
   (`39eb25aa6`–`c0b7bbf40`, CI runs **42467361511**–**42479850190** green). Landed: the
   persisted switches (`performanceHints`, `displayHz`, `threadBoost`, `voltPreset`, `dynamicFps`, `particleBudget`, `audioVoiceCap`, `autoDrs`, `thermalGuard`, `voltReflex`, `entityCap`, `itemRenderCap`, `blockEntityCap`, `menuBgMode`) and their effects — the ADPF
   gate with a proper session close, the display-rate request, the big.LITTLE timer-slack
   tuning, the frame-limiter idle cap, the particle budget (cap, distance cut, debris stop),
   the particle-sound voice cap, Auto-DRS and the thermal guard (resolution factor through the
   world post chain, particle thinning while hot), Volt Reflex (a device poll before the
   frame's sleep, Precise pacing while playing), the entity/item/block-entity nearest-first
   render caps, and the menu backdrop saver (FastDark skips the multi-pass blur).
   `apply_volt_preset` sets every one of them per the head’s profiles. Not ported, with reasons:
   the chunk-upload cap (our `UploadBudget` graphics setting already budgets uploads per frame,
   and the head’s own switch has no consumer in its tree), pipeline prewarm (also without a
   consumer in the head), and the Volt Settings hub UI (step 3, which makes every switch
   reachable in-game).
   **Tier 2/3 sweep complete** (`2cc6ad735`–`8c3a328ff`, CI runs **42484393177**–
   **42495038054** green): the animation throttle and active-animation cull, fast translucency,
   fast weather, the sign-text cull, the mob throttle, the hub shield, the chaos shield, the
   particle frustum cull, the memory saver, the streaming lookahead, the cave sky cull and
   frustum-priority meshing all landed with their engine features and their profile values.
   The `volt_boost_plus` mesh-pool group stays out: its five switches have no consumer in the
   head’s tree either (stored only), like pipeline prewarm.
3. **In-game UI** — DONE (`5dcf4008c`, fixed `fd9ec72d4`, CI run **42500485131** green):
   the Volt Settings hub (`Page::VoltSettings`) and its Boost page (`Page::VoltBoost`) on the
   game’s own Options screen, with every ported switch (27 items: the profile and all of them)
   two to a row through the same caption/value/cycle machinery the Extras page uses, off a
   translation key each (100 new bundled en_us keys). The Options screen gets a Volt Settings
   entry; the main menu gets a Volt Settings button that opens the same hub (in the game and
   the startup menu alike). The profile applies at once and the thread tuning follows the
   switch. The hub has room for the Waypoints and Minimap pages when their engines’ UI lands.
4. **The native-mod weave** — DONE: `tools/native_mods/compat.py` (registry verification:
   SHA-256 against the pinned snapshot, revocations, minisign when published, a bounded safe
   unzip, matching manifests) and `tools/native_mods/weave.py` (staging + the hook-API gate)
   ported from the head; the dispatch-only `.github/workflows/weave.yml` fetches the pinned
   registry, downloads the release bundles, verifies them and builds the APK through this
   repository’s own weaver (`packaging/android/native_build.sh`, one horizon-mixins pass, with
   `native_mod_adapt.py` bringing the mods’ calls up to date; the compiler runs at the newest
   consumer pin, Skins 3D’s, and vendored crates propagate into horizon-android).
   **All 33 weave and build green** (CI run **42521156965**, artifact `woven-apk`, 33.6 MB;
   every bundle staged, none rejected): jei, horizon_physics, jade, veinminer, appleskin,
   status_effect_hud, toggle_sprint, inventory_profiles, shulker_peek, mousetricks, itemnew,
   eatinganimation, enchantmenthints, light_overlay, right_click_crops, smartf3, armor_status,
   crosshair_indicator, hitbox_plus, invisible_frames, freelook, voicechat, worldedit,
   horizonmatica, visuality_horizon, held_item_info, carryblock, chatheads, fast_items,
   customhud, coords_hud, biomecompass, skins_3d.
   The last three needed their engine features ported first, all landed here: `Mob.skin_3d`,
   the `remote::Game::skin_3d_model` hook point (called where a player’s skin is applied,
   drawn in place of the flat outer layer) and `Downloader::image` for Skins 3D;
   `NativeShaders::picked`/`picking` and the pick-by-name wiring in `shader_packs.rs` for
   Visuality Horizon and Horizon Physics. The weave also drove the stale `Cargo.lock` fix, the
   `thread_local!` move out of `game.rs`’s top level (the compiler’s macro policy) and five
   `native_mod_adapt` rules.

## Rebrand (Rezyth Launcher)

The app is Rezyth Launcher with the applicationId `android.horizon.ramil` (commit `6b6e34847`):
the label, the shared-storage folder (`Rezyth`, `RezythDev`/`RezythPreview` for the variants),
the APK names (`rezyth-*.apk`), the `rezyth-apk` artifact and every user-visible string. The CI
builds the app itself now (no Preview suffix: this repository is the app's own). The
shared-folder migration takes a `Starling` (or `Horizon` Android) folder over on the first
start. Kept on purpose: the internal Java package and JNI entry points, the `.starling` file
association, `StarlingVisuals.zip`, the `STARLING_*` secret names and the update feed's
`starling-*` asset matching (the upstream repositories it asks publish those names). Verified
green: `android.yml` run **42561604025** and `weave.yml` run **42561592346** at `6b6e34847`
(artifacts `rezyth-apk` and `woven-apk`, both `rezyth-0.2.3-arm64-v8a.apk`).

## Resource pack gallery (Modrinth)

The resource pack screen's footer is three buttons — **Download Packs...**,
Open Pack Folder and Done — centred as one group (150 wide each when the frame
has room, narrower on phones); Download Packs opens an options page that
searches Modrinth's resource packs (resource packs only), installs one into
`resourcepacks/` with its SHA-512 checked, auto-enables it (the selection gains
it and the rebuilt screen commits it on Done) and checks the packs installed
from the gallery for updates (`pack-gallery.json` in the game folder remembers
what each file came from; a newer version for the game's own version wins). The
page is a normal options list: a search box (a pause after typing starts the
search), a row per result laid out **Icon > Name > Download** — the pack's
Modrinth icon is downloaded once per search (its URL also kept in the sidecar,
so update rows have one before a search) and written into an atlas pack-icon
slot as the row is drawn, with `unknown_pack` until it arrives; the name button
opens the pack's Modrinth page, the 86-pixel download button shows Install /
Installed / the download's percentage, author and downloads run under them, and
update rows carry an Update button. Then Load More and a Modrinth credit line.
The screen's gallery mode is on for resource packs and off for Create World's
data packs, so the vanilla path — the parity fixture's included — stays byte
for byte as it was and `test_pack_screen_parity.py`'s draw-call oracle holds.

Where it lives: the gallery state, the icons and the Modrinth answers
(`horizon-client-render/src/pack_gallery.rs`); the workers, icon downloads,
downloads, sidecar, update check and auto-enable
(`horizon-client/src/game/pack_gallery.rs`, one `GalleryOps` per options UI:
the game's `Game` and the title's `Loading`); the page (`Page::PackGallery`),
its rows, search box, acts/events and the icon-row drawing (`options_ui.rs`);
the `gallery` footer mode and `Action::OpenGallery` (`pack_screen.rs`);
`horizon.pack_gallery.*` in the bundled `en_us.json`.

Verified green: `android.yml` run **42743845481** at `73d41562d` (the icons and
the centred footer; artifact `rezyth-apk`), on top of the gallery's own runs
**42666756002** at `0cc25e9a9` and `weave.yml` **42668749214** at `14ae55366`
(artifact `woven-apk`, 33.7 MB, all 33 native mods woven).

## Volt Settings hub (the full screen from Volt Launcher)

The **Volt Settings** button (the in-game Options row and the main menu) now opens
Volt Launcher 0.9.9's split-pane frosted-glass screen itself: the left rail with
`Volt Settings`, its search field and the eight smooth-gliding tabs (Boost,
Visuals, Shaders, HUD & QoL, Chat & Voice, Maps, Controls, Extras), the right
glass groups with checkboxes, sliders, `< value >` cyclers and action buttons,
the bottom description + Cost dock, and the warm amber Undo / violet Done pair.
The Extras tab's Mod Menu opens the second screen: the pure-Rust 49-mod catalog
with its pixel-art logos, categories, per-mod config rows and engine sync.

Where it lives (all ported from `entitybrian69-bit/Volt-launcher`,
`arena/0cae2b73-volt-launcher`): `options_volt_pages.rs` (the `VoltAct` actions,
sections and row lists); `volt_settings_ui.rs` (the screen: layout, animations,
draw/click/key/scroll/type-text, `VoltUiAction`); `volt_mods.rs` (catalog,
`upstream_icon` logos, `VoltModCatalogState`, `apply_mod_to_engine` /
`sync_catalog_from_engine`, `ModMenuUi`); `volt_qol.rs` (the QoL/Chat/Visuals/
Boost+/Mods config structs and enums); `horizon-shader-api/src/volt_shaders.rs`
(`VoltShadersConfig`, profiles and enums); the mod logos in
`crates/horizon-client-render/assets/mod_icons/` (18 PNGs + licences/sources).
Rezyth's `Extras` gained the 40 backing fields (minimap, waypoints, perf HUD,
the Volt configs, `mod_catalog`), the 25 shader/perf `Item`s and the 28
Minimap/Waypoint `Item`s with their rows, cycles and persistence; `settings.rs`
gained the 25 `Opt`s (shader sliders, touch presets, resolution scale); the
options UI wires `Page::VoltSettings` to the screen and adds `Page::VoltModMenu`
for the Mod Menu; `key.volt.zoom/freelook/world_map` joined the key binds.
Rezyth has no touch-settings store and no world-map/waypoints-manager screens,
so those rows stay inert (they still show Volt's labels and descriptions) and
the two actions are no-ops; everything else mirrors the reference.

Verified green: `android.yml` run **42896093289** at `70bd540c0` (the whole
port) and run **42898376706** at `4854259fa` (the Volt button face, the restored
keybind test and this ledger); artifact `rezyth-apk` both times.

## Android signing: one key, so updates install

Every build of this repository now signs with one stable debug keystore, set as the
repository secret `STARLING_DEBUG_KEYSTORE_BASE64` (JKS, alias `horizon-debug`,
password `android`, CN=Horizon Android debug, cert SHA-256
`39:93:CE:E1:7F:D5:26:D1:A7:C0:88:42:D2:D8:B3:7A:58:6F:54:C0:1E:88:99:A7:34:E4:2C:D0:84:A7:AA:66`).
The secret had never been set, so `build_apk.py` generated a throwaway keystore on
every run: each APK had a different signature and Android refused to update over the
last one. The workflow now fails when the secret is missing on this repository's own
builds (a fork's pull request keeps its throwaway key and gets a warning), prints the
keystore's certificate as it unpacks, and `build_apk.py` runs `apksigner verify
--print-certs`, so every run's log names its signer. An app installed from an older,
throwaway-signed build must be uninstalled once; from then on every build updates over
the last. The native-mod APKs (`native_build.sh`) run after the same unpack step and
use the same key.

Verified: `android.yml` runs **42911549966** and **42912956822** at `f21b1b1e2`, two
independent builds, both `Signer #1 certificate SHA-256 digest:
3993cee17fd526d1a7c08842d2d8b37a586f54c01e8899a734e42cd084a7aa66`.

## Gallery icons: retried, throttled, and bigger

Some rows in the Modrinth gallery kept the `unknown_pack` fallback (the
cobblestone-looking texture) while others loaded. Three things fed that: a
project was marked requested *before* its fetch and a failure was dropped, so a
dropped mobile connection or a rate limit left the row on the fallback for the
rest of the session; the whole page fetched at once, each icon opening its own
`ureq` connection (a cold TLS handshake apiece); and the 1 MiB read cap
silently discarded any icon above it. `game/pack_gallery.rs` now fetches through
four lanes over one shared keep-alive agent, retries a failure up to three times
three seconds apart, clears the failure counts on a fresh search, accepts icons
up to 4 MiB and logs the URL when tries run out; the draw side is unchanged
(`options_ui.rs` still writes the icon into an atlas pack-icon slot, with
`unknown_pack` only until pixels arrive). Verified green: `android.yml` run
**42938084338** at `f93da8f56`.

## Volt Settings: no Shaders or Controls tabs

The split-pane hub's sidebar is down to six options — Boost, Visuals,
HUD & QoL, Chat & Voice, Maps, Extras. The **Shaders** and **Controls** tabs and
their screens are removed: the shader slider groups and the touch/gamepad groups
are gone from `tab_groups`, `VoltTab::ALL` is six, and the
`VoltRowKind::OpenPadBinds`/`OpenTouchEditor` rows, their `VoltUiAction`s and
the options-layer handlers for the Pad Binds and touch-layout screens went with
them. Nothing else changed: the Visuals tab still carries the Volt Shaders
master, shader-pack and video-settings entries, Maps keeps its minimap/waypoint
rows, and the standalone Controller/Pad Binds screen and the touch systems stay
reachable from the rest of the options menu. Verified green: `android.yml` run
**42996117474** at `e622fdc5b`.

## Verify

```sh
gh workflow run android.yml -R johnrenonasuncion-Ramil/Rezyth-Launcher --ref feat/volt-harvest
```

Every port reuses this game's own hooks (`with_activity`, `platform::Hooks`, `logging`, the
Options screens) and its translation files; no head file was copied verbatim where its module
layout or its launcher's context differed.
