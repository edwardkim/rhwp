#!/usr/bin/env bash
set -euo pipefail
rhwp_repo=/home/edward/mygithub/rhwp
rhwp_font_env="$rhwp_repo/output/pr-review/pr8187/minimal-font"
rhwp_node=/home/edward/.nvm/versions/node/v24.15.0
mkdir -p "$rhwp_font_env/cache"
cat > "$rhwp_font_env/fonts.conf" <<EOF
<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
<fontconfig><dir>$rhwp_repo/ttfs/opensource</dir><cachedir>$rhwp_font_env/cache</cachedir></fontconfig>
EOF
exec bwrap --ro-bind /usr /usr --symlink usr/bin /bin --symlink usr/lib /lib --symlink usr/lib64 /lib64 \
  --ro-bind /etc /etc --proc /proc --dev /dev --ro-bind /sys /sys --bind /tmp /tmp \
  --dir /home/edward --bind "$rhwp_repo" "$rhwp_repo" \
  --bind /home/edward/.cargo /home/edward/.cargo --ro-bind /home/edward/.rustup /home/edward/.rustup \
  --ro-bind "$rhwp_node" "$rhwp_node" \
  --tmpfs /usr/share/fonts --tmpfs /usr/local/share/fonts \
  --tmpfs "$rhwp_repo/output/pr-review/pr8187/validated-subset-fonts" \
  --tmpfs "$rhwp_repo/output/pr-review/pr8187/verification-fonts" \
  --setenv HOME /home/edward --setenv FONTCONFIG_FILE "$rhwp_font_env/fonts.conf" \
  --unsetenv RHWP_FONT_PATH --chdir "$rhwp_repo" "$@"
