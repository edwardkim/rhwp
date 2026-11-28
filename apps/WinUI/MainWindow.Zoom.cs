using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Input;
using Windows.System;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private enum ZoomFitMode { Width, Page }

    private ZoomFitMode? zoomFitMode;
    private bool fitResizeScheduled;
    private ulong zoomRequest;
    private bool applyingGestureZoom;

    private void OnZoomOutClicked(object sender, RoutedEventArgs args) => ZoomTo(zoom / 1.25);
    private void OnZoomInClicked(object sender, RoutedEventArgs args) => ZoomTo(zoom * 1.25);
    private void OnActualSizeClicked(object sender, RoutedEventArgs args) => ZoomTo(1);

    private void OnZoomAcceleratorInvoked(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args)
    {
        if (FocusManager.GetFocusedElement(Root.XamlRoot) is Microsoft.UI.Xaml.Controls.TextBox focused &&
            focused != DirectInput)
            return;

        args.Handled = TryZoomFromKey(sender.Key);
    }

    private bool TryZoomFromKey(VirtualKey key)
    {
        if (!DocumentActionsAvailable || page is null)
            return false;

        double? next = key switch
        {
            VirtualKey.Up or VirtualKey.Right or VirtualKey.Add => zoom * 1.25,
            VirtualKey.Down or VirtualKey.Left or VirtualKey.Subtract => zoom / 1.25,
            VirtualKey.Number0 or VirtualKey.NumberPad0 => 1,
            _ => null,
        };
        if (next is not double value)
            return false;

        ZoomTo(value);
        return true;
    }

    private bool HandleGestureZoom(bool isIntermediate)
    {
        if (applyingGestureZoom)
            return true;
        float factor = PageScroll.ZoomFactor;
        if (factor == 1)
            return false;
        if (isIntermediate || closing)
            return true;

        applyingGestureZoom = true;
        try
        {
            if (page is not { } current || continuousPages is not { } layout || !layout.Matches(current))
            {
                PageScroll.ChangeView(null, null, 1, true);
                return true;
            }

            // WinUI scales the current visual during the pinch. Fold its scale into
            // the document's zoom at the end so Skia redraws crisp pixels at that scale.
            double transient = zoom * factor;
            double next = Math.Clamp(transient, Math.Min(MinZoom, zoom), MaxZoom);
            double ratio = next / transient;
            double left = PageScroll.HorizontalOffset * ratio;
            double top = PageScroll.VerticalOffset * ratio;
            zoomRequest++;
            zoomFitMode = null;
            renderRequest?.Cancel();
            zoom = next;
            UpdatePageExtent(layout);
            PageScroll.UpdateLayout();
            PageScroll.ChangeView(left, top, 1, true);
            UpdateViewportPage();
            RefreshViewportOverlays(page);
            RefreshButtons();
            QueueRender();
            return true;
        }
        finally
        {
            applyingGestureZoom = false;
        }
    }

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
