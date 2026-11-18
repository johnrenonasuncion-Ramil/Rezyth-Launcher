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

The weave APK was the exception: `weave.yml` never unpacked the secret, and
`native_build.sh` builds in its own cache folder (where `build_apk.py` looks for the
key, not the checkout's), so every woven build generated a throwaway key of its own
(the 2026-11-12 21:59Z run signed with `d983728a...`) and could not install over the
standard APK. `weave.yml` now unpacks the same secret to `target/android/debug.keystore`
like android.yml, and `native_build.sh` carries the source tree's key into the build
folder it runs in — which also makes android.yml's own native-mod builds (`native_mods:`
input) share the key. Verified: weave run **43327935765** at `dec81bce9`, `Signer #1
certificate SHA-256 digest: 3993cee1...`, artifact `woven-apk` — the same signer as the
standard APK, so either installs over the other.

## The woven APK is the branch's default build

Every push to `feat/volt-harvest` now runs the weave workflow: its trigger's
`paths` filter is gone, so the 33-mod APK is built automatically (the standard
APK stays a manual dispatch on this branch), and nothing needs bumping to
rebuild it — the `.weave-trigger` sentinel is no longer read. Verified: the
trigger change's own push built it (weave run **43335963941** at `a60a3a3e3`),
and a later push with no tooling in it builds it too.

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

## Gallery icons: the WebP, GIF and JPEG reader, and no size cap

Modrinth serves most resource packs' icons as WebP — 269 of 400 in a survey
of the search API on 2026-11-13; 112 were PNG, 17 GIF, 2 JPEG — and the
reader was PNG-only, so two rows in three kept `unknown_pack` for good.
`hud::decode_bytes` now dispatches on the file's signature to WebP, GIF or
JPEG decoders (`gif`, `image-webp`, `jpeg-decoder` in
horizon-client-render) beside PNG; an animation decodes its first frame. The
gallery's fetcher reads a whole response whatever its size (the 4 MiB cap is
gone), and reopening the page clears the failed-icon counts, so one flaky
moment cannot strand a row. Three tests pin the codecs (a transparent 1x1
GIF, a baseline 1x1 JPEG, a WebP round-tripped through the encoder).

It pulled along the fork's first format and lint pass, on a temporary
`lockfile-bootstrap.yml` workflow (the authoring tree has no Rust toolchain);
that run resolved `Cargo.lock` for the new crates, formatted 22 files the
fork had never rustfmt'ed, and cleared 14 lints in the touched crates —
`manual_flatten` and a needless borrow in the Volt screens, two useless
`String` conversions in extras, two unused bindings and a
`field_reassign_with_default` in the Mod Menu, `chunks_exact`→`as_chunks` in
the new decoders, the audio feature-off build's unreachable match arm, and
`native_shaders.picked` (kept: native mods read it). The stale
`horizon_settings_hang_off_options_...` test — it still expected the buttons
from before the Volt Settings icon work — passes again too.

Verified green: weave run **43462894439** (the woven APK) and
lockfile-bootstrap run **43462894638** at `0bc2f6dcb` — the render crate
clippy-clean in both feature states, 1231 tests passing.

## Settings screens: vanilla Minecraft styling

The Mod Menu (the hub's Extras tab opens it) follows the same restyle: its
modal sits on vanilla's tiled list background, the preset gear and Done are
vanilla buttons, the search box is the vanilla text field, and its palette is
vanilla's — dark translucent panels, black edges/separators, white
highlights, its green and red for on/off, and white/grey text. The 33-mod
woven APK was rebuilt over all of it (weave run **43156179865**, artifact
`woven-apk`), alongside the standard APK (android run **43156345217**,
artifact `rezyth-apk`), both at `70b26855f`.

The look's second pass roomed and enlarged the same module: rows are 24 px
with vanilla's own 20 px button height on them (`ROW`, and `CONTROL_W`/
`CONTROL_H` 112/16 -> 128/20), the plate and its description strip carry a
vanilla container's black frame over a gentle wash, checkboxes centre in
their row and use vanilla's highlighted sprites while pointed at, the
`< value >` cycle buttons size to their text and fit it so the arrows are
always fully visible (no more scrolling labels), and the page/tab rails take
a vanilla list row's height on tall screens — squeezed on short ones so they
never grow over Undo/Done. Both APKs were rebuilt over it at `f9bb02022`:
weave run **43315607842** (artifact `woven-apk`) and android run
**43315751964** (artifact `rezyth-apk`), both green.


