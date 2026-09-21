# Changelog

Release notes for Blenderbase. The section matching a version tag is
copied into the GitHub release by the release workflow, so keep the
`## X.Y.Z` headings exact.

## 1.3.2

### Added
- On the first start, Blender versions already installed on the computer are
  picked up: the Blender installer's folder in Program Files and Steam's copy
  on Windows, Applications on macOS, `/opt`, snap and Flatpak on Linux. Their
  folders appear under Settings › Locations, and **Find installed Blender
  versions** there runs the same sweep again. Installs elsewhere are still
  added with **Add location**.

### Changed
- In What to share, **Include addon files** sits under the list instead of
  at its end, so it stays in view however many Blender series there are.

### Fixed
- The first-download prompt asked for the installation folder a second time:
  the folder registered from the prompt was not stored as confirmed.

## 1.3.1

### Added
- **Settings › About**: who makes Blenderbase, with links to the website,
  the documentation (the wiki), the community on Discord and the source and
  issues on GitHub, plus what the app is built with.
- An info button next to the Settings and Sync titles opens the matching
  wiki page in your browser.
- **Back to Addons** at the top right of Settings, Sync and What to share,
  as in Install Blender; a click on the dimmed Blender list returns as well.

### Changed
- **What to share** is no longer a row of its own: it opens when you turn on
  sharing, send a transfer code, save to the sync folder or save a file, and
  its first button goes on with that way.
- Computers found on the local network sit in a box of their own under the
  rows, so they read as things found rather than as more actions.
- The Launch button steps back, dimmed, while Settings, Sync, What to share
  or a restore has the middle column.
- **macOS:** the app says why it needs access to the local network, so the
  permission prompt reads properly, and a refused local connection names
  the Local Network privacy setting instead of blaming a firewall.

## 1.3.0

### Added
- **Sync**: a new button in the title bar and a view that shares your Blender
  setup across computers: the installed Blender versions and, per series,
  preferences, theme, keymap and the addon list. Four ways, in order of
  preference: **Local network** (turn on sharing on one computer, type its
  six-digit PIN on the others; nothing leaves the network), **Transfer code**
  (the setup travels encrypted through a relay under a code like
  `brave-otter-4412`, kept up to 7 days and removed once received),
  **Sync folder** (a folder in Dropbox, OneDrive, iCloud or Google Drive;
  every computer is told when a newer setup was saved) and **Setup file**
  (one `.bbsetup` file to carry yourself).
- **What to share**: before sharing, choose which installed Blender versions
  go and, per series, which parts and which addons. Series left out are not
  read at all, so a small share is quick.
- **Restore view**: a received or opened setup shows one row per Blender
  series with switches for preferences, theme, keymap and addons. Each series
  is backed up before it is applied and one click undoes it. Blender versions
  the setup names and this computer lacks are listed there for download and
  install.
- Addons installed from a file can travel with the setup (**Include addon
  files**); extensions are installed again by name on the other computer.
- A `.bbsetup` file opened with Blenderbase (Open with) lands in the restore
  view, and the first-run screen offers **Restore from a setup file**.

### Changed
- The README says what leaves the computer: nothing, unless you send a
  transfer code, and then only the encrypted setup.

## 1.2.9

### Changed
- **macOS:** the app is signed with the Developer ID of Physical Software SIA
  and notarized by Apple. It opens like any downloaded app; clearing the
  quarantine flag with `xattr` is no longer needed.

## 1.2.8

### Fixed
- A Blender registered from a folder whose name carries no version (an app
  bundle in /Applications on macOS, an MSI install on Windows) showed an
  empty version. The number now comes from Blender itself, read together
  with the build details, and versions still missing it are read again once.

## 1.2.7

### Changed
- In Settings, **Add location** is the last row of the folder list instead
  of a block in the toolbar band.
- The Recent Files column and its toggle are gone while Settings or Install
  Blender is open; they return with the Addons view.
- The Windows installer and executable are code-signed by SIA Physical
  Software (Azure Artifact Signing). SmartScreen may still warn on first run
  until the publisher has earned reputation.
- New application icon.
- **Windows installer** dressed in Blenderbase artwork: a dark banner and
  welcome image with white wizard text, in place of the stock WiX pages.
- The console segment of the Launch button is grey while off and joins the
  blue launch part while on.
- The release list is fetched with a lighter touch: one small index request
  tells which Blender series changed since the last look, only those series
  are read again, the result is kept on disk between runs, and repeated
  refreshes within a minute do not contact the mirror at all. A "429 Too Many
  Requests" answer is retried with a pause instead of failing the list.

### Fixed
- **macOS:** Blender installed in a chosen location as an app bundle
  (`/Applications/Blender.app`, or a renamed copy such as `Blender 4.2.app`)
  is found and registered; only version folders were scanned before.
- The Discord icon and the Blenderbase logo link in the footer open the
  browser again.

## 1.2.6

### Added
- A terminal segment on the left of the Launch button. Click it to launch
  Blender with its console visible: Python output, script errors and
  warnings in a console window on Windows, Terminal on macOS, or your
  terminal emulator on Linux. The choice is remembered and the button then
  reads "with console" under the version. Hovering the segment explains it
  in the status line.

### Changed
- Settings now open in the middle column like Install Blender, with
  Locations, Launch, Updates and Appearance sections; the separate
  settings page is gone.
- The **Show / hide Recent Files** button is a block at the right end of the
  toolbar band instead of a floating icon in the header corner: in the
  middle column while the panel is hidden (a block as wide as the column,
  labelled "Recent Files"), and the whole toolbar band of the Recent Files
  column while it is shown. The addon columns Version, Type, Enabled and
  delete fit in the same 300px, so the Name column ends where Recent Files
  begins.
