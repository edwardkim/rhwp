using System.Diagnostics;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media.Imaging;
using Microsoft.Windows.Storage.Pickers;
using Windows.Graphics.Imaging;
using Windows.Security.Cryptography;

namespace Rhwp.WinUI;

public sealed partial class MainWindow : Window
{
    private const string IdleStatus = "HWP 또는 HWPX 파일을 여세요.";
    private const double MinZoom = 0.5;
    private const double MaxZoom = 4;
    private readonly NativeWorkerDispatcher worker = new();
    private readonly SemaphoreSlim documentActions = new(1);
    private readonly DispatcherQueueTimer renderTimer;
    private CancellationTokenSource? documentRequest;
    private CancellationTokenSource? renderRequest;
    private PageInfo? page;
    private string? fileName;
    private ulong viewVersion;
    private double zoom = 1;
    private bool picking;
    private bool closing;

    public MainWindow()
    {
        InitializeComponent();
        renderTimer = DispatcherQueue.CreateTimer();
        renderTimer.Interval = TimeSpan.FromMilliseconds(32);
        renderTimer.IsRepeating = false;
        renderTimer.Tick += OnRenderTick;
        AppWindow.Closing += OnClosing;
    }

    private async void OnOpenClicked(object sender, RoutedEventArgs args)
    {
        if (picking || closing)
            return;

        picking = true;
        RefreshButtons();
        string? path = null;
        try
        {
            var picker = new FileOpenPicker(AppWindow.Id) { FileTypeFilter = { ".hwp", ".hwpx" } };
            path = (await picker.PickSingleFileAsync())?.Path;
        }
        catch (Exception error)
        {
            if (!closing)
                StatusText.Text = "파일 선택기를 열 수 없습니다. " + error.Message;
        }
        finally
        {
            picking = false;
            if (!closing)
                RefreshButtons();
        }

        if (path is not null && !closing)
            await OpenFileAsync(path);
    }

    internal Task OpenFileAsync(string path) =>
        ChangeDocumentAsync(token => worker.OpenAsync(path, token),
            Path.GetFileName(path), resetZoom: true);

    private async void OnCloseClicked(object sender, RoutedEventArgs args)
    {
        if (!closing)
        {
            await ChangeDocumentAsync(async token =>
            {
                await worker.CloseDocumentAsync(token);
                return null;
            }, null, resetZoom: true);
        }
    }

    private async void OnPreviousClicked(object sender, RoutedEventArgs args)
    {
        if (page is { Page: > 0 } current && !closing)
            await GoToPageAsync(current, current.Page - 1);
    }

    private async void OnNextClicked(object sender, RoutedEventArgs args)
    {
        if (page is PageInfo current && current.Page + 1 < current.PageCount && !closing)
            await GoToPageAsync(current, current.Page + 1);
    }

    private Task GoToPageAsync(PageInfo current, uint index) =>
        ChangeDocumentAsync(token => worker.GetPageAsync(current, index, token), fileName, resetZoom: false);

    private async Task ChangeDocumentAsync(Func<CancellationToken, Task<PageInfo?>> action,
        string? name, bool resetZoom)
    {
        documentRequest?.Cancel();
        using var request = new CancellationTokenSource();
        documentRequest = request;
        ClearPage();
        if (resetZoom)
            zoom = 1;
        fileName = name;
        Title = name is null ? "RHWP" : name + " — RHWP";
        StatusText.Text = name is null ? "문서 닫는 중…" : "문서 읽는 중…";
        RefreshButtons();
        try
        {
            // Retain only the newest waiting file action. The native owner
            // drains any older action already accepted by the worker.
            await documentActions.WaitAsync(request.Token);
            try
            {
                PageInfo? result = await action(request.Token);
                if (closing || request.IsCancellationRequested)
                    return;
                page = result;
                if (result is null)
                    StatusText.Text = IdleStatus;
                else
                {
                    SetPageSize(result);
                    PageScroll.ChangeView(0, 0, null, true);
                    StatusText.Text = "쪽 표시 중…";
                    QueueRender();
                }
            }
            finally
            {
                documentActions.Release();
            }
        }
        catch (OperationCanceledException) when (closing || request.IsCancellationRequested)
        {
        }
        catch (Exception error)
        {
            if (!closing && !request.IsCancellationRequested)
                StatusText.Text = "문서 요청을 처리할 수 없습니다. " + error.Message;
        }
        finally
        {
            if (!closing && ReferenceEquals(documentRequest, request))
            {
                documentRequest = null;
                RefreshButtons();
            }
        }
    }

    private void ClearPage()
    {
        page = null;
        renderRequest?.Cancel();
        renderTimer.Stop();
        PageImage.Source = null;
        PageCanvas.Width = 0;
        PageCanvas.Height = 0;
    }

    private void SetPageSize(PageInfo current)
    {
        PageCanvas.Width = current.Width * zoom;
        PageCanvas.Height = current.Height * zoom;
    }

