using System.Runtime.InteropServices;
using Windows.Foundation;
using Windows.System;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private const int DoubleClickWidthMetric = 36;
    private const int DoubleClickHeightMetric = 37;
    private const int DragWidthMetric = 68;
    private const int DragHeightMetric = 69;

    private readonly record struct TextClick(DocumentTarget Target, uint Page, ulong Timestamp, Point Origin, int Count);

    private TextClick? lastTextClick;
    private Point mouseDownPosition;
    private Size mouseDragTolerance;
    private bool textDragStarted;

    private NativeTextUnit? BeginTextClick(PageInfo page, Point position, ulong timestamp)
    {
        double scale = Root.XamlRoot?.RasterizationScale ?? 1;
        uint dpi = (uint)Math.Round(96 * scale);
        mouseDownPosition = position;
        mouseDragTolerance = GetMouseTolerance(DragWidthMetric, DragHeightMetric, dpi, scale);
        textDragStarted = false;

        if (KeyDown(VirtualKey.Control) || KeyDown(VirtualKey.Shift) || KeyDown(VirtualKey.Menu))
        {
            lastTextClick = null;
            return null;
        }

        int count = 1;
        Point origin = position;
        Size tolerance = GetMouseTolerance(DoubleClickWidthMetric, DoubleClickHeightMetric, dpi, scale);
        if (lastTextClick is { Count: < 3 } previous && previous.Target == page.Target && previous.Page == page.Page &&
            timestamp >= previous.Timestamp && timestamp - previous.Timestamp <= 1000UL * GetDoubleClickTime() &&
            WithinMouseTolerance(previous.Origin, position, tolerance))
        {
            count = previous.Count + 1;
            origin = previous.Origin;
        }

        lastTextClick = new TextClick(page.Target, page.Page, timestamp, origin, count);
        return count switch { 2 => NativeTextUnit.Word, 3 => NativeTextUnit.Sentence, _ => null };
    }

    private bool ContinueTextDrag(Point position)
    {
        if (!textDragStarted && !WithinMouseTolerance(mouseDownPosition, position, mouseDragTolerance))
        {
            textDragStarted = true;
            lastTextClick = null;
        }
        return textDragStarted;
    }

    private static Size GetMouseTolerance(int width, int height, uint dpi, double scale) =>
        new(Math.Max(1, GetSystemMetricsForDpi(width, dpi)) / (2 * scale),
            Math.Max(1, GetSystemMetricsForDpi(height, dpi)) / (2 * scale));

    private static bool WithinMouseTolerance(Point origin, Point position, Size tolerance) =>
        Math.Abs(position.X - origin.X) <= tolerance.Width && Math.Abs(position.Y - origin.Y) <= tolerance.Height;

    private static int CompareInTextContainer(TextPosition left, TextPosition right)
    {
        uint leftParagraph = left.CellPath.Length > 0 ? left.CellPath[^1].Paragraph : left.Paragraph;
        uint rightParagraph = right.CellPath.Length > 0 ? right.CellPath[^1].Paragraph : right.Paragraph;
        return leftParagraph == rightParagraph ? left.ScalarOffset.CompareTo(right.ScalarOffset) :
            leftParagraph.CompareTo(rightParagraph);
    }

    [DllImport("user32.dll", ExactSpelling = true)]
    private static extern uint GetDoubleClickTime();

    [DllImport("user32.dll", ExactSpelling = true)]
    private static extern int GetSystemMetricsForDpi(int index, uint dpi);
}
