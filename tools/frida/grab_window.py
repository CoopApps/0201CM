"""Launch CM0102 and screenshot its window -> the exact framebuffer as a file.
No Frida hooks (no crash risk). Gives a ground-truth image to sample exact colors from."""
import subprocess, time, sys, ctypes
from ctypes import wintypes

OUT = "D:/cm0102-rs/reports/carve_segment_index/renders/game_grab.png"
WAIT = float(sys.argv[1]) if len(sys.argv) > 1 else 75

user32 = ctypes.windll.user32

def find_game_hwnd():
    res = []
    @ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
    def cb(h, l):
        if user32.IsWindowVisible(h):
            n = user32.GetWindowTextLengthW(h)
            buf = ctypes.create_unicode_buffer(n + 1)
            user32.GetWindowTextW(h, buf, n + 1)
            t = buf.value.lower()
            if "championship" in t or "cm 01" in t or "cm0102" in t:
                res.append(h)
        return True
    user32.EnumWindows(cb, 0)
    return res[0] if res else None

def main():
    subprocess.Popen(["D:/cm0102/cm0102.exe"], cwd="D:/cm0102")
    print(f"launched game, waiting {WAIT}s for it to load to the menu...")
    time.sleep(WAIT)
    from PIL import ImageGrab
    hwnd = find_game_hwnd()
    if hwnd:
        rect = wintypes.RECT()
        user32.GetWindowRect(hwnd, ctypes.byref(rect))
        box = (rect.left, rect.top, rect.right, rect.bottom)
        print("game window rect:", box)
        img = ImageGrab.grab(bbox=box)
    else:
        print("game window not found by title; grabbing full screen")
        img = ImageGrab.grab()
    img.save(OUT)
    print("saved", OUT, img.size)

if __name__ == "__main__":
    main()
