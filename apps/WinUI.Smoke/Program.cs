using System.Security.Cryptography;
using Rhwp.WinUI;

if (args is ["--local-agent-contract-only"])
{
    await LocalAgentSmoke.RunAsync();
    return;
}

if (args is ["--local-agent-stub", .. var stubArguments])
{
    await LocalAgentSmoke.RunStubAsync(stubArguments);
    return;
}

if (args is ["--pdf-contract-only"])
{
    await LivePdfSmoke.RunAsync(expectExport: false);
    return;
}

if (args is ["--table-object-contract-only"])
{
    await TableObjectSmoke.RunAsync();
    return;
}

bool testLivePdf = args.Length == 3 && args[2] == "--live-pdf";
if (args.Length != 2 && !testLivePdf)
    throw new ArgumentException("Pass a multipage HWP and a second HWPX path, optionally followed by --live-pdf.");

var dispatcher = new NativeWorkerDispatcher();
try
{
    PageInfo first = await dispatcher.OpenAsync(args[0], CancellationToken.None);
    Assert(first.Page == 0 && first.PageCount >= 2, "multipage HWP");
    PageInfo second = await dispatcher.GetPageAsync(first, 1, CancellationToken.None);

    var overview = Visible(first, 0, 0, 1000, 1000, 1, 1);
    using var cancel = new CancellationTokenSource();
    Task<PageFrame> pending = dispatcher.RenderAsync(first, overview, cancel.Token);
    cancel.Cancel();
    try
    {
        // A very fast render may finish before cancellation was requested.
        CheckFrame(await pending, overview);
    }
    catch (OperationCanceledException)
    {
    }

    PageFrame firstFrame = CheckFrame(await dispatcher.RenderAsync(first, overview, CancellationToken.None), overview);
    var enlarged = Visible(second, 24.75, 101.25, 512, 384, 2, 1.5);
    PageFrame zoomFrame = CheckFrame(await dispatcher.RenderAsync(second, enlarged, CancellationToken.None), enlarged);
    Assert(zoomFrame.OriginX > 0 && zoomFrame.OriginY > 0, "scrolled raster origin");
    Assert(zoomFrame.PixelWidth / zoomFrame.DeviceScale < second.Width * 2, "viewport excludes hidden content");
    var reduced = Visible(first, 0, 0, 1000, 1000, 0.5, 1);
    PageFrame smallFrame = CheckFrame(await dispatcher.RenderAsync(first, reduced, CancellationToken.None), reduced);
    Assert(smallFrame.PixelWidth < firstFrame.PixelWidth && smallFrame.PixelHeight < firstFrame.PixelHeight,
        "zoom out renders a smaller page");

    var capped = Visible(second, 24.75, 101.25, 8000, 8000, 4, 3);
    Assert(capped.DeviceScale < 3, "visible region respects the raster budget");
    PageFrame largeFrame = CheckFrame(await dispatcher.RenderAsync(second, capped, CancellationToken.None), capped);
    Assert(largeFrame.Bgra.Length <= 16 * 1024 * 1024, "maximum byte length");
    Assert(PageViewport.Visible(second, second.Width * 4, 0, 512, 384, 4, 3) is null,
        "viewport outside the page skips rendering");

    await Reject(dispatcher.RenderAsync(second with { FontGeneration = second.FontGeneration + 1 },
        enlarged, CancellationToken.None), "stale font generation");
    await Reject(dispatcher.RenderAsync(second, enlarged with
    {
        Region = enlarged.Region with { Width = -1 },
    }, CancellationToken.None), "invalidArgument");

    string invalid = Path.GetTempFileName();
    try
    {
        await File.WriteAllTextAsync(invalid, "invalid HWP input");
        await Reject(dispatcher.OpenAsync(invalid, CancellationToken.None), "documentError");
    }
    finally
    {
        File.Delete(invalid);
    }

    PageFrame again = CheckFrame(await dispatcher.RenderAsync(first, overview, CancellationToken.None), overview);
    Assert(firstFrame.Bgra.AsSpan().SequenceEqual(again.Bgra), "rendering recovers after rejected requests");

    using var superseded = new CancellationTokenSource();
    Task<PageInfo> obsolete = dispatcher.OpenAsync(args[0], superseded.Token);
    superseded.Cancel();
    Task<PageInfo> newest = dispatcher.OpenAsync(args[1], CancellationToken.None);
    try
    {
        await obsolete;
    }
    catch (OperationCanceledException)
    {
    }

    PageInfo replacement = await newest;
    Assert(replacement.Target != first.Target, "replacement document has a different stamp");
    await Reject(dispatcher.RenderAsync(first, overview, CancellationToken.None), "stale or foreign document ID");
    var publicForm = Visible(replacement, 0, 0, 1000, 1000, 1, 1);
    PageFrame formFrame = CheckFrame(await dispatcher.RenderAsync(replacement, publicForm, CancellationToken.None),
        publicForm);
    Assert(!firstFrame.Bgra.AsSpan().SequenceEqual(formFrame.Bgra), "replacement renders its own pixels");

    await dispatcher.CloseDocumentAsync(CancellationToken.None);
    await Reject(dispatcher.RenderAsync(replacement, publicForm, CancellationToken.None), "no document is open");
    PageInfo reopened = await dispatcher.OpenAsync(args[0], CancellationToken.None);
    Assert(reopened.Target != first.Target && reopened.Target != replacement.Target, "reopen has a fresh target");
    PageFrame reopenFrame = CheckFrame(await dispatcher.RenderAsync(reopened, overview, CancellationToken.None),
        overview);
    Assert(firstFrame.Bgra.AsSpan().SequenceEqual(reopenFrame.Bgra), "reopen restores the same first page");
    Console.WriteLine($"HWP pages {first.Page + 1}/{second.Page + 1}; zoomed {zoomFrame.PixelWidth}x{zoomFrame.PixelHeight}; capped {largeFrame.PixelWidth}x{largeFrame.PixelHeight}");
    Console.WriteLine($"HWP first page BGRA SHA256 {Convert.ToHexString(SHA256.HashData(firstFrame.Bgra))}");
    Console.WriteLine($"HWPX first page BGRA SHA256 {Convert.ToHexString(SHA256.HashData(formFrame.Bgra))}");
}
finally
{
    await dispatcher.CloseAsync();
}

