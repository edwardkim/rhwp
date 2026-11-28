using System.Diagnostics;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media.Imaging;
using Microsoft.Windows.Storage.Pickers;
using Windows.ApplicationModel.DataTransfer;
using Windows.Graphics.Imaging;
using Windows.Security.Cryptography;
using Windows.Storage;

namespace Rhwp.WinUI;

public sealed partial class MainWindow : Window
{
    private const string IdleStatus = "HWP 또는 HWPX 파일을 여세요.";
    private const string RenderingStatus = "쪽 표시 중…";
    private const double MinZoom = 0.5;
    private const double MaxZoom = 4;
    private readonly NativeWorkerDispatcher worker = new();
    private readonly TaskCompletionSource windowLoaded = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private readonly DispatcherQueueTimer renderTimer;
    private CancellationTokenSource? documentRequest;
    private CancellationTokenSource? renderRequest;
    private PageInfo? page;
    private string? fileName;
    private string? documentError;
    private ulong viewVersion;
    private double zoom = 1;
    private bool picking;
    private bool closing;
    private bool DocumentActionsAvailable => !closing && !editorBusy && !directQueueRunning && !picking &&
        documentRequest is null && !HasActiveComposition;

    public MainWindow()
    {
        InitializeComponent();
        searchUiReady = true;
        renderTimer = DispatcherQueue.CreateTimer();
        renderTimer.Interval = TimeSpan.FromMilliseconds(32);
        renderTimer.IsRepeating = false;
        renderTimer.Tick += OnRenderTick;
        recoveryTimer = DispatcherQueue.CreateTimer();
        recoveryTimer.Interval = TimeSpan.FromMilliseconds(650);
        recoveryTimer.IsRepeating = false;
        recoveryTimer.Tick += OnRecoveryTick;
        AppWindow.Closing += OnClosing;
    }

