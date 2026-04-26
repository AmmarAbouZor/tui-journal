# Release Preparation Workflow

Use this checklist when preparing a release. The user should provide the release type: `major`, `minor`, or `patch`.

## Goals

- Prepare a release commit or PR only.
- Do not create or push a git tag.
- Do not publish a GitHub release.
- Report the tag name the user should publish after merging.

## Steps

1. Read the current version from the root package version in `Cargo.toml`.
2. Compute the next semantic version from the requested release type.
   - `major`: increment major, reset minor and patch to `0`.
   - `minor`: increment minor, reset patch to `0`.
   - `patch`: increment patch.
3. Inspect unreleased commits since the current version tag:

   ```bash
   git log --format='%h %s%n%b%n---' "v<current-version>..HEAD"
   ```

4. Extend `CHANGELOG.md` with a new top release section:

   ```md
   ## x.y.z - YYYY-MM-DD

   ### Added
   - ...

   ### Changed
   - ...

   ### Fixed
   - ...
   ```

   Keep the text concise. Group only meaningful user-facing or maintainer-facing changes. Skip routine dependency bumps, formatting-only commits, and purely internal noise unless they matter for the release.

5. Update version files:
   - `Cargo.toml` package version
   - `CITATION.cff` `version` and `date-released`

6. Run checks that validate the edited files and update `Cargo.lock` automatically:

   ```bash
   cargo check
   ```

   Do not edit `Cargo.lock` manually. Let `cargo check` update it after changing `Cargo.toml`.

7. Show the user a concise summary:
   - new version
   - changelog categories added
   - files changed
   - checks run
   - tag name to publish: `vx.y.z`

## Notes

- The release workflow should use the root package version from `Cargo.toml`.
- The expected tag name format is `vx.y.z`, for example `v0.17.0`.
- Keep `CHANGELOG.md` as the manually curated release history. Do not regenerate it from GitHub release notes.
- Do not reintroduce `CHANGELOG.ron` or Aeruginous-based changelog tooling.
