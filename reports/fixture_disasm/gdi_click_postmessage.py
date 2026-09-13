"""Send WM_LBUTTONDOWN/UP directly to the CM window via PostMessage.
This bypasses the input queue - reaches the game's message loop
regardless of foreground state.
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

# Get client size
rect = wt.RECT()
user32.GetClientRect(found_hwnd, ctypes.byref(rect))
print(f"client: {rect.right}x{rect.bottom}")

# Enumerate all child windows to see if there's a sub-window
def enum_children(parent):
    children = []
    def cb(hwnd, _):
        length = user32.GetWindowTextLengthW(hwnd)
        buf = ctypes.create_unicode_buffer(length + 1) if length else None
        if buf: user32.GetWindowTextW(hwnd, buf, length + 1)
        cls = ctypes.create_unicode_buffer(256)
        user32.GetClassNameW(hwnd, cls, 256)
        r = wt.RECT()
        user32.GetWindowRect(hwnd, ctypes.byref(r))
        children.append((hwnd, cls.value, buf.value if buf else "",
                         (r.left, r.top, r.right, r.bottom)))
        return True
    user32.EnumChildWindows(parent, EnumWindowsProc(cb), 0)
    return children

children = enum_children(found_hwnd)
print(f"child count: {len(children)}")
for c in children[:10]:
    print(f"  {c}")

WM_MOUSEMOVE   = 0x0200
WM_LBUTTONDOWN = 0x0201
WM_LBUTTONUP   = 0x0202
WM_LBUTTONDBLCLK = 0x0203
MK_LBUTTON = 0x0001

def make_lparam(x, y):
    return (y << 16) | (x & 0xffff)

def postmsg_click(hwnd, cx, cy):
    lp = make_lparam(cx, cy)
    user32.PostMessageW(hwnd, WM_MOUSEMOVE, 0, lp)
    time.sleep(0.1)
    user32.PostMessageW(hwnd, WM_LBUTTONDOWN, MK_LBUTTON, lp)
    time.sleep(0.05)
    user32.PostMessageW(hwnd, WM_LBUTTONUP, 0, lp)
    time.sleep(0.5)

# Try PostMessage to top-level window
print(f"\nposting click to top-level HWND 0x{found_hwnd:x}")
postmsg_click(found_hwnd, 710, 596)
time.sleep(2.0)

# Screenshot
from PIL import ImageGrab
wr = wt.RECT()
user32.GetWindowRect(found_hwnd, ctypes.byref(wr))
img = ImageGrab.grab(bbox=(wr.left, wr.top, wr.right, wr.bottom))
img.save(r"D:\cm0102-rs\reports\fixture_disasm\runtime\after_postmsg.png")
print("screenshot saved")
