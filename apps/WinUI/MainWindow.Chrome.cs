using Microsoft.UI;
using Microsoft.UI.Composition.SystemBackdrops;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Media;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private void InitializeWindowChrome()
    {
        if (DesktopAcrylicController.IsSupported())
        {
            // The opaque editor background covers the window backdrop below the header.
            SystemBackdrop = new DesktopAcrylicBackdrop();
            HeaderFallback.Visibility = Visibility.Collapsed;
        }

        if (!AppWindowTitleBar.IsCustomizationSupported())
            return;

        MenuBarFallback.Child = null;
        MenuBarFallback.Visibility = Visibility.Collapsed;
        EditorTitleBar.Content = DocumentMenuBar;
        EditorTitleBar.Visibility = Visibility.Visible;

        ExtendsContentIntoTitleBar = true;
        SetTitleBar(EditorTitleBar);
        AppWindowTitleBar caption = AppWindow.TitleBar;
        caption.ButtonBackgroundColor = Colors.Transparent;
        caption.ButtonInactiveBackgroundColor = Colors.Transparent;
    }
}