var shuttingDown = new NativeWorkerDispatcher();
using var shutdownTimeout = new CancellationTokenSource(TimeSpan.FromSeconds(30));
Task<PageInfo> opening = shuttingDown.OpenAsync(args[0], CancellationToken.None);
await shuttingDown.CloseAsync().WaitAsync(shutdownTimeout.Token);
try
{
    await opening;
}
catch (OperationCanceledException)
{
    Console.WriteLine("Pending open cancelled on shutdown.");
}

await EditorSmoke.RunAsync(args[1]);
await DirectEditSmoke.RunAsync();
await SearchSmoke.RunAsync(args[1]);
await SaveCopySmoke.RunAsync(args[1]);
await RecoverySmoke.RunAsync();
await LocalAgentSmoke.RunAsync();
if (testLivePdf)
    await LivePdfSmoke.RunAsync(expectExport: true, multipageDocument: args[0]);

static PageViewport Visible(PageInfo page, double x, double y, double width, double height, double zoom, double scale) =>
    PageViewport.Visible(page, x, y, width, height, zoom, scale) ?? throw new Exception("No visible page.");

static PageFrame CheckFrame(PageFrame frame, PageViewport viewport)
{
    Assert(frame.PixelWidth > 0 && frame.PixelHeight > 0, "frame geometry");
    Assert(frame.Bgra.Length == frame.PixelWidth * frame.PixelHeight * 4, "packed BGRA");
    Assert(frame.OriginX == Math.Floor(viewport.Region.X * viewport.Scale) &&
        frame.OriginY == Math.Floor(viewport.Region.Y * viewport.Scale) &&
        frame.DeviceScale == viewport.DeviceScale, "raster stamp");
    return frame;
}

static async Task Reject(Task request, string message)
{
    try
    {
        await request;
        throw new Exception("Expected rejection: " + message);
    }
    catch (InvalidOperationException error) when (error.Message.Contains(message, StringComparison.Ordinal))
    {
        Console.WriteLine("Rejected: " + error.Message);
    }
}

static void Assert(bool condition, string check)
{
    if (!condition)
        throw new Exception("Failed: " + check);
}