- The refresh icons in the Blender and Addons headers are gone. Installed
  versions are rescanned when the window regains focus (at most every 15
  seconds), and a version's addons are re-read when you come back after
  launching it from Blenderbase, which is when Preferences changes happen.
  **Rescan installed versions** and **Re-read addons from Blender** remain
  in the right-click menus of the two lists.
- Settings controls (buttons, number field, theme dropdown) are one width
  and fill the full row height.

### Fixed
- Recent files drifted off the row grid of the other columns once scrolled:
  the list started 72px above their rows, which is not a whole number of
  28px file rows. The column now has a list header like its neighbours and
  its list starts on the same line, so series and file rows stay aligned at
  every scroll position.
- A damaged database (SQLite "disk image is malformed") no longer leaves the
  app showing no versions with every action failing. At startup the file is
  checked; a damaged one is moved into a dated `corrupt-…` folder next to it
  and a fresh database is created. Installation locations then need to be
  re-added (the Install Blender prompt or Settings → Locations); installed
  versions are found again by rescanning.

## 1.2.5

### Changed
- The first download proposes a default folder for Blender versions,
  with the option to pick another: `C:\blenderbaseapps` on Windows,
  `~/Applications/Blenderbase` on macOS, `~/.local/share/blenderbase/apps`
  on Linux. The folder picker no longer opens unprompted.

### Fixed
- Deleting a Blender version inside an installation location registered
  before 1.2.1 was refused as "not inside a confirmed location". Every
  registered location now counts. A version whose folder is missing, or
  lies outside all locations, is offered to be removed from the list;
  files outside the locations are never touched.
- **macOS and Linux:** installing a Blender build failed after unpacking
  with "No such file or directory". The download path was built with a
  Windows separator and the version folder was guessed from a `.zip`
  name; the backend now reports the folder it actually created.

## 1.2.4

### Fixed
- **Open file location** on Windows now selects the item in Explorer; it
  opened the Documents folder instead.
- Blenderbase starts on databases written by builds whose migration files
  had different line endings, instead of failing with "migration ... was
  previously applied but has been modified".

### Changed
- Lists scroll smoothly and always come to rest on a row boundary, in
  every column, whether scrolled by wheel, trackpad, scrollbar or keys.

## 1.2.3

### Added
- Right-click an addon, a recent file or an installed Blender version and
  choose **Open file location** to show it in Explorer or Finder with the
  item selected.

### Fixed
- **macOS:** the window buttons sit lower, centred in the title bar.
- Recent files rows line up with the rows of the other two columns; the
  panel header was 18px shorter than its neighbours.

### Changed
- Release notes open with direct links to the three installers.
- Releases ship one installer per platform (MSI, dmg, AppImage) plus the
  signed update bundles and manifest the updater needs; the NSIS and deb
  packages are no longer published.

## 1.2.2

### Fixed
- **macOS:** the native close, minimise and zoom buttons are back in the
  top-left corner of the window.

### Changed
- Release notes now come from this changelog: the section matching the
  version tag is published with each release.

## 1.2.1

### Changed
- Pressing **Install Blender** with no install folder set now opens the
  folder picker directly. The chosen folder is registered, made the
  default and checked for write access, and the download starts right
  away. Cancelling the picker simply does nothing.
- Adding a location in Settings no longer reports an error when the
  picker is cancelled.

### Security
- Updated dependencies with published advisories on both sides of the
  app. The web side (react-router, PostCSS, nanoid, Immutable) and the
  Rust side (HTTP/2 and QUIC networking, XML parsing) now report zero
  known vulnerabilities.

## 1.2.0

### Added
- **New layout.** Installed Blender versions on the left, the addons of
  the selected version in the middle, recent files on the right, and the
  launch bar below. The Install panel lists stable, LTS, daily and patch
  builds with search.
- **Addon management.** List every addon and extension of a Blender
  version, enable or disable them, install from a `.zip` or `.py`, link
  a development folder in place (symlink), delete, and reveal in the
  file browser. Blender is driven headlessly, so nothing needs to be
  open.
- **Automatic updates.** Blenderbase checks GitHub Releases on launch
  and offers new versions; **Check for updates** in Settings does the
  same on demand.
- **macOS (Apple Silicon) and Linux builds** alongside Windows. macOS
  installs are mounted from the official dmg; Linux from the official
  tar.xz.
- Build date, commit hash and release cycle shown for every installed
  version.
- New application icon.

### Security
- Every downloaded Blender archive is verified against its published
  SHA-256 before it is unpacked; a mismatch discards the download.
- Archives can no longer write outside their target folder.
- Installation paths are never passed through a shell, and Blender
  versions are launched or deleted only from confirmed locations.
- A Content Security Policy is enforced in the app window.

### Fixed
- Launching Blender no longer freezes Blenderbase until Blender exits.
- Refreshing a version reads its download metadata from the right
  folder.
- Corrupt metadata files produce an error instead of a crash, and a
  failed start shows the reason instead of exiting silently.
- Success messages are only shown when the action actually succeeded.
- Version lists sort numerically (4.10 after 4.2).

### Changed
- Fetching lists no longer rescans the disk on every call; use the
  refresh buttons or the automatic scan at startup.
- Long operations (extracting, deleting, dialogs) no longer stall the
  rest of the app.

### Notes
- Installs older than 1.2.0 cannot update automatically to this
  version: 1.1.0 shipped without an updater, and 1.0.x builds were
  signed with a key that is no longer available. Install 1.2.0 by hand
  once; every version from here on updates itself.
- Installers are not yet code-signed. Windows shows a SmartScreen
  prompt; on macOS, clear the quarantine flag once with
  `xattr -dr com.apple.quarantine /Applications/Blenderbase.app`.
