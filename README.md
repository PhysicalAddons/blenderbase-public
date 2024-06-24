# `Blenderbase` 

`Blenderbase` is a free utility app for `Blender 3D` (supporting versions `3.1.0.+`) that manages installed Blender versions, community and official add-ons, and blend files. It was first developed as an internal tool for the `Physical Addons` team for add-on development but has proven to be useful for general Blender use as well.

![image](https://github.com/PhysicalAddons/blenderbase-public/assets/60788469/c8ddb72a-3b2b-4260-aef7-3644fa3821d1)

It was inspired by similar apps, such as `Blender Launcher`. It was built using the `Tauri framework`, which uses the `Rust` programming language, making the app fast, reliable, and lightweight. For specialized tasks inside of a Blender instance, `Python` and `BPY` are used.

`Blenderbase` improves upon apps like `Blender Launcher` by providing faster downloadable Blender version web scraping and limiting the footprint on Blender Foundation's bandwidth by only scraping versions starting from `3.1.0.+` and ignoring older versions. It also allows the integration of already installed Blender versions (starting from `3.0.0.` - `3.1.0.` versions), meaning you can manage already installed Blender versions through `Blenderbase`, whereas `Blender Launcher` was built to only manage Blender versions downloaded through it. `Blenderbase` also allows accessing locally saved `.blend` files and launching them via any Blender version (as long as the Blender version and `.blend` file are compatible), as well as many other features.

The goal for `Blenderbase` is to function as an external manager for all things Blender. The name comes from the combination of words `Blender` and `database` - `Blenderbase`. The app functions as an interface to a locally generated database that hosts data about locally installed Blender versions.

This database stores data, such as:
- Blender version metadata, allowing you to discern which specific Blender version you want to launch,
- Community and official add-on metadata, allowing you to easily install/delete and enable/disable them outside of a launched `Blender` instance,
- Recently used `.blend` files file paths, allowing easy access to your project files, as well as a way to launch them in any Blender version of your choosing,
- Downloadable portable Blender version metadata, which is web scraped from publicly accessible Blender Foundation online resources and mirror websites.

## `Blenderbase` can:
- Support `Windows 10/11` (x64),
- `Auto-update` to new versions of the app when they are published,
- Manage up to `3` modifiable Blender parent installation directories that hold installed Blender version source file directories,
- Locally register installed Blender versions, their add-ons, recently used `.blend` files, and the most recently published downloadable Blender versions,
- Download and install portable Blender versions from publicly accessible Blender Foundation online resources (`Stable` and `LTS` versions are currently taken from the European mirror website, `Daily` and `Patch` versions are taken from the main Blender Foundation website),
- Uninstall both portable and `.msi` installed Blender versions (for `Windows 10/11`),
- Launch any registered Blender version through `Blenderbase`, enabling/disabling its add-ons, which are registered in the `Blenderbase` database,
- Enable/disable add-ons outside of a Blender instance, saving the changes in the `Blenderbase` database,
- Install any community add-ons via a `.zip` or a `.py` files,
- Semantic link add-on directories (meant for easier add-on development) when `Blenderbase` is launched `as admin`,
- Uninstall any community add-on or delete the semantic link if there is one,
- Open recently saved `.blend` files in any selected Blender version (provided the specific Blender version works for the `.blend` file).

## Notice

`Blenderbase` is the property of `Physical Addons` and has no legal binding to `Blender Foundation` unless further specified. Currently, there exists no formal agreement between `Physical Addons` and `Blender Foundation` regarding the use of their online resources, but all `Blender Foundation` resources are used in good faith and with the utmost care so as to not strain, slow down, or misuse `Blender Foundation` online resources, those being the `official` and `mirror` download pages, hosting the `Stable`, `LTS`, `Daily`, and `Patch` Blender version releases.

As of v1.0.27, `Blenderbase` scrapes downloadable portable Blender versions from:
- https://ftp.nluug.nl/pub/graphics/blender/release/
- https://builder.blender.org/download/daily/
- https://builder.blender.org/download/patch/

**Note:** **_There are plans to allow users to choose from any one of the publicly accessible mirror websites or their own mirror websites (link), to better suit their own geographical region. The EU mirror was chosen to currently be the only `LTS` and `Stable` version source because the app's developers are based in the EU region. There are also plans to create our own endpoints that would decrease the impact on `Blender Foundation` online resources even more._**

`Blenderbase` is currently free (closed source) and is allowed to be used in any projects involving Blender use, add-on development, or Blender project management whether for hobby, educational, or commercial reasons. If there ever were to be a paid `Blenderbase` version (Enterprise), it will exist alongside a free and open-source (Community) version.
