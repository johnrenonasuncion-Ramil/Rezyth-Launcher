#!/usr/bin/env python3
"""Verify a set of registry native mods and stage them for Starling's builder.

Each `--mod ID=PATH` bundle is checked against a pinned snapshot of the
horizon-rust/horizon-mods registry (SHA-256, revocation, the author's
signature, a safe unzip, the manifests) and, when this tree can serve the
hooks it declares, rebound to this tree's commit and staged under
`--output/bundles/<id>` in the `<bundle>/horizon-mixin.toml` shape
`packaging/android/native_build.sh` takes. What the bundle's calls reach
that Starling moved or changed is `native_mod_adapt.py`'s to adapt, as for
every native mod built here; a failure inside that build is reported per
mod, the whole set failing with it.

    python3 tools/native_mods/weave.py --registry DIR --mod id=file.hnmod ... \
        --output DIR [--raw-targets]

Prints the staged bundle directories (one a line, for a shell to pass to
native_build.sh) and writes `report.json` beside them.

Run ONLY in a disposable, isolated worker with no credentials and no
signing keys. This is not a sandbox; the later build executes mod code.
"""
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import compat


def selection(value: str) -> tuple[str, Path]:
    mod_id, sep, path = value.partition("=")
    if not sep or not compat.ID.fullmatch(mod_id) or not path:
        raise argparse.ArgumentTypeError("expected a registry id and bundle path as ID=PATH")
    return mod_id, Path(path)


def run(args: argparse.Namespace) -> dict:
    commit = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=compat.ROOT, text=True
    ).strip()
    dirty = subprocess.check_output(
        ["git", "status", "--porcelain", "--untracked-files=no"],
        cwd=compat.ROOT,
        text=True,
    ).strip()
    if dirty:
        raise compat.Rejected("the checkout is dirty; weave from a clean pinned source")
    if not compat.REV.fullmatch(args.registry_revision):
        raise compat.Rejected("the registry revision must be a full commit hash")
    report = {
        "schema": 1,
        "commit": commit,
        "registry_revision": args.registry_revision,
        "status": "not_built",
        "runtime_tested": False,
        "signed": False,
        "mods": [],
    }
    staged = []
    work = Path(tempfile.mkdtemp(prefix="starling-weave-"))
    try:
        for mod_id, bundle in sorted(args.mod):
            entry_report = {"id": mod_id}
            try:
                entry = compat.read_entry(args.registry, mod_id)
                extracted = work / mod_id
                data = compat.verified_extract(bundle, entry, extracted, args.registry)
                meta = data["meta"]
                entry_report.update(
                    version=meta["version"],
                    sha256=data["sha256"],
                    horizon_revision=meta["horizon"],
                    mixins_revision=meta["mixins"],
                    hooks=meta.get("hooks"),
                )
                rebind = compat.check_hooks(
                    data, compat.ROOT, commit, raw_targets=args.raw_targets
                )
                if rebind:
                    manifest_path = data["manifest_path"]
                    text = manifest_path.read_text(encoding="utf-8")
                    old = f'horizon_revision = "{meta["horizon"]}"'
                    if text.count(old) != 1:
                        raise compat.Rejected("cannot safely rebind the revision")
                    manifest_path.write_text(
                        text.replace(old, f'horizon_revision = "{commit}"'),
                        encoding="utf-8",
                    )
                entry_report["hook_revision_rebound"] = rebind
                bundle_dir = args.output / "bundles" / mod_id
                shutil.copytree(extracted / "mixins" / mod_id, bundle_dir)
                entry_report["status"] = "staged"
                staged.append(bundle_dir)
            except (compat.Rejected, OSError, ValueError) as error:
                entry_report["status"] = "rejected"
                entry_report["reason"] = str(error)
            report["mods"].append(entry_report)
    finally:
        shutil.rmtree(work, ignore_errors=True)
    report["status"] = "staged" if staged else "nothing_staged"
    (args.output / "report.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8"
    )
    print(json.dumps(report, indent=2), file=sys.stderr)
    for bundle in staged:
        print(bundle)
    return 0 if staged else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registry", type=Path, required=True)
    parser.add_argument("--registry-revision", required=True)
    parser.add_argument(
        "--mod",
        action="append",
        type=selection,
        required=True,
        help="registry ID=verified bundle path; repeat for a set",
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--raw-targets",
        action="store_true",
        help="also try bundles that pin an exact revision without a hook-API "
        "declaration, the mixin compiler judging their targets",
    )
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    try:
        return run(args)
    except (compat.Rejected, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"weave: {error}", file=sys.stderr)
        (args.output / "report.json").write_text(
            json.dumps({"schema": 1, "status": "rejected", "reason": str(error)}, indent=2)
            + "\n",
            encoding="utf-8",
        )
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