    private async void OnOpenClicked(object sender, RoutedEventArgs args)
    {
        if (!DocumentActionsAvailable)
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

    internal async Task OpenFileAsync(string path)
    {
        if (!DocumentActionsAvailable)
            return;

        UnsavedDecision decision = await ResolveUnsavedChangesAsync();
        if (decision == UnsavedDecision.Cancel)
            return;

        OpenedDocument? opened = null;
        await ChangeDocumentAsync(async token =>
        {
            opened = await worker.OpenWithLocationAsync(path, token, discardChanges: decision == UnsavedDecision.Discard);
            return opened.Page;
        }, Path.GetFileName(path), resetZoom: true, async result =>
        {
            AdoptSaveLocation(result, opened?.Location);
            await ClearRecoveryAsync();
        });
    }

    private void OnFilesDragOver(object sender, DragEventArgs args)
    {
        if (DocumentActionsAvailable && args.DataView.Contains(StandardDataFormats.StorageItems))
        {
            args.AcceptedOperation = DataPackageOperation.Copy;
            args.Handled = true;
        }
    }

    private async void OnFilesDropped(object sender, DragEventArgs args)
    {
        if (!DocumentActionsAvailable || !args.DataView.Contains(StandardDataFormats.StorageItems))
            return;

        args.Handled = true;
        var deferral = args.GetDeferral();
        string? path = null;
        try
        {
            var files = await args.DataView.GetStorageItemsAsync();
            path = FileActivation.FirstDocumentPath(files.OfType<StorageFile>().Select(file => file.Path));
            if (path is null && !closing)
                StatusText.Text = "HWP 또는 HWPX 파일을 놓으세요.";
        }
        catch (Exception error)
        {
            if (!closing)
                StatusText.Text = "놓은 파일을 읽을 수 없습니다. " + error.Message;
        }
        finally
        {
            deferral.Complete();
        }

        if (path is not null)
            await OpenFileAsync(path);
    }

    private async void OnCloseClicked(object sender, RoutedEventArgs args)
    {
        if (!DocumentActionsAvailable)
            return;

        UnsavedDecision decision = await ResolveUnsavedChangesAsync();
        if (decision == UnsavedDecision.Cancel)
            return;

        await ChangeDocumentAsync(async token =>
        {
            await worker.CloseDocumentAsync(token, discardChanges: decision == UnsavedDecision.Discard);
            return null;
        }, null, resetZoom: true, async result =>
        {
            AdoptSaveLocation(result, null);
            await ClearRecoveryAsync();
        });
    }

    private async void OnPreviousClicked(object sender, RoutedEventArgs args)
    {
        if (page is { Page: > 0 } current && DocumentActionsAvailable)
            await GoToPageAsync(current, current.Page - 1);
    }

    private async void OnNextClicked(object sender, RoutedEventArgs args)
    {
        if (page is PageInfo current && current.Page + 1 < current.PageCount && DocumentActionsAvailable)
            await GoToPageAsync(current, current.Page + 1);
    }

    private Task GoToPageAsync(PageInfo current, uint index) =>
        ChangeDocumentAsync(async token => await worker.GetPageAsync(current, index, token), fileName, resetZoom: false);

    private async Task<bool> ChangeDocumentAsync(Func<CancellationToken, Task<PageInfo?>> action,
        string? name, bool resetZoom, Func<PageInfo?, Task>? onCommitted = null)
    {
        // Keep an accepted document action in flight until its UI and worker
        // state have both settled. Later open or drop gestures can then retry.
        if (!DocumentActionsAvailable)
            return false;

        var priorPage = page;
        var priorFile = fileName;
        double priorZoom = zoom;
        documentError = null;
        using var request = new CancellationTokenSource();
        documentRequest = request;
        ClearPage();
        StatusText.Text = name is null ? "문서 닫는 중…" : "문서 읽는 중…";
        RefreshButtons();
        try
        {
            PageInfo? result = await action(request.Token);
            if (closing || request.IsCancellationRequested)
                return false;
            if (resetZoom)
            {
                zoom = 1;
                editorInfo = null;
                InvalidateSearchResults();
            }

            fileName = name;
            page = result;
            if (onCommitted is not null)
                await onCommitted(result);
            UpdateTitle();
            if (result is null)
                StatusText.Text = IdleStatus;
            else
            {
                SetPageSize(result);
                PageScroll.ChangeView(0, 0, null, true);
                StatusText.Text = RenderingStatus;
                QueueRender();
            }
            return true;
        }
        catch (OperationCanceledException) when (closing || request.IsCancellationRequested)
        {
        }
        catch (Exception error)
        {
            if (!closing && !request.IsCancellationRequested)
            {
                documentError = "문서 요청을 처리할 수 없습니다. " + error.Message;
                StatusText.Text = documentError;
                if (priorPage is not null)
                    await RestorePageAsync(priorPage, priorFile, priorZoom, request.Token);
            }
        }
        finally
        {
            if (ReferenceEquals(documentRequest, request))
            {
                documentRequest = null;
                if (!closing)
                    RefreshButtons();
            }
        }

        return false;
    }

    private async Task RestorePageAsync(PageInfo prior, string? name, double priorZoom, CancellationToken token)
    {
        try
        {
            PageInfo restored = await worker.GetPageAsync(prior, prior.Page, token);
            if (closing || token.IsCancellationRequested)
                return;

            page = restored;
            fileName = name;
            zoom = priorZoom;
            SetPageSize(restored);
            UpdateTitle();
            QueueRender();
        }
        catch (Exception)
        {
            // A replacement may have opened while its layout failed to load.
            // Only a validated previous target can supply the restored page.
        }
    }

    private void ClearPage()
    {
        DismissDirectEditor();
        page = null;
        UpdateAutomationDocument(null);
        DrawSearchHighlight(null);
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
        UpdateAutomationDocument(current);
        DrawSearchHighlight(current);
        DrawDirectGeometry();
    }

    private void OnZoomOutClicked(object sender, RoutedEventArgs args) => ZoomTo(zoom / 1.25);
    private void OnZoomInClicked(object sender, RoutedEventArgs args) => ZoomTo(zoom * 1.25);

    private void ZoomTo(double value)
    {
        if (!DocumentActionsAvailable || page is not PageInfo current)
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

    internal Task WaitForWindowLoadedAsync() => windowLoaded.Task;

    private void OnLoaded(object sender, RoutedEventArgs args)
    {
        if (Root.XamlRoot is XamlRoot root)
            root.Changed += OnXamlRootChanged;
        _ = LoadPdfCapabilityAsync();
        QueueRender();
        windowLoaded.TrySetResult();
    }

    private void OnUnloaded(object sender, RoutedEventArgs args)
    {
        if (Root.XamlRoot is XamlRoot root)
            root.Changed -= OnXamlRootChanged;
    }

    private void OnXamlRootChanged(XamlRoot sender, XamlRootChangedEventArgs args) => QueueRender();
    private void OnViewportSizeChanged(object sender, SizeChangedEventArgs args) => QueueRender();
    private void OnViewChanged(object? sender, ScrollViewerViewChangedEventArgs args) => QueueRender();

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
            if (pdfCancellation is null && StatusText.Text == RenderingStatus)
                StatusText.Text = documentError ?? string.Empty;
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
        bool available = DocumentActionsAvailable;
        DirectInput.IsReadOnly = closing || picking || documentRequest is not null ||
            (editorBusy && !directQueueRunning);
        OpenButton.IsEnabled = available;
        SaveButton.IsEnabled = available && SaveTarget is not null;
        SaveCopyButton.IsEnabled = SaveButton.IsEnabled;
        ExportPdfButton.Label = pdfCancellation is null ? "현재 쪽 PDF 미리보기" : "PDF 작업 취소";
        ExportPdfButton.IsEnabled = pdfCancellation is CancellationTokenSource export
            ? !export.IsCancellationRequested
            : available && page is not null && pdfLimits is not null;
        CloseButton.IsEnabled = available && fileName is not null;
        PreviousButton.IsEnabled = available && page is { Page: > 0 };
        NextButton.IsEnabled = available && page is PageInfo current && current.Page + 1 < current.PageCount;
        ZoomOutButton.IsEnabled = available && page is not null && zoom > MinZoom;
        ZoomInButton.IsEnabled = available && page is not null && zoom < MaxZoom;
        UndoButton.IsEnabled = available && page is not null && editorInfo?.CanUndo == true;
        RedoButton.IsEnabled = available && page is not null && editorInfo?.CanRedo == true;
        PagePositionText.Text = page is PageInfo shown ? $"{shown.Page + 1} / {shown.PageCount}쪽" : "";
        ZoomValueText.Text = page is null ? "" : $"{(int)Math.Round(zoom * 100)}%";
        RefreshSearchActions();
        RefreshClipboardActions(available);
    }

    private async void OnClosing(AppWindow sender, AppWindowClosingEventArgs args)
    {
        args.Cancel = true;
        if (closing || picking || documentRequest is not null)
            return;

        UnsavedDecision decision = await ResolveUnsavedChangesAsync();
        if (decision == UnsavedDecision.Cancel)
            return;

        closing = true;
        recoveryTimer.Stop();
        documentRequest?.Cancel();
        ClearPage();
        RefreshButtons();
        StatusText.Text = "닫는 중…";
        try
        {
            await worker.CloseAsync();
            await ClearRecoveryAsync();
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
