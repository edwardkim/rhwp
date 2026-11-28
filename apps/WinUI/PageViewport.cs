namespace Rhwp.WinUI;

internal sealed record DocumentTarget(ulong DocumentId, ulong Revision);
internal sealed record PageInfo(DocumentTarget Target, ulong FontGeneration, uint Page,
    uint PageCount, double Width, double Height);
internal sealed record RenderRegion(double X, double Y, double Width, double Height);

internal sealed record PageViewport(RenderRegion Region, double Scale, double DeviceScale)
{
    internal const int MaxRasterDimension = 2048;

    public static PageViewport? Visible(PageInfo page, double offsetX, double offsetY,
        double viewportWidth, double viewportHeight, double zoom, double deviceScale)
    {
        if (!double.IsFinite(offsetX) || !double.IsFinite(offsetY) ||
            !double.IsFinite(viewportWidth) || !double.IsFinite(viewportHeight) ||
            !double.IsFinite(zoom) || !double.IsFinite(deviceScale) ||
            viewportWidth <= 0 || viewportHeight <= 0 || zoom <= 0 || deviceScale <= 0)
            return null;

        double x = Math.Max(0, offsetX / zoom);
        double y = Math.Max(0, offsetY / zoom);
        double width = Math.Min(page.Width - x, viewportWidth / zoom);
        double height = Math.Min(page.Height - y, viewportHeight / zoom);
        if (width <= 0 || height <= 0)
            return null;

        // The native pixel window rounds both edges outward. Leave room for
        // both edges so the returned frame is at most 2048² pixels (16 MiB).
        double scale = Math.Min(zoom * deviceScale,
            (MaxRasterDimension - 2) / Math.Max(width, height));
        return new PageViewport(new RenderRegion(x, y, width, height), scale, scale / zoom);
    }
}

internal sealed record PageFrame(int PixelWidth, int PixelHeight, int OriginX, int OriginY,
    double DeviceScale, byte[] Bgra);