The split-pane design module (`graphics_style.rs`, shared by the Horizon
Graphics screens, every Volt Settings tab and the Mod Menu) now draws vanilla
Minecraft's widgets instead of the frosted-glass controls, with every size,
position and layout call untouched. Plates and the description dock tile
vanilla's list background (`menu_list_background` / its in-world variant; the
call sites pass `in_world`) and group panels are gone, as vanilla option lists
have none; buttons (Done, Undo, row buttons) are the grey vanilla button
sprite with its highlighted and disabled states and scrolling label; cycle
buttons are vanilla buttons labelled `< value >`; sliders are vanilla's
track/handle sprites with the value centred; checkboxes and the Mod Menu's
ON/OFF switch are vanilla's checkbox sprites; scrollbars use vanilla's
scroller sprites; the search box is the vanilla text field; separators are
black rules and the accent/hover/selection colours are vanilla's whites,
greys and dark selection fill. Verified green: `android.yml` run
**43110705093** at `c2cd36be8`.

## Volt Settings icons in the title and pause rows

The title screen's icon row (Friends, Language, Accessibility) and the pause
menu's (report bugs, send feedback, friends, player reporting) each gain a Volt
Settings button: a fourth and fifth icon that draws the 20x20 sprite from the
user's `button.zip` over its 20x20 button and opens the split-pane screen. The
sprite is bundled as `crates/horizon-client-render/assets/volt_settings.png`
and decoded into the HUD atlas as `gui/sprites/icon/volt_settings`
(`hud.rs`). The title's icon keeps id 9 and returns `Action::VoltSettings`,
with the full-width "Volt Settings..." button below Options and Quit Game
removed; the pause icon (always active, id 16) opens the options at
`Page::VoltSettings`, and both rows' narration, tooltips, focus ids and the
tests pinning their geometry are updated. Verified green: `android.yml` run
**43024144898** at `8fa62bde3`.

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

## Title and pause: the Volt Settings icon button

The two icon rows both gained Horizon's own button, drawn with the sprite
from the user's `button.zip` (a 20x20 glyph, shipped as
`crates/horizon-client-render/assets/volt_settings.png` and packed into the
HUD atlas as `gui/sprites/icon/volt_settings`): the title screen's row is now
Friends, Language, Accessibility, Volt Settings (`title.rs`, ids 5-7 and 9,
recentred to `cx - 46 + i*24`), and the pause menu's is Bugs, Feedback,
Friends, Player Reporting, Volt Settings (`game/screens.rs`, ids 12, 13, 10,
15, 16, recentred to `x0 + 48 + i*24`). The Volt icon opens the split-pane
Volt Settings screen in both places (`Action::VoltSettings` from the title,
`open_options_at(false, Page::VoltSettings)` from the pause menu), carries
the `horizon.pages.volt` tooltip/narration, and sits in keyboard and
controller focus from the pause menu. The title screen's full-width
"Volt Settings..." button is removed. Verified green: `android.yml` run
**43024144898** at `8fa62bde3`.

## Every container screen: Bedrock's X close button

