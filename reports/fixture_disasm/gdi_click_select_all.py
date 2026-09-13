"""Click 'Select All' button to test whether any input reaches the game."""
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

def click(cx, cy, tag):
    sx = origin.x + cx
    sy = origin.y + cy
    user32.SetCursorPos(sx, sy); time.sleep(0.2)
    user32.mouse_event(0x02, 0, 0, 0, 0); time.sleep(0.05)
    user32.mouse_event(0x04, 0, 0, 0, 0); time.sleep(1.5)
    from PIL import ImageGrab
    r = wt.RECT()
    user32.GetWindowRect(found_hwnd, ctypes.byref(r))
    ImageGrab.grab(bbox=(r.left, r.top, r.right, r.bottom)).save(
        fr"D:\cm0102-rs\reports\fixture_disasm\runtime\sa_{tag}.png")

# Select All button appears at client (~595, ~181) based on screenshot
click(595, 181, "select_all")
# Try clicking Australia to change it
click(160, 236, "click_australia")
# Now try Next again
click(710, 596, "next_after_selectall")
