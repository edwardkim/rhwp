using Microsoft.UI;
using Microsoft.UI.Composition;
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
            SystemBackdrop = new HeaderAcrylicBackdrop();
            HeaderFallback.Visibility = Visibility.Collapsed;
        }

        if (!AppWindowTitleBar.IsCustomizationSupported())
            return;

        MenuBarFallback.Child = null;
        MenuBarFallback.Visibility = Visibility.Collapsed;
        EditorTitleBar.Content = TitleBarLeadingContent;
        EditorTitleBar.Visibility = Visibility.Visible;

        ExtendsContentIntoTitleBar = true;
        SetTitleBar(EditorTitleBar);
        AppWindowTitleBar caption = AppWindow.TitleBar;
        caption.ButtonBackgroundColor = Colors.Transparent;
        caption.ButtonInactiveBackgroundColor = Colors.Transparent;
    }

    private sealed class HeaderAcrylicBackdrop : SystemBackdrop
    {
        private DesktopAcrylicController? controller;

        protected override void OnTargetConnected(ICompositionSupportsSystemBackdrop target, XamlRoot root)
        {
            base.OnTargetConnected(target, root);
            controller = new DesktopAcrylicController { Kind = DesktopAcrylicKind.Thin };
            controller.SetSystemBackdropConfiguration(GetDefaultSystemBackdropConfiguration(target, root));
            controller.AddSystemBackdropTarget(target);
        }

        protected override void OnTargetDisconnected(ICompositionSupportsSystemBackdrop target)
        {
            base.OnTargetDisconnected(target);
            controller?.RemoveSystemBackdropTarget(target);
            controller?.Dispose();
            controller = null;
        }
    }
}
