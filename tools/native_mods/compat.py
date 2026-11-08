#!/usr/bin/env python3
"""Offline native-mod registry verification for Starling's weave.

Ported from the Volt tree (pull request #2's head) with Starling's own
adaptation layer kept where it is (`packaging/android/native_mod_adapt.py`,
run by `native_build.sh`): this module only verifies a downloaded `.hnmod`
against a pinned registry snapshot and stages it for that builder.

This is a development/review tool, not an installer or a security sandbox.
"""
from __future__ import annotations
import base64
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[2]
ABIS = ("arm64-v8a", "armeabi-v7a")
ID = re.compile(r"[a-z][a-z0-9_]{0,63}\Z")
SHA = re.compile(r"[0-9a-f]{64}\Z")
REV = re.compile(r"[0-9a-f]{40}\Z")
MAX_ARCHIVE = 16 * 1024 * 1024
MAX_EXPANDED = 64 * 1024 * 1024
MAX_FILES = 2048


class Rejected(ValueError):
    pass


def read_entry(registry: Path, mod_id: str) -> dict:
    if not ID.fullmatch(mod_id):
        raise Rejected("invalid mod id")
    path = registry / "mods" / f"{mod_id}.toml"
    entry = tomllib.loads(path.read_text(encoding="utf-8"))
    if entry.get("id") != mod_id or entry.get("kind") != "native-mod":
        raise Rejected("entry is not the requested native mod")
    return entry


def check_revoked(registry: Path, mod_id: str, digest: str) -> None:
    path = registry / "revoked.toml"
    if not path.is_file():
        raise Rejected("pinned registry revocation list is missing")
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    for item in data.get("revoked", []):
        if item.get("id") == mod_id and (not item.get("sha256") or item["sha256"] == digest):
            raise Rejected("bundle is revoked in pinned registry")


def verify_minisign(data: bytes, key_b64: str, signature: str) -> None:
    """Verify a minisign signature block over `data` with the author key.

    Supports the two minisign algorithms: `Ed` (signature over the raw file)
    and `ED` (signature over the file's Blake2b-512 digest). The trusted
    comment's global signature is verified too. Fails closed on anything
    unexpected, including a missing cryptography backend.
    """
    try:
        from cryptography.exceptions import InvalidSignature
        from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
    except ImportError:
        raise Rejected("signed bundle: minisign verification needs the 'cryptography' package")
    try:
        raw_key = base64.b64decode(key_b64.strip(), validate=True)
    except (ValueError, TypeError):
        raise Rejected("signed bundle: author key is not valid base64")
    if len(raw_key) != 42 or raw_key[:2] != b"Ed":
        raise Rejected("signed bundle: unsupported author key format")
    key_id, public = raw_key[2:10], raw_key[10:]
    lines = [line.strip() for line in signature.strip().splitlines() if line.strip()]
    payloads = [
        line
        for line in lines
        if not line.startswith(("untrusted comment:", "trusted comment:"))
    ]
    trusted = [
        line[len("trusted comment:") :].strip()
        for line in lines
        if line.startswith("trusted comment:")
    ]
    if len(payloads) != 2 or len(trusted) != 1:
        raise Rejected("signed bundle: malformed minisign signature block")
    try:
        record = base64.b64decode(payloads[0], validate=True)
        global_sig = base64.b64decode(payloads[1], validate=True)
    except (ValueError, TypeError):
        raise Rejected("signed bundle: signature is not valid base64")
    if len(record) != 74 or len(global_sig) != 64:
        raise Rejected("signed bundle: malformed minisign signature record")
    alg, sig_key_id, sig = record[:2], record[2:10], record[10:]
    if sig_key_id != key_id:
        raise Rejected("signed bundle: signature was made with a different key")
    if alg == b"ED":
        message = hashlib.blake2b(data, digest_size=64).digest()
    elif alg == b"Ed":
        message = data
    else:
        raise Rejected("signed bundle: unsupported minisign algorithm")
    verifier = Ed25519PublicKey.from_public_bytes(public)
    try:
        verifier.verify(sig, message)
        verifier.verify(global_sig, sig + trusted[0].encode())
    except InvalidSignature:
        raise Rejected("signed bundle: minisign signature does not verify")


