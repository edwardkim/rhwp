namespace Rhwp.WinUI;

internal readonly record struct PagePlacement(uint Index, double Left, double Top, double Width, double Height)
{
    public double Bottom => Top + Height;
}

internal sealed class ContinuousPageLayout
{
    private const double Gap = 16;
    private PageInfo?[] measured = [];
    private PagePlacement[] placements = [];
    private double estimatedWidth;
    private double estimatedHeight;
    private DocumentTarget? target;
    private ulong fontGeneration;

    public double Width { get; private set; }
    public double Height { get; private set; }
    public ulong PlacementVersion { get; private set; }

    public bool UpdateDocument(PageInfo page)
    {
        int count = checked((int)page.PageCount);
        bool replace = target?.DocumentId != page.Target.DocumentId;
        target = page.Target;
        fontGeneration = page.FontGeneration;
        if (replace)
        {
            estimatedWidth = page.Width;
            estimatedHeight = page.Height;
            measured = [];
        }
        bool resized = placements.Length != count || replace;
        if (resized)
        {
            Array.Resize(ref measured, count);
            placements = new PagePlacement[count];
        }

        bool changed = Measure(page);
        if (resized || changed)
            Rebuild();
        return resized || changed;
    }

    public bool Matches(PageInfo page) =>
        target == page.Target && fontGeneration == page.FontGeneration && placements.Length == page.PageCount;

    public bool Measure(PageInfo page)
    {
        PageInfo? previous = measured[page.Page];
        measured[page.Page] = page;
        return (previous?.Width ?? estimatedWidth) != page.Width ||
            (previous?.Height ?? estimatedHeight) != page.Height;
    }

    public void Rebuild()
    {
        PlacementVersion++;
        Width = 0;
        foreach (PageInfo? page in measured)
            Width = Math.Max(Width, page?.Width ?? estimatedWidth);

        double top = 0;
        for (int index = 0; index < placements.Length; index++)
        {
            double width = measured[index]?.Width ?? estimatedWidth;
            double height = measured[index]?.Height ?? estimatedHeight;
            placements[index] = new PagePlacement((uint)index, (Width - width) / 2, top, width, height);
            top += height + Gap;
        }
        Height = Math.Max(0, top - Gap);
    }

    public bool TryGet(uint index, out PagePlacement placement)
    {
        if (index < placements.Length)
        {
            placement = placements[index];
            return true;
        }
        placement = default;
        return false;
    }

    public PageInfo? Measured(PageInfo current, uint index)
    {
        PageInfo? known = index < measured.Length ? measured[index] : null;
        return known?.Target == current.Target && known.FontGeneration == current.FontGeneration &&
            known.PageCount == current.PageCount ? known : null;
    }

    public PageInfo Describe(PageInfo document, PagePlacement placement) => document with
    {
        Page = placement.Index, Width = placement.Width, Height = placement.Height,
    };

    public PagePlacement AtOffset(double y)
    {
        int low = 0;
        int high = placements.Length - 1;
        while (low < high)
        {
            int middle = low + (high - low + 1) / 2;
            if (placements[middle].Top > y)
                high = middle - 1;
            else
                low = middle;
        }
        return placements[low];
    }

    public bool TryHit(double x, double y, out PagePlacement placement)
    {
        placement = AtOffset(y);
        return x >= placement.Left && x < placement.Left + placement.Width &&
            y >= placement.Top && y < placement.Bottom;
    }

    public PagePlacement NearestPage(double y)
    {
        PagePlacement above = AtOffset(y);
        return y >= above.Bottom && TryGet(above.Index + 1, out PagePlacement below) &&
            below.Top - y <= y - above.Bottom ? below : above;
    }

    public IEnumerable<PagePlacement> Intersecting(double top, double bottom)
    {
        for (uint index = AtOffset(top).Index; index < placements.Length; index++)
        {
            PagePlacement placement = placements[index];
            if (placement.Top >= bottom)
                yield break;
            if (placement.Bottom > top)
                yield return placement;
        }
    }
}
