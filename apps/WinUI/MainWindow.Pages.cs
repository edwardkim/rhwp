using System.Runtime.InteropServices.WindowsRuntime;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Foundation;
using Windows.Graphics.Imaging;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private const double PageOverscan = 96;
    private const int PagesPerRender = 4;
    private ContinuousPageLayout? continuousPages;
    private readonly Dictionary<uint, PageSurface> pageSurfaces = [];
    private ulong? firstPaintDocument;
    private uint? pendingPriorityRenderPage;
    private bool priorityRenderPosted;

    private void ResetContinuousPages()
    {
        continuousPages = null;
        firstPaintDocument = null;
        pendingPriorityRenderPage = null;
        ClearPictureObjectSelection();
        pageSurfaces.Clear();
        PageLayers.Children.Clear();
        PageCanvas.Width = 0;
        PageCanvas.Height = 0;
    }

    private void SetPageSize(PageInfo current)
    {
        var layout = continuousPages ??= new ContinuousPageLayout();
        if (layout.UpdateDocument(current))
            UpdatePageExtent(layout);
        RefreshViewportOverlays(current);
    }

    private void UpdatePageExtent(ContinuousPageLayout layout)
    {
        PageCanvas.Width = layout.Width * zoom;
        PageCanvas.Height = layout.Height * zoom;
        foreach (var (index, surface) in pageSurfaces)
            if (layout.TryGet(index, out PagePlacement placement))
                ArrangePage(surface, placement);
    }

    private void RefreshViewportOverlays(PageInfo? current)
    {
        UpdateAutomationDocument(current);
        DrawSearchHighlight(current);
        DrawDirectGeometry();
        DrawPictureObjectSelection();
        DrawTableObjectSelection();
    }

    private Task GoToPageAsync(PageInfo current, uint index)
    {
        if (continuousPages is { } layout && layout.TryGet(index, out PagePlacement destination))
        {
            page = layout.Describe(current, destination);
            PageScroll.UpdateLayout();
            PageScroll.ChangeView(null, destination.Top * zoom, null, true);
            RefreshViewportOverlays(page);
            RefreshButtons();
            QueueRender();
        }
        return Task.CompletedTask;
    }

    private async Task<PagePlacement?> EnsurePagePlacementAsync(PageInfo current, uint index,
        CancellationToken cancellation)
    {
        if (continuousPages is not { } layout || !layout.Matches(current) || !layout.TryGet(index, out _))
            return null;

        if (layout.Measured(current, index) is null)
        {
            PageInfo measured = await worker.GetPageAsync(current, index, cancellation);
            if (closing || !ReferenceEquals(continuousPages, layout) || page is not { } active ||
                !layout.Matches(active) || !layout.Matches(current))
                return null;

            if (ApplyPageMeasurements(layout, [measured]))
                QueueRender();
        }
        return layout.TryGet(index, out PagePlacement placement) ? placement : null;
    }

    private void ShowDocumentPoint(PagePlacement placement, double localX, double localY, double height)
    {
        double x = (placement.Left + localX) * zoom;
        double y = (placement.Top + localY) * zoom;
        double left = PageScroll.HorizontalOffset;
        double top = PageScroll.VerticalOffset;
        double? horizontal = x < left || x + 2 > left + PageScroll.ViewportWidth
            ? Math.Max(0, x - 16) : null;
        double? vertical = y < top || y + height * zoom > top + PageScroll.ViewportHeight
            ? Math.Max(0, y - 24) : null;
        if (horizontal is not null || vertical is not null)
        {
            PageScroll.UpdateLayout();
            PageScroll.ChangeView(horizontal, vertical, null, true);
        }
    }

    private void OnViewChanged(object? sender, ScrollViewerViewChangedEventArgs args)
    {
        if (HandleGestureZoom(args.IsIntermediate))
            return;
        PageInfo? previous = page;
        UpdateViewportPage();
        if (ReferenceEquals(page, previous) && selectedTable is not null)
            DrawTableObjectSelection();
        QueueRender();
    }

    private void UpdateViewportPage()
    {
        if (page is not PageInfo current || continuousPages is not { } layout || !layout.Matches(current))
            return;

        double top = PageScroll.VerticalOffset / zoom;
        PagePlacement visible = top > 0 && PageScroll.VerticalOffset >= PageScroll.ScrollableHeight - 0.5
            ? layout.AtOffset(layout.Height)
            : layout.AtOffset(top);
        if (current.Page == visible.Index && current.Width == visible.Width && current.Height == visible.Height)
            return;

        page = layout.Describe(current, visible);
        RefreshViewportOverlays(page);
        RefreshButtons();
    }

    private void QueueRender(uint? priorityPage = null)
    {
        if (closing || page is not { } current || continuousPages?.Matches(current) != true)
            return;

        viewVersion++;
        if (priorityPage is uint index)
            pendingPriorityRenderPage = index;
        ScheduleRender();
    }

    private void ScheduleRender()
    {
        if (closing || renderRequest is not null || page is not { } current || continuousPages?.Matches(current) != true)
            return;

        if (pendingPriorityRenderPage is not null || firstPaintDocument == current.Target.DocumentId)
        {
            renderTimer.Stop();
            if (!priorityRenderPosted)
            {
                priorityRenderPosted = DispatcherQueue.TryEnqueue(async () =>
                {
                    priorityRenderPosted = false;
                    if (pendingPriorityRenderPage is not null || firstPaintDocument is not null)
                        await RenderPagesAsync();
                });
            }
            if (priorityRenderPosted)
                return;
        }

        if (!renderTimer.IsRunning)
            renderTimer.Start();
    }

    private async void OnRenderTick(DispatcherQueueTimer sender, object args) => await RenderPagesAsync();

    private async Task RenderPagesAsync()
    {
        if (closing || renderRequest is not null || page is not PageInfo current ||
            continuousPages is not { } layout || !layout.Matches(current) ||
            PageScroll.ViewportHeight <= 0 || PageScroll.ViewportWidth <= 0 || PageScroll.ZoomFactor != 1)
            return;

        bool firstPaint = firstPaintDocument == current.Target.DocumentId;
        uint? priorityPage = pendingPriorityRenderPage ?? (firstPaint ? current.Page : null);
        pendingPriorityRenderPage = null;
        ulong version = viewVersion;
        ulong placementVersion = layout.PlacementVersion;
        double renderZoom = zoom;
        double deviceScale = Root.XamlRoot?.RasterizationScale ?? 1;
        using var request = new CancellationTokenSource();
        renderRequest = request;
        bool hasMore = false;
        try
        {
            var wanted = VisiblePages(layout).ToArray();
            TrimPageSurfaces(wanted);
            PagePlacement[] batch = wanted;
            if (priorityPage is uint preferred && wanted.Length > 1)
            {
                // Draw the first visible or just-edited sheet before measuring the neighboring sheets.
                batch = [wanted.FirstOrDefault(placement => placement.Index == preferred &&
                    GetPageViewport(layout.Describe(current, placement), placement, deviceScale, 0) is not null,
                    wanted[0])];
                hasMore = true;
            }

            foreach (PagePlacement placement in batch)
                GetPageSurface(placement);

            var measuredPages = new List<PageInfo>(PagesPerRender);
            foreach (PagePlacement placement in batch)
            {
                if (layout.Measured(current, placement.Index) is not null)
                    continue;
                if (measuredPages.Count == PagesPerRender)
                {
                    hasMore = true;
                    break;
                }
                PageInfo measured = await worker.GetPageAsync(current, placement.Index, request.Token);
                if (!OwnsRender(current, layout, placementVersion, renderZoom, deviceScale, request))
                    return;
                measuredPages.Add(measured);
            }
            if (ApplyPageMeasurements(layout, measuredPages))
            {
                hasMore = true;
                if (priorityPage is null || !layout.TryGet(batch[0].Index, out PagePlacement updated))
                    return;
                placementVersion = layout.PlacementVersion;
                batch = [updated];
            }

            int rendering = 0;
            foreach (PagePlacement placement in batch)
            {
                if (!OwnsRender(current, layout, placementVersion, renderZoom, deviceScale, request))
                    return;
                if (layout.Measured(current, placement.Index) is not PageInfo measured)
                    continue;

                PageViewport? visible = GetPageViewport(measured, placement, deviceScale, 0);
                if (((firstPaint ? visible : null) ??
                    GetPageViewport(measured, placement, deviceScale, PageOverscan)) is not { } viewport)
                    continue;
                PageViewport needed = visible ?? viewport;
                PageSurface surface = GetPageSurface(placement);
                if (surface.Covers(measured, needed, viewport, zoom, deviceScale))
                    continue;
                if (rendering++ == PagesPerRender)
                {
                    hasMore = true;
                    break;
                }

                PageFrame frame = await worker.RenderAsync(measured, viewport, request.Token);
                if (!OwnsRender(current, layout, placementVersion, renderZoom, deviceScale, request) ||
                    !IsFrameUseful(measured, placement, viewport, deviceScale))
                    continue;

                using var bitmap = SoftwareBitmap.CreateCopyFromBuffer(frame.Bgra.AsBuffer(), BitmapPixelFormat.Bgra8,
                    frame.PixelWidth, frame.PixelHeight, BitmapAlphaMode.Premultiplied);
                var source = new SoftwareBitmapSource();
                await source.SetBitmapAsync(bitmap);
                if (!OwnsRender(current, layout, placementVersion, renderZoom, deviceScale, request) ||
                    !IsFrameUseful(measured, placement, viewport, deviceScale))
                    continue;

                surface.Commit(measured, viewport, renderZoom, deviceScale, frame, source);
                ArrangePage(surface, placement);
                CompleteFirstPaint(current.Target.DocumentId);
                if (pdfCancellation is null && StatusText.Text == RenderingStatus)
                    StatusText.Text = documentError ?? string.Empty;
            }
        }
        catch (OperationCanceledException) when (closing || request.IsCancellationRequested)
        {
        }
        catch (Exception error)
        {
            if (!closing && !request.IsCancellationRequested)
            {
                StatusText.Text = "쪽을 표시할 수 없습니다. " + error.Message;
                CompleteFirstPaint(current.Target.DocumentId);
            }
        }
        finally
        {
            renderRequest = null;
            if (hasMore || version != viewVersion)
                ScheduleRender();
        }
    }

    private void CompleteFirstPaint(ulong documentId)
    {
        if (firstPaintDocument != documentId || page is not { } active || active.Target.DocumentId != documentId)
            return;

        firstPaintDocument = null;
        UpdateAutomationDocument(active);
    }

    private bool ApplyPageMeasurements(ContinuousPageLayout layout, IEnumerable<PageInfo> measuredPages)
    {
        bool changed = false;
        foreach (PageInfo measured in measuredPages)
            changed |= layout.Measure(measured);
        if (!changed)
            return false;

        // Preserve the top visible sheet and the offset inside it when different paper sizes reflow the document.
        double before = PageScroll.VerticalOffset;
        PagePlacement anchor = layout.AtOffset(before / zoom);
        double localOffset = before / zoom - anchor.Top;
        layout.Rebuild();
        UpdatePageExtent(layout);
        if (layout.TryGet(anchor.Index, out PagePlacement updated))
        {
            double offset = (updated.Top + Math.Min(localOffset, updated.Height)) * zoom;
            if (Math.Abs(before - offset) > 0.5)
            {
                PageScroll.UpdateLayout();
                PageScroll.ChangeView(null, offset, null, true);
            }
        }
        UpdateViewportPage();
        RefreshViewportOverlays(page);
        return true;
    }

    private bool OwnsRender(PageInfo current, ContinuousPageLayout layout, ulong placementVersion,
        double renderZoom, double deviceScale, CancellationTokenSource request) =>
        !closing && !request.IsCancellationRequested && ReferenceEquals(continuousPages, layout) &&
        layout.PlacementVersion == placementVersion && page is { } active &&
        active.Target == current.Target && active.FontGeneration == current.FontGeneration &&
        zoom == renderZoom && PageScroll.ZoomFactor == 1 && (Root.XamlRoot?.RasterizationScale ?? 1) == deviceScale;

    private IEnumerable<PagePlacement> VisiblePages(ContinuousPageLayout layout)
    {
        double top = PageScroll.VerticalOffset / zoom;
        double bottom = (PageScroll.VerticalOffset + PageScroll.ViewportHeight) / zoom;
        double margin = PageOverscan / zoom;
        return layout.Intersecting(top - margin, bottom + margin)
            .OrderByDescending(placement => Math.Min(placement.Bottom, bottom) - Math.Max(placement.Top, top));
    }

    private PageViewport? GetPageViewport(PageInfo current, PagePlacement placement, double deviceScale, double margin)
    {
        double pageX = placement.Left * zoom;
        double pageY = placement.Top * zoom;
        double left = Math.Max(PageScroll.HorizontalOffset - margin, pageX);
        double top = Math.Max(PageScroll.VerticalOffset - margin, pageY);
        double right = Math.Min(PageScroll.HorizontalOffset + PageScroll.ViewportWidth + margin,
            pageX + placement.Width * zoom);
        double bottom = Math.Min(PageScroll.VerticalOffset + PageScroll.ViewportHeight + margin,
            pageY + placement.Height * zoom);
        return PageViewport.Visible(current, left - pageX, top - pageY, right - left, bottom - top, zoom, deviceScale);
    }

    private bool IsFrameUseful(PageInfo page, PagePlacement placement, PageViewport requested, double deviceScale)
    {
        if ((GetPageViewport(page, placement, deviceScale, 0) ??
            GetPageViewport(page, placement, deviceScale, PageOverscan)) is not { Region: var visible })
            return false;

        RenderRegion region = requested.Region;
        return region.X < visible.X + visible.Width && region.X + region.Width > visible.X &&
            region.Y < visible.Y + visible.Height && region.Y + region.Height > visible.Y;
    }

    private void TrimPageSurfaces(PagePlacement[] wanted)
    {
        var keep = wanted.Select(placement => placement.Index).ToHashSet();
        foreach (uint index in pageSurfaces.Keys.Where(index => !keep.Contains(index)).ToArray())
        {
            PageLayers.Children.Remove(pageSurfaces[index].Layer);
            pageSurfaces.Remove(index);
        }
    }

    private PageSurface GetPageSurface(PagePlacement placement)
    {
        if (pageSurfaces.TryGetValue(placement.Index, out PageSurface? existing))
            return existing;

        var raster = new Image { Stretch = Stretch.Fill };
        var sheet = new Canvas
        {
            Background = (Brush)PageLayers.Resources["PagePaperBrush"],
            Clip = new RectangleGeometry(),
            Children = { raster },
        };
        var surface = new PageSurface(sheet, raster);
        pageSurfaces.Add(placement.Index, surface);
        PageLayers.Children.Add(sheet);
        ArrangePage(surface, placement);
        return surface;
    }

    private void ArrangePage(PageSurface surface, PagePlacement placement)
    {
        surface.Layer.Width = placement.Width * zoom;
        surface.Layer.Height = placement.Height * zoom;
        ((RectangleGeometry)surface.Layer.Clip!).Rect = new Rect(0, 0, surface.Layer.Width, surface.Layer.Height);
        Canvas.SetLeft(surface.Layer, placement.Left * zoom);
        Canvas.SetTop(surface.Layer, placement.Top * zoom);
        if (surface.RenderedBounds is { } bounds)
        {
            Canvas.SetLeft(surface.Raster, bounds.X * zoom);
            Canvas.SetTop(surface.Raster, bounds.Y * zoom);
            surface.Raster.Width = bounds.Width * zoom;
            surface.Raster.Height = bounds.Height * zoom;
        }
    }

    private sealed class PageSurface(Canvas layer, Image raster)
    {
        public Canvas Layer { get; } = layer;
        public Image Raster { get; } = raster;
        public Rect? RenderedBounds { get; private set; }
        private (DocumentTarget Target, ulong FontGeneration, double Zoom, double DeviceScale, double Scale)? stamp;

        public bool Covers(PageInfo current, PageViewport needed, PageViewport padded, double zoom, double deviceScale)
        {
            if (stamp != (current.Target, current.FontGeneration, zoom, deviceScale, padded.Scale) ||
                RenderedBounds is not { } bounds)
                return false;

            RenderRegion region = needed.Region;
            const double tolerance = 0.00001;
            return bounds.X <= region.X + tolerance && bounds.Y <= region.Y + tolerance &&
                bounds.Right >= region.X + region.Width - tolerance &&
                bounds.Bottom >= region.Y + region.Height - tolerance;
        }

        public void Commit(PageInfo page, PageViewport viewport, double zoom, double deviceScale, PageFrame frame,
            ImageSource source)
        {
            double scale = frame.DeviceScale * zoom;
            RenderedBounds = new Rect(frame.OriginX / scale, frame.OriginY / scale,
                frame.PixelWidth / scale, frame.PixelHeight / scale);
            stamp = (page.Target, page.FontGeneration, zoom, deviceScale, viewport.Scale);
            Raster.Source = source;
        }
    }
}
