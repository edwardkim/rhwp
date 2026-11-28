using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.WindowsRuntime;
using Microsoft.UI;
using Microsoft.UI.Xaml.Media.Imaging;

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

    private async Task LoadTitleBarLogoAsync()
    {
        if (RhwpTitleLogo.Source is not null)
            return;

        try
        {
            using Stream? asset = typeof(MainWindow).Assembly.GetManifestResourceStream("Rhwp.WinUI.TitleLogo.png");
            if (asset is null)
                return;

            using var stream = asset.AsRandomAccessStream();
            var bitmap = new BitmapImage();
            await bitmap.SetSourceAsync(stream);
            RhwpTitleLogo.Source = bitmap;
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
        }
    }

    [DllImport("kernel32.dll", ExactSpelling = true)]
    private static extern nint GetModuleHandleW(nint moduleName);

    [DllImport("user32.dll", ExactSpelling = true)]
    private static extern nint LoadIconW(nint module, nint resourceId);
}