Every interactive block/entity screen carries a small close button in its
frame's top-right corner, inside it, the place Bedrock draws its X: a
14x14 vanilla widget button (`widget/button`, `_highlighted` under the
cursor) with an "x", narrated as `gui.close` ("Close"). A left press runs
that screen's own Escape close (the same calls `key_pressed` makes); a
right press is swallowed as a widget's is, so it reaches no slot and
drops nothing. One helper, `game/close_button.rs`, derives the frame
(`container_close_rect`: the 176-px origin, the beacon's 230 and the
merchant's 276 by their shifts) and draws it from a single call in
`draw_inventory` (chests, barrels, ender/copper chests, shulkers,
hoppers, droppers, dispensers, crafters, furnaces, mount inventories,
the player inventory and the crafting table), `draw_station` (brewing,
enchanting, anvil, grindstone, smithing, stonecutter, loom, cartography),
`draw_beacon` and `draw_merchant`; the screens that lay themselves out
carry their own rect: `draw_command_block` (chain/repeating too),
`draw_block_editor` (structure, jigsaw and test blocks) and `draw_book`
(a lectern's book and a held written book's view). Presses route through
`inventory_press`, `command_block_click`, `block_editor_click` and
`book_click`; a native mod's screen keeps its own corner (no X), as does
the creative menu's own Bedrock layout. The chest screens' sort button
moves 18 px left (160 -> 142) so the two never overlap. Two tests pin the
placement and the press rules; the module's finger-print (`gui.close`)
is in the shipped `libhorizon_android.so`. Along the way the checks
workflow's horizon-ctl test found `docs/horizon-ctl-commands.md` stale
(regenerated at `cb1a2b14a`); both findings were the fork's first full
format/lint/test pass, not new failures.

Verified green: weave run **43534383948** at `9bae44008` (the woven APK,
signed `3993cee1…`) and checks run **43534416600** at `9bae44008`
(rustfmt, clippy in both feature states, the whole workspace test suite
and doc tests: 155 test binaries green, 0 failures, including
`game::close_button::tests::the_close_button_sits_inside_the_frames_top_right_corner`
and `game::close_button::tests::a_left_press_closes_and_a_right_press_is_only_swallowed`).

## The Volt Settings icon: the user's cog from Google Drive

The title screen's icon row (id 9) and the pause menu draw the Volt
Settings button's icon from
`crates/horizon-client-render/assets/volt_settings.png` (the atlas sprite
`gui/sprites/icon/volt_settings`, compiled in with `include_bytes!`). The
old 20x20 sliders glyph is replaced by the user's `cog.png`, fetched from
their Google Drive through the NoClick MCP connection (search, download),
ferried bit-for-bit into the tree over a signed bucket link, and otherwise
left untouched: a 20x20 gray+alpha PNG, 5937 bytes, md5
`5004fa1df95782fe814fe1aa5c14621e` — the same hash the bucket stored as its
ETag, and the same bytes the game decodes (its C2PA id
`urn:c2pa:8fbe282f-…`, twice, is in the woven APK's
`libhorizon_android.so`; the 5937 bytes compare `cmp`-identical to the
tree's copy, and the build before it has none). No code changed: the
atlas reads the sprite's native size, 20x20, unchanged. Verified green:
checks run **43548990433** and weave run **43548831117** at `0cb8ea54a`.

## Mod Menu: the `modmenu-1.1.2.hnmod` native mod, built into the game

TerraformersMC's Mod Menu — a port to Horizon's native mod format by
DeportedToHell, shipped as `modmenu-1.1.2.hnmod` — is a feature of the
game itself now (MIT; [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md),
`licenses/modmenu-MIT.txt`). Its mixin sources became
`game/modmenu.rs` (the runtime), `game/modmenu_glue.rs` (the game side),
`game/modmenu_view.rs`, `game/modmenu_sprites.rs`, and
`title/modmenu_title.rs` for the title screen's side — the title lives in
the client library, which cannot reach the game, so the game feeds its
statics once a frame and the title reads the presses back, as the mod did.
Every hook the mixin fired is a call in the game now: `native_frame`
(frame), `native_screen_render_post` (render), the seven screen input
hooks, the key hook and `clipboard::title_edit`. `native_on` is "always
there" (the Options > Mods off-switch went with the mod), the catalog
index is read from `mods-index.json` beside the game when the launcher
leaves one (this game has no registry client of its own), catalog icons
fetch through a local ureq helper with a capped chunked read, and images
decode through `hud::decode_bytes`. The Mod Menu lists itself like any
mod, its own options behind Configure. Its settings keep the mod's
per-key files (`horizon-mods/.horizon-mod-data/modmenu/`), so a player's
config — and a key bound while it was a mod — carry over.

Conflicts with this fork's own changes, resolved: the title's icon row
was four wide (Friends, Language, Accessibility, Volt Settings) and the
mod's icon style wanted the same corner — the row is five, recentred
(Volt Settings and Mod Menu), with every `mods_button_style` supported
(Below Realms / Replace Realms / Adjacent / Icon) as `title.rs` builds
it, and the two tests pinning the row moved with it. The pause menu's row
is six now (its five, 12 px left, Mod Menu last) and the mod's `Insert`
row keeps its shifts; its Mods button joins `pause_buttons`, with the
icon and tooltip drawn by `modmenu_view`'s `pause_extras` (the pause
menu's `screen_render_post`) and the cover hiding the menu underneath,
exactly as the mod's priority-200 injections did. The screen's
`native_apply_screen_blur` became the tree's own `batch.blur_at` mark,
and `Button::action`/`does` (which this fork's title never had) are not
used: the Mods button opens through `click_action` and `OPEN_REQUEST`.
Its catalog also lacks `categories` in this fork's index: a Library
badge comes from the mod's own `modmenu.json`, and a catalog project
adds none.

Verified green: the temporary modmenu-check run **43598045177** at
`491a58850` (clippy `-D warnings` for horizon-client, horizon-client-render
and horizon-client-lib, and the tests: 756 gameplay ones — the five ported
`game::modmenu::tests` among them — and the client library's 1,509, all
passing; `cargo fmt` clean, its patch empty) and the weave run
**43598045247** at the same commit (the woven APK). The first run's two
findings were fixed in `491a58850`: `Read::take` cannot be called on the
boxed reader `ureq` returns (the download reads in chunks now), and the
logo/icon geometry test still pinned the four-icon row.

## Chests and barrels: vanilla's five-tick opener recheck

A container's opener count is recounted now, as vanilla recounts it. The fork had the chest,
ender chest and barrel tick arm that calls `recheck_container_openers` and reschedules itself
while anyone has the container open, but nothing scheduled that tick on the first opener and
nothing answered the recount: a close the game missed (a death, a teleport, a dropped
connection) left a stale count, and with it a chest's lid or a barrel's `open` block state —
barrels had no tick arm at all — stuck open.

- `game/block_entity_anims.rs`: `openers_at` and `recheck_opener_count`
  (`ContainerOpenersCounter.getEntitiesWithContainerOpen` / `recheckOpeners`), vanilla's
  five-tick schedule in `count_opener` when a count rises from zero, and three tests for the
  recount (a stale count is corrected, an agreeing recount changes nothing, a double chest
  answers for both halves).
- `game.rs`: `GameLevel` carries `open_containers` — the local player's and the guests' menus,
  a double chest as both halves, spectators aside — beside a mutable `lid_openers`, and
  implements `recheck_container_openers` for chests, ender chests and barrels.
- `horizon-world/src/level/tick.rs`: the `Class::Barrel` arm, upstream's: recount, and on the
  count crossing zero the barrel's own sound (volume 0.5, pitch 0.9–1.0 from the level random),
  `updateBlockState` and the `container_open`/`container_close` game events.

Verified green: checks run **43630512385** at `9eaef6023` (fmt, clippy `-D warnings`, the three
new `game::block_entity_anims::opener_recheck_tests` among the gameplay tests, and the client
library's 1,509) and the weave run **43630436075** at the same commit.

A second round closed the two gaps the first left. A shulker box's lid still stuck: its count
never rode the opener counter and no tick arm recounted it. And — for every container — the
recheck's correction never reached the clients at all, because `runBlockEvents` broadcasts an
event only when the block reports it handled, and the world's block-level `triggerEvent`
answered false for chests and shulker boxes, whose *block entities* are what handle id 1 in
vanilla (`BaseEntityBlock.triggerEvent` forwards to the block entity).

- `horizon-world/src/level/pistons.rs`: `Sim::trigger_event` reports id-1 chest,
  ender-chest and shulker box events handled, so a corrected count is broadcast
  (`LevelEvent::BlockEntityEvent` → the server's own lid state and
  `ClientboundBlockEventPacket` for the window client and the guests, which re-run it through
  `client_block_event`).
- `horizon-world/src/level/tick.rs`: the `Class::ShulkerBox` tick arm (recount, broadcast a
  changed count, reschedule while open), and the chest arm broadcasts on a change only, as
  vanilla's `openerCountChanged` does.
- `game/block_entity_anims.rs`: a shulker box's count rides `count_opener` (the chests'
  counter), so its first opener schedules the five-tick recheck;
  `game.rs`'s `recheck_container_openers` answers for `Class::ShulkerBox`.
- `horizon-world/src/level/tests.rs`: a test pins that a chest's and a shulker box's counted
  lid event is handled and reaches the clients, and that another id reports nothing.

Verified green: checks run **43661338385** at `625c5bc32` (fmt, clippy `-D warnings`, the new
`level::tests::a_chests_lid_event_is_handled_and_reaches_the_clients` among the workspace's
tests) and the weave run **43661337993** at the same commit. The test's formatting follow-up
`0c071896e` is verified by checks run **43662784525** and weave run **43662539132**.

## The creative inventory: Bedrock's X on the item panel

The creative screen's item panel carries the same close button every container screen has: a
small vanilla button in the panel's top-right corner, inside it (`close_button::rect`), at or
above the scrollbar's track and clear of the search box, the tabs and the trash slot. The
Bedrock Inventory mode's two windows keep exactly one X: the item window carries it.

A left press closes as the screen's Escape does — the carried stack goes back to the inventory
and what does not fit is dropped (`AbstractContainerMenu.removed`) — a right press is swallowed,
and neither edge reaches a slot, tab or the scrollbar. `game.rs` draws it in
`draw_creative_window` (the item window only) and tests it first in `click_creative`;
`game/close_button.rs` adds a test pinning the rect inside the panel's top row, above the
scrollbar and clear of the search box and the trash. Its first check run found clippy's
`collapsible_if` on the carried-stack return (rust 1.100 wants one let-chain there), fixed in
`162b3632f` (verified by the full checks run **43643624746** at `8cfa4249a`).

## Pistons: the extended base survives their own move

A powered piston destroyed itself. Every `setBlock` that replaced a `moving_piston` ran
`MovingPistonBlock.destroy`'s extended-base removal, so the head cell's completion — the normal
end of every extension — took the extended base with it, and so did any `/setblock`, `/fill` or
explosion over a moving piston (horizon-rust/horizon#7863; upstream's `7a79e25da4` is the fix
this port follows). Sticky and normal pistons alike vanished as their move finished.

- `horizon-world/src/level/update.rs`: `chunk_set_block_state` no longer calls
  `destroy_moving_piston_base`; the function is `pub(crate)` for the player-break path now.
- `horizon-world/src/level/hooks.rs`: `player_destroy` runs it when a player breaks a moving
  piston, as `ServerPlayerGameMode.destroyBlock` does.
- `horizon-world/src/level/tests.rs`: the break test drives `player_destroy_block`, and a new
  test pins that a `setBlock` over a moving piston leaves a headless extended piston. (The
  level tests skip without the prepared vanilla data, so CI compiles them without running them;
  the fix is upstream's own, with its tests.)

## Bedrock Inventory: off by default, and the choice saves

The toggle could never persist: `OptionsFile` is table-driven, and its old key name
(`javaCreativeInventory`) was never a row — `bool` reads only table keys, so the read always
answered false (the option was hard-wired to the Bedrock screen) and `set` silently dropped the
write. On top of that the act pushed no `Event::Changed`, the only event that makes the game
save `options.txt`.

- `horizon-client-render/src/options_file.rs`: `b("bedrockInventory", false)` is a row now,
  beside the other booleans.
- `horizon-client-render/src/settings.rs`: the setting reads and writes that key directly, off
  by default — the Java-style creative screen unless the player opts into the Bedrock one.
- `horizon-client-render/src/options_ui.rs`: the toggle pushes `Event::Changed`, so the game
  saves at once. A round-trip test pins that on and off both survive save and load.

Verified green: checks run **43682318289** at `d1e7a8ded` (fmt, clippy, the workspace's tests —
760 gameplay, 1,509 client-library and 1,232 render ones among them, the new
`the_bedrock_inventory_choice_is_off_by_default_and_round_trips` included) and the weave run
**43682317740** at the same commit.

## Verify

```sh
gh workflow run android.yml -R johnrenonasuncion-Ramil/Rezyth-Launcher --ref feat/volt-harvest
```

Every port reuses this game's own hooks (`with_activity`, `platform::Hooks`, `logging`, the
Options screens) and its translation files; no head file was copied verbatim where its module
layout or its launcher's context differed.

## Rezyth Settings wire-up, waves 22-26 (branch feat/volt-harvest, 2026-11-17)

Each wave wires more Volt Settings rows to real engine consumers; verification is CI-only
(`gh workflow run ci.yml -R johnrenonasuncion-Ramil/Rezyth-Launcher --ref feat/volt-harvest`,
plus the weave push run building the APK).

- **Wave 22** (`50122aeef`): World Inspector panel (block id/pos/light/biome under the
  crosshair), Damage Direction arrow around the crosshair, Damage Popups floating over hit
  mobs. New `game/damage_hud.rs`; `hud_arrow`/`damage_popup_style` in `volt_features.rs`.
- **Wave 23** (`a21b8178c`): Trajectory arc gizmo, Spawn Light overlay boxes (red <= 0,
  amber <= 7), Slime Tech panel (slime-chunk roll, y < 40). New `game/volt_overlays.rs`;
  `throw_direction`/`trajectory_points`/`spawn_light_color`/`slime_spawns_in_chunk` in
  `volt_features.rs`.
- **Wave 24** (`c6898fc53`): Auto Tool hotbar-slot hint, Tool Break Shield warning panel,
  Enchant Guide (real enchant names on the enchanting buttons). `best_tool_slot`/
  `tool_break_warning` in `volt_features.rs`.
- **Wave 25** (`b7b8c7abc`): Volt Cam Framing Grid thirds, underwater one-pole low-pass and
  cave feedback-comb echo in the audio mixer's `mix()` (`one_pole_lpf`/`comb_echo` in
  `audio.rs`, gated per tick from game.rs). `framing_grid` in `volt_features.rs`.
- **Wave 26** (`c6b89ad52`): AppleSkin saturation number above the food bar; Volt Atlas
  master switch (`atlas_enabled` gates the world map image/arrow), coordinate grid overlay
  (`atlas_grid`, `atlas_grid_spacing`), and the four view modes in `Minimap::compose`
  (Surface/HeightRelief/BiomeTint/SlimeChunks; tiles now record `chunk.biome`, `Minimap`
  carries the world seed; `biome_tint` in `volt_features.rs`).

Still unwired by design (need subsystems that do not exist yet, not faked): the 11 Volt
Shaders WGSL rows (`horizon-shader-api/src/volt_shaders.rs` is config-only), the 9 voice rows
(no capture pipeline), the 2 replay rows (no recorder), wall occlusion (needs a per-tick
occlusion probe into the audio mixer), and the ChatHeads catalog mod.

- **Wave 27** (`65e142174`): Optic Zoom (QolZoom/QolZoomFov/QolZoomSmooth — hold
  `key.volt.zoom`, FOV target divides by 100/pct, eased when smooth, turning scales
  down; native mod zoom still wins) and 360° Freelook Orbit (QolFreelook — hold
  `key.volt.freelook`: the view is captured as the body's facing, movement uses
  `body_yaw_pitch()`, the camera orbits freely, release snaps back).
- **Wave 28** (`4f7022af4`): Toggle Sprint/Sneak HUD (latched toggles read out bottom-left),
  Keystrokes Overlay (movement keys grid, lit while held, labels from the bindings),
  Low Shield & Low Fire (blocking shield viewmodel drops 0.15; the first-person fire
  overlay drops a quarter screen via `ScreenInputs.fire_drop`).
- **Wave 29** (`b06e76804`): Shulker & Bundle Peek (ModShulkerPreview / catalog
  ShulkerBoxTooltip) — the box's `minecraft:container` contents as a 9x3 grid under
  the tooltip name lines, via the pure `shulker_cell` helper.
- **Wave 30** (`8eed1ccd9`): Wall occlusion (QolWallOcclusion /
  `volt_qol.sound_wall_occlusion`) — per-tick 26-direction occlusion probe
  from the eye via `clip_ray` (8 blocks) into `Audio::set_wall_occlusion`;
  `play_seeded` multiplies positional volume by `wall_factor(at)` (nearest
  dir to `at - listener`); pure test pins the normalized dir table.

## Rezyth Settings — direction change (user decision)
- The Volt Shaders render wiring (former waves 31–34: the WGSL post grade in
  the HDR present pass + SDR blit, clouds mode/coverage, glowing ores, night
  sky, Auto-PBR, Bloom, Torch Light Style, Post-Rain Rainbow) was **reverted**:
  the branch base is back to the wave-30 state (`16bcbefb6`, CI run
  44030829847, 5600 passed / 0 failed). The abandoned implementation is kept
  on the local branch `backup/volt-shaders-wiring` for reference.
- The **Enable Shaders** toggle (`Item::VoltShaderEnable`) is removed from the
  Rezyth Settings Visuals tab and the item enum; the `voltShadersEnabled`
  options.txt key still parses for compatibility.
- The Shaders tab's remaining rows stay as they were before the wiring work
  (config-only). All other tabs' wired rows (waves 1–30) are untouched.
- **Wave 36** (`c4a884e88`, CI-green run 44160353557, 5607 passed / 0 failed):
  3D Proximity Voice — `voice.rs` (pure: RMS, the activity gate, mic/output
  gains, proximity falloff) + the Mixer's voice state shared by a new cpal
  input stream (`capture_voice`: PTT while `key.volt.voice_ptt` (V) is held,
  else the VAD threshold; mic-gained samples into a rate-bounded ring) and the
  output mix, which drains it while the loopback test path is on (resampled,
  cave-muffled, at the output gain). The voice HUD button (VoiceHudBtn) shows
  the mode and a live Speaking/Muted line. Wires VoiceEnable, VoiceModePtt,
  VoiceVad, VoiceMicGain, VoiceOutGain, VoiceDistance, VoiceCaveMuffle,
  VoiceHudBtn, VoiceLoopback. The network transport (voice_port UDP) is not
  built; the loopback makes every setting audible on your own voice.
- **Wave 37** (`7dc0eda40`, CI-green run 44170343174, 5609 passed / 0 failed):
  Replay Recording + On-Screen Replay Button — `ReplayTrack`/`ReplaySample`
  (serde JSON, tested round-trip) sampled once per world tick while recording;
  the button (or `key.volt.replay`/R, or a bound touch button) cycles
  idle → recording ⇄ paused → idle, writing the take to the player's
  `replays/` folder and reporting in chat. The floating button (QolReplayButton)
  draws at its configured position/size/opacity with a state dot (grey idle,
  blinking red recording, amber paused). The launcher-side replay player
  (timeline, speeds, free camera) is not built; the takes are real camera
  tracks.
- **Wave 38** (`58a34bdcb`, CI-green run 44182002853, 5609 passed / 0 failed):
  ChatHeads (the catalog mod) — the player-chat packet resolves the sender's
  tab-list name, threaded through the chat-delay queue to `Chat`; each
  message's first line draws the sender's 8x8 skin face + hat layer
  (`tab_list::face`) left of the text. `Chat.sender_skins` (name -> face
  texture) is refreshed per frame from the resolved skins while the ChatHeads
  mod is on, the local player included. The catalog toggle now draws real
  heads; no new options.txt key (the catalog's own persistence covers it).
- **Wave 39** (`a74ba185a`, CI-green run 44186839564, 5609 passed / 0 failed):
  the replay button's touch gestures — a tap acts (start, pause, end), a drag
  moves the button (position follows the finger), a held drag (500 ms)
  resizes it up-down and fades it left-right within the config's ranges; the
  release after a drag saves options.txt. Desktop keeps the R key; the
  gestures are touch-only.
- **Wave 40** (`5ee7a9576`, CI-green run 44206546639): the launcher's
  **Replays page** — a Home tab next to Screenshots (both skins) listing the
  takes the recorder saved (`replays/*.voltreplay.json`), with a viewer: the
  camera's path from above (dots along the track, the playhead's marker), a
  scrubber (press/drag/release to seek), play/pause, speeds 0.5–4×, and the
  interpolated pose readout at the playhead; playback advances on the
  launcher's 80 ms frame tick; Escape closes. `launcher/src/replays.rs` is the
  pure list/load/pose layer, tested. The in-world free camera (watching the
  take from inside the live game) is not built.
- **Wave 41** (`d8d5f16c5`, CI-green weave 44221602339 + checks 44221602331):
  3D Proximity Voice over the session connection — `horizon-protocol/src/voice.rs`
  defines the `horizon:voice` custom payload: a guest's frame (capture rate,
  count, mono f32 samples, capped at 8192), the hub's relay (sender uuid and
  server position in front), and their parsers (garbage refused), tested.
  `audio.rs` feeds the same gated, mic-gained samples to a network ring
  (`voice_net`, 200 ms bound), drained per tick by `take_voice_net`; relayed
  voices mix at their distance gain from the listener, the output gain and the
  cave muffle, with per-speaker resampling/muffle state and bounded queues
  (1 s each, 64 speakers); `push_remote_voice`/`clear_remote_voice(s)` manage
  them, and a dry ring plays silence. `remote.rs` sends the captured chunks
  each tick to a Horizon server (`r.mod_net`), receives the relays, and
  forgets a player's voice on `PlayerInfoRemove` and every voice on
  disconnect. `game/lan_voice.rs`: the server validates and relays each chunk
  to the other Horizon players in the same dimension with the sender's uuid
  and server position, at most 24 chunks per connection per tick. A vanilla
  server ignores the unknown channel, so voice stays between Horizon players;
  no UDP `voice_port`.
