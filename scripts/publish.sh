#!/usr/bin/env bash
# Publish the seven public stuk crates to crates.io in dependency order.
#
# Usage:
#   scripts/publish.sh                # publish all publishable crates
#   scripts/publish.sh --dry-run      # dry-run only, nothing uploaded
#   scripts/publish.sh --allow-dirty  # also pass through to cargo
#   scripts/publish.sh --package-only # like --dry-run, but skip the verify step
#                                     # (needed when upstream crates that we depend on
#                                     # haven't been published to crates.io yet)
#
# Required: a valid crates.io token (`cargo login <TOKEN>`).
set -euo pipefail

cd "$(dirname "$0")/.."

DRY_RUN=""
PACKAGE_ONLY=""
CARGO_EXTRA=()
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --package-only) PACKAGE_ONLY=1; CARGO_EXTRA+=(--no-verify) ;;
    *) CARGO_EXTRA+=("$arg") ;;
  esac
done

# Publishable crates in dependency order. Leaves first, dependents after.
CRATES=(
  stuk-style
  stuk-layout
  stuk-platform-shell
  stuk-actions
  stuk-accessibility
  stuk-render
  stuk-platform
)

if [[ -n "$DRY_RUN" || -n "$PACKAGE_ONLY" ]]; then
  echo "package-only run for ${#CRATES[@]} stuk crates:"
  printf '  - %s\n' "${CRATES[@]}"
  echo
  # Package all crates in a single `cargo package` invocation. This is
  # required: cargo only spins up the temporary overlay registry that lets
  # it resolve workspace-internal deps against the in-progress publish when
  # more than one package is being packaged in the same call (see the
  # `do_package` function in cargo's `cargo_package` source — the overlay
  # is gated on `deps.has_dependencies()`). Per-crate calls like
  # `cargo package -p stuk-actions` would try to resolve `stuk-layout`
  # against crates.io and fail because we haven't published it yet.
  package_args=()
  for crate in "${CRATES[@]}"; do
    package_args+=(-p "$crate")
  done
  cargo package "${package_args[@]}" "${CARGO_EXTRA[@]}"
  echo
  echo "all stuk crates packaged. run without --dry-run/--package-only to publish."
  exit 0
fi

echo "publishing ${#CRATES[@]} stuk crates in dependency order:"
printf '  - %s\n' "${CRATES[@]}"
echo

for crate in "${CRATES[@]}"; do
  echo "==> publishing $crate"
  cargo publish -p "$crate" "${CARGO_EXTRA[@]}"
  # crates.io needs a moment for the index to refresh before the next crate
  # can resolve its dep. Skip the wait on --dry-run.
  echo "waiting 30s for crates.io index to update..."
  sleep 30
done

echo
echo "all stuk crates published."
