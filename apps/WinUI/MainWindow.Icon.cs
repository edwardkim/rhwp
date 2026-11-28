using System.Runtime.InteropServices;
using Microsoft.UI;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    // The C# compiler emits ApplicationIcon as this Win32 icon group.
    private const int ApplicationIconResourceId = 32512;

    private void InitializeAppIcon()
    {
        nint module = GetModuleHandleW(0);
        if (module == 0)
            return;

        // LoadIcon returns a shared handle backed by the running executable.
        nint icon = LoadIconW(module, ApplicationIconResourceId);
        if (icon != 0)
            AppWindow.SetIcon(Win32Interop.GetIconIdFromIcon(icon));
    }

    [DllImport("kernel32.dll", ExactSpelling = true)]
    private static extern nint GetModuleHandleW(nint moduleName);

    [DllImport("user32.dll", ExactSpelling = true)]
    private static extern nint LoadIconW(nint module, nint resourceId);
}
