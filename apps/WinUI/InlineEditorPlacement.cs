namespace Rhwp.WinUI;

internal sealed record InlineEditorPlacement(double Left, double Top, double Width,
    double MaxTextHeight, double FontSize)
{
    public static InlineEditorPlacement? Create(PageInfo page, PageCaret caret, PageCaret? paragraphStart,
        double zoom, double viewportWidth, double viewportHeight)
    {
        double pageWidth = page.Width * zoom;
        double pageHeight = page.Height * zoom;
        if (!double.IsFinite(pageWidth) || !double.IsFinite(pageHeight) || pageWidth <= 0 || pageHeight <= 0 ||
            !double.IsFinite(zoom) || zoom <= 0 || !IsPageCaret(page, caret))
            return null;

        PageCaret start = paragraphStart is not null && IsPageCaret(page, paragraphStart) ? paragraphStart : caret;
        double availableWidth = Available(pageWidth, viewportWidth);
        double availableHeight = Available(pageHeight, viewportHeight);
        double preferredWidth = Math.Min(440, availableWidth);
        double left = start.X;
        if (caret.CellBounds is { Width: > 0 } cell && double.IsFinite(cell.X) &&
            double.IsFinite(cell.Width * zoom))
        {
            // Match the cell when it can fit the editor controls. On narrower
            // cells the overlay still starts at the cell or fits the page edge.
            left = cell.X;
            preferredWidth = cell.Width * zoom;
        }
        double width = Math.Clamp(preferredWidth, Math.Min(168, availableWidth), Math.Min(600, availableWidth));
        double fontSize = Math.Clamp(caret.Height * zoom * 0.8, 12, 40);
        return new InlineEditorPlacement(Math.Clamp(left * zoom, 0, pageWidth - width), start.Y * zoom, width,
            Math.Clamp(availableHeight * 0.6, 32, 280), fontSize);
    }

    public double FitTop(double pageHeight, double editorHeight) =>
        Math.Clamp(Top, 0, Math.Max(0, pageHeight - editorHeight));

    private static double Available(double pageSize, double viewportSize) =>
        double.IsFinite(viewportSize) && viewportSize > 0 ? Math.Min(pageSize, viewportSize) : pageSize;

    private static bool IsPageCaret(PageInfo page, PageCaret caret) =>
        caret.Page == page.Page && double.IsFinite(caret.X) && double.IsFinite(caret.Y) &&
        double.IsFinite(caret.Height) && caret.Height > 0 &&
        caret.X >= 0 && caret.X <= page.Width && caret.Y >= 0 && caret.Y <= page.Height;
}
