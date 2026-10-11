# Arch Linux package update recipe

This folder contains the Zed source patch used by the HTML tag completion
feature. Do not apply it to the installed `/usr/lib/zed/zed-editor` binary.
The official `zed` package installs a prebuilt binary; pacman updates replace
that binary and cannot retain a local source change.

## Applying the patch to the official package recipe

1. Obtain the current official Arch `zed` package recipe from
   [Arch Linux packaging](https://gitlab.archlinux.org/archlinux/packaging/packages/zed).
2. Copy `html-tag-auto-close.patch` into the directory containing that recipe.
3. In `PKGBUILD`, add the patch to `source` and add its SHA-256 checksum to
   `sha256sums`. Keep the existing checksum for the upstream source archive.
4. In the recipe's existing `prepare()` function, after the upstream source
   has been unpacked, apply the patch:

   ```sh
   patch -d "$_archive" -p1 < "$srcdir/html-tag-auto-close.patch"
   ```

   The current Arch recipe uses `$_archive` for the extracted source folder.
   Check the current recipe before applying this line because packaging
   functions can change.
5. Increment `pkgrel` for that upstream `pkgver`, then build and inspect the
   package using Arch's normal `makepkg` workflow. Install it only after
   reviewing the resulting package and its file list.
6. Install the parent folder containing `extension.toml` as a Zed dev
   extension. This turns off HTML Language Server close-tag insertion while
   keeping regular HTML completions such as `h1` suggestions.

## `pacman -Syu` behavior

This recipe is not an automatic patch hook for the official package. To keep
the feature across upgrades, refresh the recipe from Arch packaging whenever
`zed` is updated, reapply this patch, rebuild the package, and publish it in
your own pacman repository. Configure that repository before the official
repository and ensure its package version is at least as new as the official
`zed` package. Then `pacman -Syu` can select your patched package.

This still requires you to rebuild and publish the patched package for each
upstream update before upgrading. There is no supported way for a local
extension or pacman hook to inject a code change into Arch's official prebuilt
Zed binary. The maintenance-free option is for the change to be merged into
Zed upstream and released by Arch.

This recipe is documentation only. It does not modify pacman configuration,
build packages, or install software.
