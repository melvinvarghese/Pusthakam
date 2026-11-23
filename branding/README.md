# Pusthakam logo

Approved by the owner on 23 November 2026: the latest teal book-shaped P, used throughout Pusthakam.

`pusthakam-logo.png` is the exact approved transparent artwork, preserved as the raster master. SHA-256: `346294bd76501e598273de1ff60114b43133632eaa5c891befe89d46278e34ba`. No redraw, recoloring or vector substitute has been applied.

Run `./tools/generate-brand-icons.ps1` from PowerShell after installing project dependencies to regenerate desktop PNG/ICO/ICNS/Windows tile assets, the app mark, and website mark/favicons. Tauri uses contain sizing to pad this non-square master transparently, preserving its proportions and the complete silhouette. Only desktop outputs are copied into the repository.

The shared React BrandMark supplies the app navigation, sign-in/setup and startup splash. The tray uses the configured default window icon. LAN staff/OPAC pages embed the same app logo and favicon as bundled PNG data URIs, including disabled browser access; no external image service or new public file route is used. The local progress monitor reuses the app logo and favicon through explicit local routes. All seven website pages use the same mark and favicons through relative paths, including nested blog pages. Icon changes appear in newly built desktop executables/installers; an already running or previously packaged app will retain its embedded icon. Website changes remain local until published.

The approved master is a raster asset; it is not a vector file or a guarantee of exact uniform color. Preserve it as the reference for any future vector production work.

## Verification checkpoint

23 November 2026: 24 app tests, 4 monitor checks and 7 Rust LAN tests passed; formatting, lint, TypeScript, frontend and Windows native builds passed. Website links and image formats/transparency checked; actual splash reviewed on dark/light samples. Final executable SHA-256: `718d535642f07c01636f80c2a0da0ccb9ba193f8b413980432a9309d77b85991` (26575872 bytes). No installer or website publication in this checkpoint.

Latest splash presentation build: all 213 frontend tests and frontend/native builds passed. Current Windows executable SHA-256: `9348c89dba47fd9df62acc4273b18b055ff0cf6af08323e6a512f947e6b467e8` (26575872 bytes). The approved master is unchanged; only splash sizing/layout/motion changed.
