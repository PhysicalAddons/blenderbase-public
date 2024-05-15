# Blenderbase

Blenderbase is a utility app for Blender 3D (supporting versions 3.1.0+) that manages installed Blender versions, community and official addons and blend files.

It was inspired by similar apps, such as `Blender launcher` and owes its success to all forerunners, who, like Blenderbase, tried to improve the user experience for Blender.

Blenderbase is the property of `Physical Addons` and has no legal binding to `Blender Foundation`, unless further specified. All Blender Foundation resources are used in good faith and with the utmost care, so as to not slow down, damage or misuse `Blender Foundation` online resources, those being the official and mirror download pages, hosting the `Stable`, `LTS`, `Daily` and `Patched` Blender version releases. 

Currently there exists no formal agreement between `Physical Addons` and `Blender Foundation` regarding the use of their online resources.

## As of `v1.0.21`, Blenderbase can:
- Run on Windows 10/11 (x64),
- Manage up to `3` modifiable Blender parent installation directories,
- Externally register installed Blender versions, their addons, recent blend files and up to date downloadable Blender versions,
- Download and install portable Blender versions from publicly accessible Blender foundation resources (Stable and LTS version are taken from the European mirror website),
- Uninstall both portable and installer installed (.msi) Blender versions,
- Launch selected Blender versions, 
- Enable and disable addons when a Blender instance is launched through Blenderbase,
- Enable/disable addons outside of a Blender instance,
- Install community addons via `.zip` or `.py`,
- Semantic link addon directories (meant for addon development),
- Uninstall (or delete the semantic link of) community addons,
- Open recently opened blend files in any selected Blender version.

Blenderbase is currently free (closed source) and allowed to be used in any projects involving Blender use, addon development or Blender project management, weather for hobby, educational or commercial reasons.

```
⠀⠀⠀⠀⠀⠀⠀⠀⣀⣠⣤⠴⣶⣶⣶⣶⣶⣦⣤⣄⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⣀⣴⣶⠛⠉⠀⢠⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⣄⡀⠀⠀⠀⠀⠀⠀
⠀⢀⣴⣾⣿⣿⣿⠀⠀⠀⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠻⣦⠀⠀⠀⠀⠀
⢠⣾⣿⣿⣿⣿⣿⠀⠀⠀⢹⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠀⠈⢷⡀⠀⠀⠀
⣼⣿⣿⣿⣿⣿⡇⠀⠀⠀⠈⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠁⠀⠀⠈⢿⡀⠀⠀
⢻⣿⣿⣿⡿⠋⢀⣠⡤⠤⠤⣤⣙⡻⠿⣿⣿⣿⣿⡿⠟⠋⠀⠀⠀⠀⠀⠘⣧⠀⠀
⠀⢿⡛⠉⠀⢠⣿⣿⣶⡄⠀⠀⠈⠙⠶⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣤⣤⣿⠀⠀
⠀⠈⠻⣦⡴⠟⠙⠛⠋⠁⠀⠀⣤⣶⣶⣾⣳⣄⠀⠀⠀⠀⠀⣠⣾⣿⣿⣿⣿⠀⠀
⠀⠀⠀⣿⢱⣶⣦⣄⣀⡀⠀⠀⠙⠿⠿⡿⠁⠙⣦⠀⠀⠀⣰⣿⣿⣿⣿⣿⣿⠀⠀The source code is in another repo!
⠀⠀⢀⠙⢿⣿⣿⣿⣿⣿⣷⣦⠀⠀⠀⠀⠀⠀⢻⡆⠀⠀⣿⣿⣻⣿⣿⣿⣿⠀⠀
⠀⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⠀⠀⣠⣾⠃⠀⠀⣿⣿⣿⣿⣿⣿⠇⠀⠀
⠀⠀⣴⣾⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⢀⣀⣾⣿⣁⠀⠀⠀⢿⣿⣿⣿⡿⠋⠀⠀⠀
⠀⠀⠛⠻⣿⣿⣿⣿⣿⣿⣿⣿⢁⣴⣿⠛⢿⡄⠉⠛⠛⠒⠚⠛⠛⠉⠀⠀⠀⠀⠀
⠀⠀⠀⢀⣿⣿⣿⣿⣿⣿⣻⣿⣿⣿⣿⣦⡀⠻⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⢸⣿⣧⣤⣭⣉⠉⣿⣿⡟⣿⣿⣿⣿⣦⠹⣆⠀⠀⠀⠀⠀
⠀⠀⠀⢸⣿⠃⠀⠀⠈⠙⣿⣿⣷⣿⣿⣿⣿⣽⣷⢽⣆⠀
⠀⠀⠀⢿⣿⣧⡀⠀⠀⠀⠹⠿⠭⠭⢭⣿⡿⢷⡿⠉⣿⢼⠀⠀⠀
⠀⠀⠀⠀⠉⠉⠳⢦⣄⣀⣀⣀⣀⣤⣾⣏⠀⠸⣷⣼⠏⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢿⡍⠉⢻⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠻⣦⣽⣿⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
```
