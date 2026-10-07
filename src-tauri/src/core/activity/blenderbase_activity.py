# blenderbase_activity.py - installed by Blenderbase when "Count time and events in Blender" is on.
# Appends one JSON line per event to LOG_DIR on this computer and nothing else.
# Delete this file (or turn the switch off in Blenderbase) to stop.
import atexit
import json
import math
import os
import time
import uuid
from datetime import datetime, timezone

import bpy
from bpy.app.handlers import persistent

SCRIPT_VERSION = 2
LOG_DIR = {{LOG_DIR}}  # Blenderbase fills this in when it writes the file
HEARTBEAT_SECONDS = 60
# How often the 3D views are looked at for panning, orbiting and zooming, which fire no event.
SAMPLE_SECONDS = 0.25

_s = {"path": "", "t0": 0.0, "active": False, "undo": 0, "redo": 0,
      "cube": None, "objects": -1, "meshes": -1, "suzannes": 0, "render_t0": 0.0,
      # Since the last heartbeat: Blender units panned, degrees orbited, zoom doublings,
      # units objects moved, degrees objects turned, units the scene camera moved, addons enabled.
      "pan": 0.0, "orbit": 0.0, "zoom": 0.0, "moved": 0.0, "turned": 0.0, "cam_moved": 0.0, "addons": 0,
      # The last known state of each 3D view and each moved object, to take the differences from.
      "views": {}, "transforms": {}, "addon_count": -1}


def _write(kind, **fields):
    line = {"t": kind,
            "at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "up": round(time.monotonic() - _s["t0"]),
            **fields}
    try:
        with open(_s["path"], "a", encoding="utf-8") as f:
            f.write(json.dumps(line, separators=(",", ":")) + "\n")
    except OSError:
        pass  # a full disk or a missing folder must never disturb Blender


def _is_default_cube(ob):
    if ob.type != 'MESH' or ob.name != "Cube" or ob.data is None:
        return False
    return (len(ob.data.vertices) == 8 and len(ob.data.polygons) == 6
            and all(abs(d - 2.0) < 1e-4 for d in ob.dimensions))


def _count_suzannes():
    return sum(1 for me in bpy.data.meshes if me.name.startswith("Suzanne"))


def _track_scene():
    fresh = not bpy.data.filepath
    _s["cube"] = next((ob for ob in bpy.data.objects if _is_default_cube(ob)), None) if fresh else None
    _s["objects"] = len(bpy.data.objects)
    _s["meshes"] = len(bpy.data.meshes)
    _s["suzannes"] = _count_suzannes()
    # Every object's place now, so its first move counts in full rather than setting the baseline.
    transforms = {}
    for ob in bpy.data.objects:
        try:
            matrix = ob.matrix_world
            transforms[ob.as_pointer()] = (matrix.to_translation(), matrix.to_quaternion())
        except Exception:
            continue
    _s["transforms"] = transforms


def _is_playing():
    try:
        return any(s.is_animation_playing for s in bpy.data.screens)
    except Exception:
        return False


def _track_transforms(scene, depsgraph):
    """Adds up how far the objects that just changed transform moved and turned, in world space.
    Playback and scrubbing move animated objects by themselves, so those steps are not counted."""
    if depsgraph is None:
        return
    try:
        updates = depsgraph.updates
    except Exception:
        return
    playing = _is_playing()
    camera_key = None
    try:
        if scene is not None and scene.camera is not None:
            camera_key = scene.camera.as_pointer()
    except Exception:
        pass
    for update in updates:
        try:
            if not update.is_updated_transform:
                continue
            ob = update.id
            if not isinstance(ob, bpy.types.Object):
                continue
            original = getattr(ob, "original", None) or ob
            key = original.as_pointer()
            matrix = ob.matrix_world
            location = matrix.to_translation()
            rotation = matrix.to_quaternion()
            previous = _s["transforms"].get(key)
            _s["transforms"][key] = (location, rotation)
            if previous is None or playing:
                continue
            distance = (location - previous[0]).length
            degrees = math.degrees(previous[1].rotation_difference(rotation).angle)
            if distance > 1e-6:
                _s["moved"] += distance
                if key == camera_key:
                    _s["cam_moved"] += distance
            if degrees > 1e-3:
                _s["turned"] += degrees
        except Exception:
            continue