- **Wave 42** (`3e3627fab`, fixups `31d29b0d6` + `a720abd11`; CI-green weave
  44225100410 + 44226948014 and checks 44226948551, 5612 passed / 0 failed):
  in-game replay playback and the recorder's tick — `ReplayTrack::pose_at`
  interpolates the recorded camera at a tick (short-way yaw around ±180),
  tested; the recorder's sampling moved from `apply_settings` (event-driven,
  so takes were rarely sampled) to `tick_remote`, giving one camera sample per
  world tick in single player, LAN and servers. `key.volt.replay_watch` (K)
  starts the newest take in `replays/` in its own dimension (recording,
  dimension, empty and malformed guards, translated messages) and the key
  again stops it; the HUD shows its name, `m:ss / m:ss` and the stop key; the
  render camera flies the take's pose (block outline hidden); the watch clears
  on world open/close/dimension change and when a new recording starts. The
  Replay Recording row's description now matches the feature (launcher page
  plus the in-game camera, no third-person promise). The fixups add the
  rustfmt wrap, the dry-voice-ring silence with its test, and the clippy
  let-chain.
- **Direction (user decision)**: the Shaders tab's remaining rows stay
  config-only (the WGSL revert stands; the abandoned wiring remains on the
  local `backup/volt-shaders-wiring` branch).
