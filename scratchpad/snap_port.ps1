Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$sig = @'
using System;
using System.Runtime.InteropServices;
public class W1 {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int L, T, R, B; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
}
'@
Add-Type -TypeDefinition $sig -ReferencedAssemblies "System.Drawing"
$p = Get-Process app -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if ($null -eq $p) { Write-Output "no app window"; exit 1 }
$h = $p.MainWindowHandle
[void][W1]::SetForegroundWindow($h)
Start-Sleep -Milliseconds 400
$r = New-Object 'W1+RECT'
[void][W1]::GetWindowRect($h, [ref]$r)
$x = $r.L; $y = $r.T; $w = $r.R - $r.L; $ht = $r.B - $r.T
Write-Output "rect=$x,$y,$w,$ht"
$bmp = New-Object System.Drawing.Bitmap $w, $ht
$g = [System.Drawing.Graphics]::FromImage($bmp)
$sz = New-Object System.Drawing.Size $w, $ht
$g.CopyFromScreen($x, $y, 0, 0, $sz)
$out = "D:\cm0102-rs\scratchpad\prelaunch\port_transfers.png"
$bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
Write-Output "wrote $out"
