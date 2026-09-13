"""Inspect cm0102_GDI.exe UI control tree via pywinauto to determine
what UI automation is possible.
"""
from pywinauto import Application
from pywinauto.findwindows import find_windows

# Connect to running CM01/02 window.
windows = find_windows(title_re="Championship Manager.*")
print(f"found {len(windows)} matching windows: {windows}")
if not windows:
    raise SystemExit("no CM01/02 window found")

app = Application(backend="win32").connect(handle=windows[0])
main = app.window(handle=windows[0])
print(f"main window: {main}")
print(f"  is_visible: {main.is_visible()}")
print(f"  rect: {main.rectangle()}")
print(f"  child count: {len(main.children())}")

# Try to dump full control identifiers.
try:
    main.print_control_identifiers(depth=3)
except Exception as e:
    print(f"print_control_identifiers failed: {e}")

# Alternative: enumerate visible controls
print("\n=== Children ===")
for i, child in enumerate(main.children()):
    try:
        print(f"  [{i}] class={child.class_name()!r} text={child.window_text()!r} "
              f"rect={child.rectangle()}")
    except Exception as e:
        print(f"  [{i}] error: {e}")