    private void OnZoomOutClicked(object sender, RoutedEventArgs args) => ZoomTo(zoom / 1.25);
    private void OnZoomInClicked(object sender, RoutedEventArgs args) => ZoomTo(zoom * 1.25);

    private void ZoomTo(double value)
    {
        if (closing || page is not PageInfo current)
            return;

        double nextZoom = Math.Clamp(value, MinZoom, MaxZoom);
        double ratio = nextZoom / zoom;
        zoom = nextZoom;
        PageImage.Source = null;
        SetPageSize(current);
        PageScroll.ChangeView(PageScroll.HorizontalOffset * ratio, PageScroll.VerticalOffset * ratio,
            null, true);
        RefreshButtons();
        QueueRender();
    }

    private void OnLoaded(object sender, RoutedEventArgs args)
    {
        if (Root.XamlRoot is XamlRoot root)
            root.Changed += OnXamlRootChanged;
        QueueRender();
    }

    private void OnUnloaded(object sender, RoutedEventArgs args)
    {
        if (Root.XamlRoot is XamlRoot root)
            root.Changed -= OnXamlRootChanged;
    }

    private void OnXamlRootChanged(XamlRoot sender, XamlRootChangedEventArgs args) => QueueRender();
    private void OnViewportSizeChanged(object sender, SizeChangedEventArgs args) => QueueRender();
    private void OnViewChanged(object sender, ScrollViewerViewChangedEventArgs args) => QueueRender();

    private void QueueRender()
    {
        if (closing || page is null)
            return;

        viewVersion++;
        renderRequest?.Cancel();
        if (renderRequest is null)
        {
            renderTimer.Stop();
            renderTimer.Start();
        }
    }

    private async void OnRenderTick(DispatcherQueueTimer sender, object args)
    {
        if (closing || renderRequest is not null || page is not PageInfo current)
            return;

        double deviceScale = Root.XamlRoot?.RasterizationScale ?? 1;
        PageViewport? viewport = PageViewport.Visible(current, PageScroll.HorizontalOffset,
            PageScroll.VerticalOffset, PageScroll.ViewportWidth, PageScroll.ViewportHeight, zoom, deviceScale);
        if (viewport is null)
            return;

        ulong version = viewVersion;
        using var request = new CancellationTokenSource();
        renderRequest = request;
        try
        {
            PageFrame frame = await worker.RenderAsync(current, viewport, request.Token);
            if (closing || request.IsCancellationRequested)
                return;

            var pixels = CryptographicBuffer.CreateFromByteArray(frame.Bgra);
            using var bitmap = SoftwareBitmap.CreateCopyFromBuffer(pixels, BitmapPixelFormat.Bgra8,
                frame.PixelWidth, frame.PixelHeight, BitmapAlphaMode.Premultiplied);
            var source = new SoftwareBitmapSource();
            await source.SetBitmapAsync(bitmap);
            if (closing || request.IsCancellationRequested)
                return;

            Canvas.SetLeft(PageImage, frame.OriginX / frame.DeviceScale);
            Canvas.SetTop(PageImage, frame.OriginY / frame.DeviceScale);
            PageImage.Width = frame.PixelWidth / frame.DeviceScale;
            PageImage.Height = frame.PixelHeight / frame.DeviceScale;
            PageImage.Source = source;
            StatusText.Text = $"{current.Page + 1} / {current.PageCount}쪽 · {(int)Math.Round(zoom * 100)}%";
        }
        catch (OperationCanceledException) when (closing || request.IsCancellationRequested)
        {
        }
        catch (Exception error)
        {
            if (!closing && !request.IsCancellationRequested)
                StatusText.Text = "쪽을 표시할 수 없습니다. " + error.Message;
        }
        finally
        {
            renderRequest = null;
            if (!closing && page is not null && version != viewVersion)
                renderTimer.Start();
        }
    }

    private void RefreshButtons()
    {
        OpenButton.IsEnabled = !closing && !picking;
        CloseButton.IsEnabled = !closing && (fileName is not null || documentRequest is not null);
        PreviousButton.IsEnabled = !closing && page is { Page: > 0 };
        NextButton.IsEnabled = !closing && page is PageInfo current && current.Page + 1 < current.PageCount;
        ZoomOutButton.IsEnabled = !closing && page is not null && zoom > MinZoom;
        ZoomInButton.IsEnabled = !closing && page is not null && zoom < MaxZoom;
    }

    private async void OnClosing(AppWindow sender, AppWindowClosingEventArgs args)
    {
        args.Cancel = true;
        if (closing)
            return;

        closing = true;
        documentRequest?.Cancel();
        ClearPage();
        RefreshButtons();
        StatusText.Text = "닫는 중…";
        try
        {
            await worker.CloseAsync();
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
        }
        finally
        {
            sender.Closing -= OnClosing;
            Close();
        }
    }
}
