# Blenderbase 

[Download](https://github.com/PhysicalAddons/blenderbase-public/releases) | [Documentation](https://github.com/PhysicalAddons/blenderbase-public/wiki) | [Get Help](https://discord.com/invite/4pseCn9pys)

Blenderbase is a free utility app for Blender 3D (supporting versions `3.0.0.+`) that manages installed Blender versions, community and official addons and blend files. It was first developed as an internal tool for the `Physical Addons` team for addon development, but has been proven to be useful for general Blender use as well.

![image](https://github.com/PhysicalAddons/blenderbase-public/assets/60788469/c8ddb72a-3b2b-4260-aef7-3644fa3821d1)

It was inspired by similar apps, such as `Blender launcher` and owes its success to all forerunners, who, like Blenderbase, tried to improve the user experience for Blender. It was built using the `Tauri framework`, which uses the `Rust` programming language, making the app fast and reliable as well as lightweight. For specialized tasks inside of a Blender instance, `Python` and `BPY` are used.

Blenderbase improves upon apps like `Blender launcher`, by providing a faster downloadable Blender version webscraping, as well as limiting the footprint on Blender Foundations bandwidth, by only scraping versions starting from `3.1.0.+`, and ingoring older versions. It also allows the integration of already installed Blender versions (starting from `3.0.0.` versions), meaning you can manage already installed Blender versions through Blenderbase, whereas Blender launcher was built to only manage Blender versions downloaded through it. Blenderbase also allows accessing locally saved .blend files and launching them via any Blender version (as long as the Blender version and .blend file are compatible), as well as many other features.

The goal for Blenderbase is to function as an external manager for all things Blender. The name comes from the combination of words `Blender` and `database` - `Blenderbase`. The app functions as an interface to a locally generated database, that hosts data about locally installed Blender versions.

This database stores data, such as:
- Blender version metadata, allowing you to decern which specific Blender version you want to launch,
- Community and official addon metadata, allowing you to also easily install/delete, and enable/disable them outside of a launched `Blender` instance,
- Recently used .blend files file paths, allowing easy access to your project files, as well as a way to launch them in any Blender version of your choosing,
- Downloadable portable Blender version metadata, which is webscraped from publically accesible Blender Foundation online resources and mirror websites.

## Blenderbase can:
- Support `Windows 10/11` (x64),
- `Auto update` to new versions of the app, when they are published,
- Manage up to `3` modifiable Blender parent installation directories, that hold installed Blender version source file directories,
- Locally register installed Blender versions, their addons, recently used .blend files and the most recently published downloadable Blender versions,
- Download and install portable Blender versions from publicly accessible Blender Fouddtion online resources (`Stable` and `LTS` versions are currently taken from the European mirror website, `Daily` and `Patch` versions are taken from the main Blender Foundation website)
- Uninstall both portable and .msi installed Blender versions (for `Windows 10/11`),
- Launch any registered Blender version through Blenderbase, enabling/disabling its addons, which are registered in the Blenderbase database,
- Enable/disable addons outside of a Blender instance, saving the changes in the Blenderbase databse,
- Install any community addons via a `.zip` or a `.py`,
- Semantic link addon directories (meant for easier addon development), when Blenderbase is launched `as admin`,
- Uninstall any community addon or delete the semantic link, if there is one,
- Open recently saved .blend files in any selected Blender version (provided the specific Blender version works for the .blend file).

## Notice

`Blenderbase` is the property of `Physical Addons` and has no legal binding to `Blender Foundation`, unless further specified. Currently there exists no formal agreement between `Physical Addons` and `Blender Foundation` regarding the use of their online resources, but all `Blender Foundation` resources are used in good faith and with the utmost care, so as to not strain, slow down or misuse `Blender Foundation` online resources, those being the `official` and `mirror` download pages, hosting the `Stable`, `LTS`, `Daily` and `Patch` Blender version releases.

Since v1.0.28, Blenderbase scrapes downloadable portable Blender versions from:
- https://ftp.nluug.nl/pub/graphics/blender/release/
- https://builder.blender.org/download/daily/
- https://builder.blender.org/download/patch/

**Note:** **_The EU mirror was chosen to currently be the only LTS and Stable version source, because the apps developers are based in the EU region._**

Blenderbase is currently free (closed source) and is allowed to be used in any projects involving Blender use, addon development or Blender project management wheter for hobby, educational or commercial reasons. 