def _sample_views():
    """Every 3D view, a few times a second: the view's location, rotation and distance give
    panning, orbiting and zooming, which Blender reports through no handler."""
    try:
        seen = set()
        for wm in bpy.data.window_managers:
            for window in wm.windows:
                screen = window.screen
                if screen is None:
                    continue
                for area in screen.areas:
                    if area.type != 'VIEW_3D':
                        continue
                    space = area.spaces.active
                    regions = [space.region_3d] + list(space.region_quadviews)
                    for index, r3d in enumerate(regions):
                        if r3d is None:
                            continue
                        key = (space.as_pointer(), index)
                        seen.add(key)
                        location = r3d.view_location.copy()
                        rotation = r3d.view_rotation.copy()
                        distance = r3d.view_distance
                        previous = _s["views"].get(key)
                        _s["views"][key] = (location, rotation, distance)
                        if previous is None:
                            continue
                        panned = (location - previous[0]).length
                        orbited = math.degrees(previous[1].rotation_difference(rotation).angle)
                        if panned > 1e-6:
                            _s["pan"] += panned
                        if orbited > 1e-3:
                            _s["orbit"] += orbited
                        if distance > 1e-9 and previous[2] > 1e-9:
                            zoomed = abs(math.log2(distance / previous[2]))
                            if zoomed > 1e-6:
                                _s["zoom"] += zoomed
        for key in list(_s["views"]):
            if key not in seen:
                del _s["views"][key]
        count = len(bpy.context.preferences.addons)
        if _s["addon_count"] >= 0 and count > _s["addon_count"]:
            _s["addons"] += count - _s["addon_count"]
        _s["addon_count"] = count
    except Exception:
        pass
    return SAMPLE_SECONDS


@persistent
def _on_load(*_args):
    _write("new_file" if not bpy.data.filepath else "file_opened")
    _track_scene()


@persistent
def _on_save(*_args):
    _s["active"] = True
    _write("file_saved")


@persistent
def _on_depsgraph(*args):
    _s["active"] = True
    cube = _s["cube"]
    if cube is not None:
        try:
            cube.name
        except ReferenceError:  # the object was deleted
            _s["cube"] = None
            _write("cube_deleted")
    # Objects and meshes both: a delete and an add in one step leave the object count as it was.
    n_objects, n_meshes = len(bpy.data.objects), len(bpy.data.meshes)
    if n_objects != _s["objects"] or n_meshes != _s["meshes"]:
        _s["objects"], _s["meshes"] = n_objects, n_meshes
        suzannes = _count_suzannes()
        if suzannes > _s["suzannes"]:
            _write("suzanne_added", count=suzannes - _s["suzannes"])
        _s["suzannes"] = suzannes
    _track_transforms(args[0] if args else None, args[1] if len(args) > 1 else None)


@persistent
def _on_undo(*_args):
    _s["undo"] += 1


@persistent
def _on_redo(*_args):
    _s["redo"] += 1


@persistent
def _on_render_init(*_args):
    _s["active"] = True
    _s["render_t0"] = time.monotonic()
    _write("render_started")


def _render_seconds():
    t0, _s["render_t0"] = _s["render_t0"], 0.0
    return round(time.monotonic() - t0) if t0 else 0


@persistent
def _on_render_complete(*_args):
    _write("render_finished", seconds=_render_seconds())


@persistent
def _on_render_cancel(*_args):
    _write("render_cancelled", seconds=_render_seconds())


def _heartbeat():
    _write("hb", active=_s["active"], undo=_s["undo"], redo=_s["redo"],
           pan=round(_s["pan"], 3), orbit=round(_s["orbit"], 2), zoom=round(_s["zoom"], 3),
           moved=round(_s["moved"], 3), turned=round(_s["turned"], 2), cam_moved=round(_s["cam_moved"], 3),
           addons=_s["addons"])
    _s["active"], _s["undo"], _s["redo"] = False, 0, 0
    _s["pan"] = _s["orbit"] = _s["zoom"] = _s["moved"] = _s["turned"] = _s["cam_moved"] = 0.0
    _s["addons"] = 0
    return HEARTBEAT_SECONDS


def _first_look():
    _track_scene()  # bpy.data is restricted during register(), so this waits half a second
    return None


def _on_exit():
    _heartbeat()
    _write("end")


def _hook(name, fn):
    handlers = getattr(bpy.app.handlers, name, None)
    if handlers is not None:  # older Blender versions lack some of these; they simply count less
        handlers.append(fn)


def register():
    if bpy.app.background:
        return  # headless runs (Blenderbase uses them to list addons) are not sessions
    os.makedirs(LOG_DIR, exist_ok=True)
    session = uuid.uuid4().hex
    _s["t0"] = time.monotonic()
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    _s["path"] = os.path.join(LOG_DIR, f"{stamp}_{session}.jsonl")
    build_hash = bpy.app.build_hash
    _write("start", session=session, pid=os.getpid(), version=bpy.app.version_string,
           version_tuple=list(bpy.app.version),
           build_hash=build_hash.decode() if isinstance(build_hash, bytes) else str(build_hash),
           binary=bpy.app.binary_path, script=SCRIPT_VERSION)
    _hook("load_post", _on_load)
    _hook("save_post", _on_save)
    _hook("depsgraph_update_post", _on_depsgraph)
    _hook("undo_post", _on_undo)
    _hook("redo_post", _on_redo)
    _hook("render_init", _on_render_init)
    _hook("render_complete", _on_render_complete)
    _hook("render_cancel", _on_render_cancel)
    bpy.app.timers.register(_heartbeat, first_interval=HEARTBEAT_SECONDS, persistent=True)
    bpy.app.timers.register(_sample_views, first_interval=SAMPLE_SECONDS, persistent=True)
    bpy.app.timers.register(_first_look, first_interval=0.5)
    atexit.register(_on_exit)


def unregister():
    pass  # startup scripts are not toggled; Blenderbase removes the file instead
