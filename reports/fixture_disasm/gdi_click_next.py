"""Click Next once on the current cm0102_GDI screen, with a long
delay. Take before/after screenshots to verify state advance."""
import ctypes, time
from ctypes import wintypes as wt

user32 = ctypes.windll.user32
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
SW_RESTORE = 9
user32.ShowWindow(found_hwnd, SW_RESTORE)
time.sleep(0.5)
user32.SetForegroundWindow(found_hwnd)
time.sleep(0.5)

class POINT(ctypes.Structure):
    _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]
origin = POINT(0, 0)
user32.ClientToScreen(found_hwnd, ctypes.byref(origin))
print(f"client origin: ({origin.x}, {origin.y})")

# Snap before
from PIL import ImageGrab
rect = wt.RECT()
user32.GetWindowRect(found_hwnd, ctypes.byref(rect))
img = ImageGrab.grab(bbox=(rect.left, rect.top, rect.right, rect.bottom))
img.save(r"D:\cm0102-rs\reports\fixture_disasm\runtime\before_next.png")
print("before screenshot saved")

# Click Next at client (710, 596)
sx = origin.x + 710
sy = origin.y + 596
print(f"clicking Next at screen ({sx},{sy})")
user32.SetCursorPos(sx, sy)
time.sleep(0.5)
user32.mouse_event(0x02, 0, 0, 0, 0)  # LEFTDOWN
time.sleep(0.1)
user32.mouse_event(0x04, 0, 0, 0, 0)  # LEFTUP
time.sleep(2.0)  # long delay for exe to advance

# Snap after
img = ImageGrab.grab(bbox=(rect.left, rect.top, rect.right, rect.bottom))
img.save(r"D:\cm0102-rs\reports\fixture_disasm\runtime\after_next.png")
print("after screenshot saved")
