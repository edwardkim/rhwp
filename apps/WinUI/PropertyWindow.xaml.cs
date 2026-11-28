using System.ComponentModel;
using System.Runtime.InteropServices;
using Microsoft.UI;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Input;
using Windows.Graphics;

namespace Rhwp.WinUI;

public sealed partial class PropertyWindow : Window
{
    private const int WindowOwner = -8;
    private readonly Func<bool>? apply;
    private readonly TaskCompletionSource<bool> dismissed = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private bool accepted;

    private PropertyWindow(string title, FrameworkElement content, Func<bool>? apply)
    {
        this.apply = apply;
        InitializeComponent();
        Title = title;
        Fields.Content = content;
        ApplyButton.Visibility = apply is null ? Visibility.Collapsed : Visibility.Visible;
        CloseButton.Content = apply is null ? "닫기" : "취소";
        Closed += (_, _) => dismissed.TrySetResult(accepted);
    }

    internal static async Task<bool> ShowAsync(Window owner, string title, FrameworkElement content,
        Func<bool>? apply, int height)
    {
        var window = new PropertyWindow(title, content, apply);
        try
        {
            window.PlaceOver(owner, height);
            window.Activate();
            return await window.dismissed.Task;
        }
        catch
        {
            window.Close();
            throw;
        }
    }

    private void PlaceOver(Window owner, int height)
    {
        nint hwnd = Win32Interop.GetWindowFromWindowId(AppWindow.Id);
        nint parent = Win32Interop.GetWindowFromWindowId(owner.AppWindow.Id);
        if (SetWindowLongPtrW(hwnd, WindowOwner, parent) == 0)
        {
            int error = Marshal.GetLastPInvokeError();
            if (error != 0)
                throw new Win32Exception(error);
        }

        if (AppWindow.Presenter is OverlappedPresenter presenter)
        {
            presenter.IsMinimizable = false;
            presenter.IsMaximizable = false;
        }

        double scale = 1;
        if (owner.Content is FrameworkElement ownerRoot)
        {
            LayoutRoot.RequestedTheme = ownerRoot.RequestedTheme;
            scale = ownerRoot.XamlRoot?.RasterizationScale ?? 1;
        }

        RectInt32 work = DisplayArea.GetFromWindowId(owner.AppWindow.Id, DisplayAreaFallback.Primary).WorkArea;
        AppWindow.ResizeClient(new SizeInt32(Math.Min((int)Math.Round(470 * scale), work.Width),
            Math.Min((int)Math.Round(height * scale), Math.Max(1, work.Height - (int)Math.Round(48 * scale)))));
        SizeInt32 size = AppWindow.Size;
        PointInt32 position = owner.AppWindow.Position;
        SizeInt32 ownerSize = owner.AppWindow.Size;
        AppWindow.Move(new PointInt32(
            Math.Clamp(position.X + (ownerSize.Width - size.Width) / 2, work.X, Math.Max(work.X, work.X + work.Width - size.Width)),
            Math.Clamp(position.Y + (ownerSize.Height - size.Height) / 2, work.Y, Math.Max(work.Y, work.Y + work.Height - size.Height))));
    }

    private void OnApplyClicked(object sender, RoutedEventArgs args)
    {
        if (apply?.Invoke() == true)
        {
            accepted = true;
            Close();
        }
    }

    private void OnCloseClicked(object sender, RoutedEventArgs args) => Close();

    private void OnEscapeInvoked(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args)
    {
        args.Handled = true;
        Close();
    }

    [DllImport("user32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern nint SetWindowLongPtrW(nint window, int index, nint value);
}
