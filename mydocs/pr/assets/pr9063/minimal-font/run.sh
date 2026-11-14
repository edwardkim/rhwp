#!/bin/bash
set -euo pipefail
base=/home/edward/mygithub/rhwp
work=/home/edward/mygithub/rhwp-worktrees/pr-9063-restricted-object-next-page
bwrap --ro-bind /usr /usr --symlink usr/bin /bin --symlink usr/lib /lib --symlink usr/lib64 /lib64 \
 --ro-bind /home/edward/.nvm/versions/node/v24.15.0 /home/edward/.nvm/versions/node/v24.15.0 \
 --ro-bind /etc /etc --ro-bind /mnt/wsl/resolv.conf /mnt/wsl/resolv.conf --proc /proc --dev /dev --ro-bind /sys /sys --bind /tmp /tmp \
 --dir /home/edward --bind "$base" "$base" --bind "$work" "$work" \
 --bind /home/edward/.cargo /home/edward/.cargo --ro-bind /home/edward/.rustup /home/edward/.rustup \
 --tmpfs /usr/share/fonts --tmpfs /usr/local/share/fonts \
 --tmpfs "$base/ttfs" --ro-bind "$base/ttfs/opensource" "$base/ttfs/opensource" \
 --tmpfs "$work/ttfs" --ro-bind "$base/ttfs/opensource" "$work/ttfs/opensource" \
 --ro-bind "$base/ttfs/redistributable" "$base/ttfs/redistributable" \
 --ro-bind "$base/ttfs/redistributable" "$work/ttfs/redistributable" \
 --tmpfs "$base/output/pr-review/native-webfont-cache" \
 --setenv HOME /home/edward --setenv FONTCONFIG_FILE "$base/output/pr-review/pr9063-correction/minimal-font/fonts.conf" \
 --unsetenv RHWP_FONT_PATH --chdir "$work" "$@"
