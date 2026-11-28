using Microsoft.UI.Xaml;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private enum ZoomFitMode { Width, Page }

    private ZoomFitMode? zoomFitMode;
    private bool fitResizeScheduled;
    private ulong zoomRequest;

    private void OnZoomOutClicked(object sender, RoutedEventArgs args) => ZoomTo(zoom / 1.25);
    private void OnZoomInClicked(object sender, RoutedEventArgs args) => ZoomTo(zoom * 1.25);
    private void OnActualSizeClicked(object sender, RoutedEventArgs args) => ZoomTo(1);

    private async void OnFitWidthClicked(object sender, RoutedEventArgs args)
    {
        if (DocumentActionsAvailable)
            await FitZoomAsync(ZoomFitMode.Width, userInitiated: true);
    }

    private async void OnFitPageClicked(object sender, RoutedEventArgs args)
    {
        if (DocumentActionsAvailable)
            await FitZoomAsync(ZoomFitMode.Page, userInitiated: true);
    }

    private void ZoomTo(double value)
    {
        if (!DocumentActionsAvailable || page is not PageInfo current || continuousPages is not { } layout)
            return;

        zoomRequest++;
        zoomFitMode = null;
        ApplyZoom(current, layout, Math.Clamp(value, MinZoom, MaxZoom));
    }

    private async Task FitZoomAsync(ZoomFitMode mode, bool userInitiated)
    {
        if (closing || page is not PageInfo current || continuousPages is not { } layout)
            return;

        ulong request = userInitiated ? ++zoomRequest : zoomRequest;
        try
        {
            PagePlacement? placement = await EnsurePagePlacementAsync(current, current.Page, CancellationToken.None);
            if (closing || request != zoomRequest || (userInitiated ? !DocumentActionsAvailable : zoomFitMode != mode) ||
                placement is not { } sheet || !ReferenceEquals(continuousPages, layout) ||
                page is not { } active || active.Target != current.Target || active.Page != current.Page ||
                active.FontGeneration != current.FontGeneration ||
                PageScroll.ViewportWidth <= 0 || PageScroll.ViewportHeight <= 0)
                return;

            double nextZoom = PageScroll.ViewportWidth / sheet.Width;
            if (mode == ZoomFitMode.Page)
                nextZoom = Math.Min(nextZoom, PageScroll.ViewportHeight / sheet.Height);
            if (!double.IsFinite(nextZoom) || nextZoom <= 0)
                return;

            // Fit can require less than 50% even for an ordinary page in a small window.
            zoomFitMode = mode;
            ApplyZoom(active, layout, Math.Min(nextZoom, MaxZoom), sheet,
                userInitiated && mode == ZoomFitMode.Page);
        }
        catch (Exception error)
        {
            if (!closing && request == zoomRequest && ReferenceEquals(continuousPages, layout) &&
                page?.Target == current.Target)
                StatusText.Text = "쪽을 화면에 맞출 수 없습니다. " + error.Message;
        }
    }

    private void ApplyZoom(PageInfo current, ContinuousPageLayout layout, double nextZoom,
        PagePlacement? sheet = null, bool alignPageTop = false)
    {
        double ratio = nextZoom / zoom;
        double left = PageScroll.HorizontalOffset * ratio;
        double top = PageScroll.VerticalOffset * ratio;
        if (nextZoom != zoom)
        {
            renderRequest?.Cancel();
            zoom = nextZoom;
            UpdatePageExtent(layout);
            RefreshViewportOverlays(current);
            PageScroll.UpdateLayout();
        }
        else if (sheet is null)
        {
            return;
        }

        if (sheet is { } placement)
        {
            left = Math.Max(0, (placement.Left + placement.Width / 2) * zoom - PageScroll.ViewportWidth / 2);
            if (alignPageTop)
                top = placement.Top * zoom;
        }
        PageScroll.ChangeView(left, top, null, true);
        RefreshButtons();
        QueueRender();
    }

    private void OnViewportSizeChanged(object sender, SizeChangedEventArgs args)
    {
        QueueRender();
        if (zoomFitMode is null || fitResizeScheduled)
            return;

        // SizeChanged runs during layout. Read the actual viewport after that pass.
        ulong request = zoomRequest;
        fitResizeScheduled = DispatcherQueue.TryEnqueue(async () =>
        {
            try
            {
                if (request == zoomRequest && zoomFitMode is { } mode)
                    await FitZoomAsync(mode, userInitiated: false);
            }
            finally
            {
                fitResizeScheduled = false;
            }
        });
    }

    private void RefreshFitZoomActions(bool enabled)
    {
        FitWidthMenuItem.IsEnabled = enabled;
        FitPageMenuItem.IsEnabled = enabled;
        ActualSizeMenuItem.IsEnabled = enabled;
    }
}
