#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob
packages=(src-tauri/target/release/bundle/deb/*.deb)
if (( ${#packages[@]} != 1 )); then
  echo "Expected exactly one Debian package; found ${#packages[@]}." >&2
  exit 1
fi
package=${packages[0]}
[[ $(dpkg-deb -f "$package" Package) == pusthakam ]]
installed=false
cleanup() {
  if [[ $installed == true ]]; then sudo dpkg --remove pusthakam; fi
}
trap cleanup EXIT
installed=true
sudo dpkg --install "$package"
[[ -x /usr/bin/pusthakam ]]
assets=$(dpkg-query -L pusthakam)
grep -Eq '^/usr/share/applications/[^/]+\.desktop$' <<< "$assets"
grep -Eq '^/usr/share/icons/.+\.png$' <<< "$assets"
while IFS= read -r asset; do
  [[ -f "$asset" ]]
done < <(grep -E '^/usr/share/(applications/[^/]+\.desktop|icons/.+\.png)$' <<< "$assets")
xvfb-run -a python3 tools/release/linux-launch-smoke.py /usr/bin/pusthakam
sudo dpkg --remove pusthakam
installed=false
[[ ! -e /usr/bin/pusthakam ]]
echo "Debian installation, assets, process startup/restart and uninstall checks passed."
