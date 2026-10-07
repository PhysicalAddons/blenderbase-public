# Stats

Blenderbase counts the time you spend in each installed Blender version and a few things you do there, and turns the numbers into achievements. Everything stays on your computer. The feature is on from the start; the switch **Time tracking and achievements** under Settings › Launch turns it off and on.

*(Draft for the wiki page `11.-Stats`, which the info button next to the Stats title opens. Screenshots to add once the view is final.)*

## How it works

While the switch is on, Blenderbase keeps one small Python file, `blenderbase_activity.py`, in the startup folder of every Blender series it knows (2.80 and later):

- Windows: `%APPDATA%\Blender Foundation\Blender\<series>\scripts\startup\`
- macOS: `~/Library/Application Support/Blender/<series>/scripts/startup/`
- Linux: `~/.config/blender/<series>/scripts/startup/`

Blender runs that file at every launch, whether Blender was started from Blenderbase, a desktop shortcut or a double-clicked .blend. It does not appear in Blender's add-on list; it is not an add-on. Blenderbase refreshes the file when a new Blender version is installed or when Blenderbase itself is updated.

## What is counted

While Blender runs, the script writes a log line once a minute and whenever one of these happens:

- Blender starts, a file is opened or a new file started, a file is saved
- A render starts, finishes or is cancelled, and how long it took
- Undo and redo
- The default cube is deleted (the untouched cube of a fresh file; a renamed or scaled one does not count)
- A Suzanne is added
- How far the 3D views were panned, orbited and zoomed (sampled a few times a second, since Blender fires no event for navigation)
- How far objects and the scene camera were moved and how far objects were turned, from the scene updates; animation playback is left out
- Addons enabled inside Blender

Nothing else: no file names, no paths, no object names other than those two, no keystrokes, no screenshots. The log lives in the Blenderbase data folder and is read by Blenderbase when it starts and whenever its window comes back into focus, so sessions are counted even when Blenderbase was closed the whole time. After a session is read, its log file is deleted; the numbers stay in Blenderbase's own database.

Nothing is sent anywhere. The sync bundle (`.bbsetup`) does not carry stats either.

## Where the numbers show

- **Blender column**: the hours spent in each version sit at the right of its row, and the total in the column subtitle. Whole hours; "<1 h" for anything shorter.
- **Stats** (the trophy in the title bar), **Overview** tab: hours today and this week, then over the chosen range the total, active time (minutes in which something changed), the longest session and the default cubes deleted, and the time per version with a bar for each. The block at the right of the band picks the range: today, this week, the last 30 days or all time. Sessions of versions you have since uninstalled keep their hours under one row, "Versions no longer installed".
- **Stats**, **Achievements** tab: every badge with its progress, or the day it was unlocked.

## Achievements

Thirty-three badges, read off the counted numbers and off what Blenderbase itself does:

| Badge | Rule |
| --- | --- |
| First Steps | Finish one session |
| Cube Curious, Cube Slayer, No Cube Left Behind | Delete 1, 100, 1,000 default cubes |
| Monkey Business | Add 50 Suzannes |
| Centurion, Thousand Hours | 100 and 1,000 hours in Blender |
| Loyal | 500 hours in one series |
| Focused | 50 hours of active time |
| Marathon | One session longer than 6 hours |
| Night Shift | A session still running at 2 AM |
| Version Hopper | Launch 5 different versions in one week |
| Early Adopter | A session in an alpha build |
| Render Farm, Patient | 100 renders finished; 10 hours of rendering |
| Save Habit | Save a file 1,000 times |
| Undo Champion | 10,000 undos |
| Crash Survivor | Come back after Blender closed unexpectedly |
| Dizzy | Orbit the viewport 1,000 full turns |
| Globetrotter | Pan the viewport 100 km (Blender units; panning while zoomed out covers ground fast) |
| Microscope | Zoom in and out 1,000 doublings |
| Director | Move the camera 1 km in total |
| Mover, Spinner | Move objects 10 km; rotate objects 1,000 full turns |
| Plugged In | Install an addon through Blenderbase |
| Tinkerer | Install or enable 25 addons |
| Collector, Extensionist | 25 different addons, 10 different extensions installed at once |
| Well Connected, In Sync | Share your setup with another computer; apply one from another computer |
| First Download, Hoarder | Install a Blender version through Blenderbase; 10 versions installed at once |
| Full Spectrum | LTS, Stable, Beta and Alpha installed at the same time |

A new unlock is announced in the status line at the bottom of the window, and the trophy button carries a dot until you open the Stats view.

## Turning it off

Turn **Time tracking and achievements** off under Settings › Launch. Blenderbase removes the file from every series folder and hides the hours, the trophy and the Stats view. A Blender already running keeps counting until it closes, and that last session is still read. Your hours and badges stay; nothing is deleted, and everything comes back when the switch goes on again.

## Good to know

- Two Blender windows at once are two sessions, so the total can exceed the clock.
- A session with no clean end (Blender crashed, or was killed) is still counted up to its last minute, and is what Crash Survivor looks for.
- "Load Factory Settings" and `--factory-startup` skip user scripts, so such a session is not counted.
- Portable Blender installs (a `portable` folder next to the executable) read their startup scripts from there; Blenderbase handles that folder when it knows the install.
- Blender series older than 2.80 are left alone.
