using System.Diagnostics;
using Microsoft.UI;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.Web.WebView2.Core;
using Windows.UI.ViewManagement;

namespace Rhwp.WinUI;

public sealed partial class StudioWindow : Window
{
    private readonly TaskCompletionSource initialized = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private readonly AccessibilitySettings accessibility = new();
    private readonly HashSet<Window> printWindows = [];
    private StudioWebBridge? bridge;
    private bool started;
    private bool closing;
    private bool busy;

    public StudioWindow()
    {
        InitializeComponent();
        AppWindow.Closing += OnClosing;
        Closed += OnClosed;
        accessibility.HighContrastChanged += OnHighContrastChanged;
    }

    private async void OnStudioLoaded(object sender, RoutedEventArgs args)
    {
        if (started)
            return;
        started = true;

        try
        {
            string bundle = Path.Combine(AppContext.BaseDirectory, "Studio");
            if (!File.Exists(Path.Combine(bundle, "studio", "index.html")))
                throw new FileNotFoundException("앱에 포함된 Studio 빌드를 찾을 수 없습니다.");

            string localData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
            if (string.IsNullOrEmpty(localData))
                throw new DirectoryNotFoundException("사용자 데이터 폴더를 찾을 수 없습니다.");

            string profile = Path.Combine(localData, "rhwp", "Studio", "WebView2");
            ApplyWindowTheme();
            CoreWebView2Environment environment = await CoreWebView2Environment.CreateWithOptionsAsync(
                null, profile, new CoreWebView2EnvironmentOptions());
            await StudioView.EnsureCoreWebView2Async(environment);
            if (closing)
                return;

            CoreWebView2 core = StudioView.CoreWebView2;
            core.Settings.AreDefaultContextMenusEnabled = false;
            core.Settings.AreBrowserAcceleratorKeysEnabled = false;
            core.Settings.IsStatusBarEnabled = false;
            core.SetVirtualHostNameToFolderMapping(StudioWebBridge.HostName, bundle,
                CoreWebView2HostResourceAccessKind.DenyCors);
            core.NavigationStarting += OnNavigationStarting;
            core.NavigationCompleted += OnNavigationCompleted;
            core.NewWindowRequested += (_, args) => OnNewWindowRequested(args, environment, bundle);

            bridge = new StudioWebBridge(core, HandleCommandAsync);
            bridge.StateChanged += UpdateStudioState;
            await core.AddScriptToExecuteOnDocumentCreatedAsync(StudioWebBridge.InstallTransport);
            if (closing)
                return;
            core.Navigate(StudioWebBridge.StartUri);
            await bridge.Ready;
            initialized.TrySetResult();
            await UpdateThemeAsync();
        }
        catch (Exception error) when (!closing)
        {
            ShowError("편집기를 열 수 없습니다. " + error.Message);
            initialized.TrySetCanceled();
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
        }
        finally
        {
            HideLoading();
        }
    }

    private void OnStudioThemeChanged(FrameworkElement sender, object args) => _ = UpdateThemeAsync();

    private void OnHighContrastChanged(AccessibilitySettings sender, object args) =>
        DispatcherQueue.TryEnqueue(() => _ = UpdateThemeAsync());

    private async Task UpdateThemeAsync()
    {
        if (!started || closing)
            return;

        string theme = ApplyWindowTheme();
        if (!initialized.Task.IsCompletedSuccessfully || bridge is null)
            return;

        try
        {
            await bridge.RequestAsync("theme", new { theme });
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
        }
    }

    private string ApplyWindowTheme()
    {
        bool highContrast = accessibility.HighContrast;
        bool dark = StudioRoot.ActualTheme == ElementTheme.Dark;
        if (AppWindowTitleBar.IsCustomizationSupported())
        {
            AppWindow.TitleBar.PreferredTheme = highContrast ? TitleBarTheme.UseDefaultAppMode :
                dark ? TitleBarTheme.Dark : TitleBarTheme.Light;
        }

        StudioView.DefaultBackgroundColor = highContrast ? Colors.Transparent :
            dark ? ColorHelper.FromArgb(255, 32, 32, 32) : ColorHelper.FromArgb(255, 243, 243, 243);
        return highContrast ? "highContrast" : dark ? "dark" : "light";
    }

    private static void OnNavigationStarting(CoreWebView2 sender, CoreWebView2NavigationStartingEventArgs args)
    {
        if (!StudioWebBridge.IsStudioPage(args.Uri))
        {
            args.Cancel = true;
            if (args.IsUserInitiated)
                _ = OpenExternalLinkAsync(args.Uri);
        }
    }

    private void OnNavigationCompleted(CoreWebView2 sender, CoreWebView2NavigationCompletedEventArgs args)
    {
        if (args.IsSuccess)
            HideLoading();
        else if (args.WebErrorStatus != CoreWebView2WebErrorStatus.OperationCanceled)
            bridge?.FailInitialization(new InvalidOperationException("Studio 화면에 연결할 수 없습니다. " + args.WebErrorStatus));
    }

    private void HideLoading()
    {
        StudioProgress.IsActive = false;
        StudioLoading.Visibility = Visibility.Collapsed;
    }

    private async void OnNewWindowRequested(CoreWebView2NewWindowRequestedEventArgs args,
        CoreWebView2Environment environment, string bundle)
    {
        args.Handled = true;
        if (!args.IsUserInitiated || closing)
            return;

        if (!StudioWebBridge.IsPrintPage(args.Uri))
        {
            await OpenExternalLinkAsync(args.Uri);
            return;
        }

        // Studio renders directly into the same-origin window returned by window.open.
        var deferral = args.GetDeferral();
        var view = new WebView2();
        var window = new Window { Title = "RHWP 인쇄 미리보기", Content = view };
        printWindows.Add(window);
        window.Closed += (_, _) =>
        {
            printWindows.Remove(window);
            view.Close();
        };
        try
        {
            window.Activate();
            await view.EnsureCoreWebView2Async(environment);
            if (closing || !printWindows.Contains(window))
                return;

            CoreWebView2 print = view.CoreWebView2;
            print.SetVirtualHostNameToFolderMapping(StudioWebBridge.HostName, bundle,
                CoreWebView2HostResourceAccessKind.DenyCors);
            print.NavigationStarting += (_, navigation) => navigation.Cancel = !StudioWebBridge.IsPrintPage(navigation.Uri);
            print.NewWindowRequested += (_, request) => request.Handled = true;
            print.WindowCloseRequested += (_, _) => window.Close();
            args.NewWindow = print;
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
            if (printWindows.Contains(window))
                window.Close();
            if (!closing)
                ShowError("인쇄 미리보기를 열 수 없습니다. " + error.Message);
        }
        finally
        {
            deferral.Complete();
        }
    }

    private static async Task OpenExternalLinkAsync(string address)
    {
        if (!Uri.TryCreate(address, UriKind.Absolute, out Uri? uri) ||
            uri.Scheme is not ("https" or "http" or "mailto") || uri.Host == StudioWebBridge.HostName ||
            uri.UserInfo.Length != 0)
        {
            return;
        }

        try
        {
            await Windows.System.Launcher.LaunchUriAsync(uri);
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
        }
    }

    private void ShowError(string message)
    {
        StudioStatus.Message = message;
        StudioStatus.IsOpen = true;
    }

    private void OnClosed(object sender, WindowEventArgs args)
    {
        closing = true;
        initialized.TrySetCanceled();
        accessibility.HighContrastChanged -= OnHighContrastChanged;
        foreach (Window window in printWindows.ToArray())
            window.Close();
        bridge?.Dispose();
        StudioView.Close();
    }
}