def verified_extract(archive: Path, entry: dict, dest: Path, registry: Path) -> dict:
    """Unpack `archive` if it is the published bundle of `entry`; its metadata."""
    if archive.stat().st_size > MAX_ARCHIVE:
        raise Rejected("archive too large")
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    ver = next(
        (
            v
            for v in entry.get("versions", [])
            if v.get("game") == "26.3"
            and str(v.get("file", "")).endswith(".hnmod")
            and SHA.fullmatch(str(v.get("sha256", "")))
            and v["sha256"] == digest
        ),
        None,
    )
    if ver is None:
        raise Rejected("bundle SHA-256 does not match any pinned registry version")
    check_revoked(registry, entry["id"], digest)
    signature = str(ver.get("signature") or "")
    key = str(entry.get("key") or "")
    if signature or key:
        if not signature or not key:
            raise Rejected("signed bundle: signature and author key must both be present")
        verify_minisign(archive.read_bytes(), key, signature)
    seen = set()
    total = 0
    with zipfile.ZipFile(archive) as z:
        infos = z.infolist()
        if len(infos) > MAX_FILES:
            raise Rejected("too many ZIP entries")
        for info in infos:
            name = info.filename
            p = PurePosixPath(name)
            mode = info.external_attr >> 16
            if (
                not name
                or "\\" in name
                or p.is_absolute()
                or any(x in (".", "..") for x in p.parts)
                or name in seen
                or stat.S_ISLNK(mode)
                or info.flag_bits & 1
            ):
                raise Rejected("unsafe, duplicate, symlink or encrypted ZIP entry")
            seen.add(name)
            total += info.file_size
            if total > MAX_EXPANDED:
                raise Rejected("expanded bundle too large")
            if info.is_dir():
                continue
            if not (
                name == "horizon-native.json"
                or (len(p.parts) >= 3 and p.parts[:2] == ("mixins", entry["id"]))
            ):
                raise Rejected("unexpected bundle path")
            out = dest.joinpath(*p.parts)
            out.parent.mkdir(parents=True, exist_ok=True)
            with z.open(info) as src, out.open("xb") as dst:
                while chunk := src.read(64 * 1024):
                    dst.write(chunk)
    meta_path = dest / "horizon-native.json"
    manifest_path = dest / "mixins" / entry["id"] / "horizon-mixin.toml"
    if not meta_path.is_file() or not manifest_path.is_file():
        raise Rejected("bundle is missing native or mixin manifest")
    meta = json.loads(meta_path.read_text(encoding="utf-8"))
    if (
        meta.get("schema") != 1
        or meta.get("id") != entry["id"]
        or meta.get("version") != ver.get("version")
        or meta.get("game") != "26.3"
        or not REV.fullmatch(str(meta.get("horizon", "")))
        or not REV.fullmatch(str(meta.get("mixins", "")))
    ):
        raise Rejected("native manifest does not match the registry")
    manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    if (
        manifest.get("mod", {}).get("id") != meta["id"]
        or manifest.get("mod", {}).get("version") != meta["version"]
    ):
        raise Rejected("mixin identity differs from bundle")
    if manifest.get("compatibility", {}).get("horizon_revision") != meta["horizon"]:
        raise Rejected("mixin and bundle Horizon revisions differ")
    return {"meta": meta, "manifest": manifest, "sha256": digest, "manifest_path": manifest_path}


def check_hooks(data: dict, root: Path, commit: str, raw_targets: bool = False) -> bool:
    """Whether the staged manifest may be rebound to this tree's commit.

    A bundle that declares the native hook API level it needs may run against
    any newer commit serving at least that level and every hook it names; one
    that pins an exact Horizon revision without the declaration is only tried
    under the experimental raw-targets path, the mixin compiler judging.
    """
    meta, manifest = data["meta"], data["manifest"]
    if manifest.get("post_build") or manifest.get("post-build"):
        raise Rejected("post-build commands are not supported")
    if meta.get("horizon") == commit:
        return False
    allowed = {"mod", "compatibility", "injection", "module"}
    if raw_targets:
        # The head's release weave takes vendored crates too under its
        # raw-targets path; the compiler stages them and native_build.sh
        # propagates their dependencies into horizon-android.
        allowed.add("crate")
    if set(manifest) - allowed:
        raise Rejected("non-hook transformations are not supported")
    if "crate" in manifest and not raw_targets:
        raise Rejected("a bundle vendoring crates needs exact pinned source")
    hooks = meta.get("hooks")
    source = (root / "crates/horizon-client/src/game/native_hooks.rs").read_text()
    found = re.search(r"const NATIVE_HOOKS_API: u32 = (\d+);", source)
    if not found:
        raise Rejected("this tree does not state its native hook API version")
    available = json.loads((root / "native-hooks.json").read_text())
    names = {(item["name"], item["version"]) for item in available["hooks"]}
    if not isinstance(hooks, int) or isinstance(hooks, bool) or hooks < 1:
        injections = manifest.get("injection", [])
        named = [
            (inj.get("hook"), inj.get("hook_version"))
            for inj in injections
            if "target" not in inj
        ]
        missing = sorted({f"{h}/{v}" for (h, v) in named if (h, v) not in names})
        if raw_targets and not missing and injections:
            return True
        raise Rejected(
            f"bundle pins Horizon {meta.get('horizon', '')[:12]} without a hook-API "
            f"declaration; {len(injections)} injections, missing named hooks: {missing[:25]}"
        )
    served = int(found.group(1))
    if hooks > served and not raw_targets:
        raise Rejected(
            f"needs native hook API {hooks}; this tree serves {served}"
        )
    injections = manifest.get("injection", [])
    if not injections:
        raise Rejected("no explicit hook injections; an exact revision is required")
    for inj in injections:
        if "hook" in inj and (inj.get("hook"), inj.get("hook_version")) not in names:
            raise Rejected(f"named hook unavailable: {inj.get('hook')}/{inj.get('hook_version')}")
    return True


def sanitized_env() -> dict:
    """The environment a build subprocess may see: no credentials of ours."""
    allowed = (
        "HOME",
        "PATH",
        "USER",
        "TMPDIR",
        "LANG",
        "LC_ALL",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "ANDROID_HOME",
        "ANDROID_SDK_ROOT",
        "JAVA_HOME",
        "CARGO_BUILD_JOBS",
    )
    return {k: v for k, v in os.environ.items() if k in allowed}
