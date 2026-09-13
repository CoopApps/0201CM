"""Try multiple approaches to advance past Select League(s):
  1. Click at multiple candidate Y coordinates for Next
  2. Send Enter/Space keys
  3. Right-click test
"""
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

def screenshot(tag):
    from PIL import ImageGrab
    r = wt.RECT()
    user32.GetWindowRect(found_hwnd, ctypes.byref(r))
    img = ImageGrab.grab(bbox=(r.left, r.top, r.right, r.bottom))
    p = fr"D:\cm0102-rs\reports\fixture_disasm\runtime\try_{tag}.png"
    img.save(p)
    print(f"  screenshot: {tag}")

WM_LBUTTONDOWN = 0x0201
WM_LBUTTONUP = 0x0202
WM_KEYDOWN = 0x0100
WM_KEYUP = 0x0101
MK_LBUTTON = 0x0001
VK_RETURN = 0x0d
VK_SPACE = 0x20
VK_TAB = 0x09

def click_client(cx, cy, tag):
    sx = origin.x + cx
    sy = origin.y + cy
    user32.SetCursorPos(sx, sy)
    time.sleep(0.15)
    user32.mouse_event(0x02, 0, 0, 0, 0)
    time.sleep(0.05)
    user32.mouse_event(0x04, 0, 0, 0, 0)
    time.sleep(1.0)
    screenshot(tag)

def key(vk, tag):
    user32.keybd_event(vk, 0, 0, 0)  # KEYDOWN
    time.sleep(0.05)
    user32.keybd_event(vk, 0, 2, 0)  # KEYUP
    time.sleep(1.0)
    screenshot(tag)

screenshot("start")

# 1. Try clicking at (710, 566) — higher up in case my y=596 was below Next
print("attempt: click client (710, 566)")
click_client(710, 566, "click_710_566")

# 2. Try (710, 610) — lower
print("attempt: click client (710, 610)")
click_client(710, 610, "click_710_610")

# 3. Try Enter key
print("attempt: Enter key")
key(VK_RETURN, "enter")

# 4. Try Space key
print("attempt: Space key")
key(VK_SPACE, "space")
