"""SendInput-based click on cm0102_GDI Next button. SendInput sits
below the standard input queue and is closer to real hardware
events than mouse_event()."""
import ctypes, time
from ctypes import wintypes as wt, c_ulong, c_long, c_ushort, c_short

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
    _fields_ = [("x", c_long), ("y", c_long)]
origin = POINT(0, 0)
user32.ClientToScreen(found_hwnd, ctypes.byref(origin))

# SendInput structs
PUL = ctypes.POINTER(c_ulong)
class MouseInput(ctypes.Structure):
    _fields_ = [("dx", c_long), ("dy", c_long), ("mouseData", c_ulong),
                ("dwFlags", c_ulong), ("time", c_ulong), ("dwExtraInfo", PUL)]
class InputUnion(ctypes.Union):
    _fields_ = [("mi", MouseInput)]
class Input(ctypes.Structure):
    _anonymous_ = ("u",)
    _fields_ = [("type", c_ulong), ("u", InputUnion)]

INPUT_MOUSE = 0
MOUSEEVENTF_LEFTDOWN = 0x0002
MOUSEEVENTF_LEFTUP = 0x0004
MOUSEEVENTF_ABSOLUTE = 0x8000
MOUSEEVENTF_MOVE = 0x0001

# Screen dimensions for MOUSEEVENTF_ABSOLUTE (normalized 0..65535)
screen_w = user32.GetSystemMetrics(0)
screen_h = user32.GetSystemMetrics(1)
print(f"screen: {screen_w}x{screen_h}")

def click_at(sx, sy):
    # Absolute normalized coords
    nx = int(sx * 65535 / screen_w)
    ny = int(sy * 65535 / screen_h)
    # Move
    mi = MouseInput(dx=nx, dy=ny, mouseData=0,
                    dwFlags=MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                    time=0, dwExtraInfo=None)
    inp = Input(type=INPUT_MOUSE, u=InputUnion(mi=mi))
    user32.SendInput(1, ctypes.byref(inp), ctypes.sizeof(inp))
    time.sleep(0.2)
    # Left down
    mi = MouseInput(dx=nx, dy=ny, mouseData=0,
                    dwFlags=MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_ABSOLUTE,
                    time=0, dwExtraInfo=None)
    inp = Input(type=INPUT_MOUSE, u=InputUnion(mi=mi))
    user32.SendInput(1, ctypes.byref(inp), ctypes.sizeof(inp))
    time.sleep(0.08)
    # Left up
    mi = MouseInput(dx=nx, dy=ny, mouseData=0,
                    dwFlags=MOUSEEVENTF_LEFTUP | MOUSEEVENTF_ABSOLUTE,
                    time=0, dwExtraInfo=None)
    inp = Input(type=INPUT_MOUSE, u=InputUnion(mi=mi))
    user32.SendInput(1, ctypes.byref(inp), ctypes.sizeof(inp))

# Screenshot before
from PIL import ImageGrab
rect = wt.RECT()
user32.GetWindowRect(found_hwnd, ctypes.byref(rect))
img = ImageGrab.grab(bbox=(rect.left, rect.top, rect.right, rect.bottom))
img.save(r"D:\cm0102-rs\reports\fixture_disasm\runtime\before_si.png")

# Click Next at client (710, 596)
sx = origin.x + 710
sy = origin.y + 596
print(f"clicking Next at screen ({sx},{sy})")
click_at(sx, sy)
time.sleep(2.0)

# Screenshot after
img = ImageGrab.grab(bbox=(rect.left, rect.top, rect.right, rect.bottom))
img.save(r"D:\cm0102-rs\reports\fixture_disasm\runtime\after_si.png")
print("after screenshot saved")
