"""Locate cm0102_GDI.exe window, restore if minimized, take a
screenshot, and report its client rectangle for click targeting."""
import ctypes, ctypes.wintypes as wt, sys, time
from ctypes import windll, wintypes

user32 = windll.user32
gdi32 = windll.gdi32

# Find the CM01/02 top-level window
EnumWindowsProc = ctypes.WINFUNCTYPE(wt.BOOL, wt.HWND, wt.LPARAM)
found_hwnd = None
def enum_cb(hwnd, _):
    global found_hwnd
    length = user32.GetWindowTextLengthW(hwnd)
    if length > 0:
        buf = ctypes.create_unicode_buffer(length + 1)
        user32.GetWindowTextW(hwnd, buf, length + 1)
        if "Championship Manager" in buf.value:
            found_hwnd = hwnd
            return False
    return True
user32.EnumWindows(EnumWindowsProc(enum_cb), 0)
if not found_hwnd:
    sys.exit("no Championship Manager window")
print(f"HWND: 0x{found_hwnd:x}")

# Check window state
placement = wt.RECT()
class WINDOWPLACEMENT(ctypes.Structure):
    _fields_ = [("length", wt.UINT), ("flags", wt.UINT),
                ("showCmd", wt.UINT), ("ptMinPosition", wt.POINT),
                ("ptMaxPosition", wt.POINT), ("rcNormalPosition", wt.RECT)]
wp = WINDOWPLACEMENT()
wp.length = ctypes.sizeof(WINDOWPLACEMENT)
user32.GetWindowPlacement(found_hwnd, ctypes.byref(wp))
print(f"  showCmd={wp.showCmd}  (1=NORMAL, 2=MIN, 3=MAX)")
print(f"  normal rect: {wp.rcNormalPosition.left},{wp.rcNormalPosition.top}"
      f" - {wp.rcNormalPosition.right},{wp.rcNormalPosition.bottom}")

# Restore if minimized
SW_RESTORE = 9
SW_SHOWNORMAL = 1
if wp.showCmd == 2:
    print("restoring minimized window")
    user32.ShowWindow(found_hwnd, SW_RESTORE)
    time.sleep(0.5)
user32.SetForegroundWindow(found_hwnd)
time.sleep(0.3)

# Get window + client rectangles
rect = wt.RECT()
user32.GetWindowRect(found_hwnd, ctypes.byref(rect))
print(f"  window rect: ({rect.left}, {rect.top}) - ({rect.right}, {rect.bottom})"
      f"  size={rect.right - rect.left}x{rect.bottom - rect.top}")

client = wt.RECT()
user32.GetClientRect(found_hwnd, ctypes.byref(client))
print(f"  client size: {client.right}x{client.bottom}")

# Client-to-screen conversion
class POINT(ctypes.Structure):
    _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]
pt = POINT(0, 0)
user32.ClientToScreen(found_hwnd, ctypes.byref(pt))
print(f"  client origin in screen coords: ({pt.x}, {pt.y})")

# Take screenshot via PIL if available
try:
    from PIL import ImageGrab
    box = (rect.left, rect.top, rect.right, rect.bottom)
    img = ImageGrab.grab(bbox=box)
    out = r"D:\cm0102-rs\reports\fixture_disasm\runtime\gdi_screenshot.png"
    img.save(out)
    print(f"  screenshot saved: {out} ({img.size[0]}x{img.size[1]})")
except Exception as e:
    print(f"  screenshot failed: {e}")
