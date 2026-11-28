using System.Collections.Concurrent;
using System.Text.Json;

namespace Rhwp.WinUI;

// The ABI's worker handle and transferred buffers belong to the OS thread
// that created the handle. Only managed page pixels leave the dispatcher.
internal sealed partial class NativeWorkerDispatcher
{
    private const int MaxDocumentBytes = 128 * 1024 * 1024;
    private const int MaxFrameBytes = PageViewport.MaxRasterDimension * PageViewport.MaxRasterDimension * 4;
    private const int PollIntervalMs = 10;
    private static readonly JsonSerializerOptions WireJson = new(JsonSerializerDefaults.Web);

    private readonly BlockingCollection<Action> requests = new(boundedCapacity: 4);
    private readonly object gate = new();
    private readonly TaskCompletionSource stopped = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private volatile bool closing;

    // These fields are read and written by Run's owning OS thread.
    private ulong? worker;
    private DocumentTarget? target;
    private bool dirty;

    public NativeWorkerDispatcher()
    {
        new Thread(Run) { IsBackground = true, Name = "RHWP ABI dispatcher" }.Start();
    }

    public Task<PageInfo> OpenAsync(string path, CancellationToken cancellation, bool discardChanges = false) =>
        Dispatch(token => OpenFile(path, discardChanges, token).Page, cancellation);

    public Task<OpenedDocument> OpenWithLocationAsync(string path, CancellationToken cancellation,
        bool discardChanges = false) =>
        Dispatch(token => OpenFile(path, discardChanges, token), cancellation);

    public Task<PageInfo> GetPageAsync(PageInfo info, uint page, CancellationToken cancellation) =>
        Dispatch(token => ReadPage(EnsureWorker(), info.Target, info.FontGeneration,
            page, info.PageCount, token), cancellation);

    public Task<PageFrame> RenderAsync(PageInfo info, PageViewport viewport, CancellationToken cancellation) =>
        Dispatch(token => Render(EnsureWorker(), info, viewport, token), cancellation);

    public Task CloseDocumentAsync(CancellationToken cancellation, bool discardChanges = false) =>
        Dispatch(token =>
        {
            CheckDirty(discardChanges);
            if (worker is ulong handle && target is DocumentTarget current)
            {
                Submit(handle, new { op = "closeDocument", target = current }, token);
                target = null;
                dirty = false;
            }
            return true;
        }, cancellation);

    private Task<T> Dispatch<T>(Func<CancellationToken, T> operation, CancellationToken cancellation)
    {
        lock (gate)
        {
            if (closing)
                throw new InvalidOperationException("The native dispatcher is closing.");

            var completion = new TaskCompletionSource<T>(TaskCreationOptions.RunContinuationsAsynchronously);
            if (!requests.TryAdd(() =>
            {
                try
                {
                    CheckClosing();
                    cancellation.ThrowIfCancellationRequested();
                    T result = operation(cancellation);
                    CheckClosing();
                    cancellation.ThrowIfCancellationRequested();
                    completion.SetResult(result);
                }
                catch (OperationCanceledException)
                {
                    completion.SetCanceled();
                }
                catch (Exception error)
                {
                    completion.SetException(error);
                }
            }))
            {
                throw new InvalidOperationException("이전 파일 요청을 처리하고 있습니다.");
            }

            return completion.Task;
        }
    }

    public Task CloseAsync()
    {
        lock (gate)
        {
            if (!closing)
            {
                closing = true;
                requests.CompleteAdding();
            }
        }

        return stopped.Task;
    }

    private void Run()
    {
        try
        {
            foreach (Action request in requests.GetConsumingEnumerable())
                request();

            if (worker is ulong handle)
                CloseWorker(handle);
            stopped.SetResult();
        }
        catch (Exception error)
        {
            stopped.SetException(error);
        }
        finally
        {
            requests.Dispose();
        }
    }

    private ulong EnsureWorker() => worker ??= CreateWorker();

    private static ulong CreateWorker()
    {
        using var report = ReadSuccess(RhwpNative.SessionCapabilities());
        JsonElement capabilities = report.RootElement;
        if (capabilities.GetProperty("abiVersion").GetInt32() != 1 ||
            !capabilities.GetProperty("worker").GetBoolean() ||
            !capabilities.GetProperty("renderTiledRegion").GetBoolean())
        {
            throw new InvalidOperationException("Windows x64 native-skia DLL이 필요합니다.");
        }

        using var created = Send(new { op = "workerCreate", config = new { maxOutstanding = 1 } });
        return created.RootElement.GetProperty("worker").GetUInt64();
    }

    private OpenedDocument OpenFile(string path, bool discardChanges, CancellationToken cancellation)
    {
        CheckDirty(discardChanges);
        ulong handle = EnsureWorker();
        string fullPath = Path.GetFullPath(path);
        byte[] contents = ReadFile(fullPath);
        DocumentFileLocation location = DocumentFileLocation.FromOpenedBytes(fullPath, contents);
        CheckClosing();
        cancellation.ThrowIfCancellationRequested();
        using var submitted = Send(new { op = "workerOpen", worker = handle, replace = target }, contents);
        JsonElement opened = Poll(handle, submitted.RootElement.GetProperty("requestId").GetUInt64(), cancellation);
        return new OpenedDocument(InitialPage(handle, GetCompleted(opened), cancellation), location);
    }

    private PageInfo InitialPage(ulong handle, JsonElement reply, CancellationToken cancellation)
    {
        NativeDocumentInfo info = AdoptInfo(reply);
        if (info.PageCount == 0)
            throw new InvalidDataException("표시할 쪽이 없습니다.");

        return ReadPage(handle, info.Target, info.FontGeneration, 0, info.PageCount, cancellation);
    }

    private void CheckDirty(bool discardChanges)
    {
        if (dirty && !discardChanges)
            throw new InvalidOperationException("문서를 닫기 전에 변경 사항을 저장하거나 폐기해야 합니다.");
    }

    private NativeDocumentInfo AdoptInfo(JsonElement reply, DocumentTarget? expected = null)
    {
        NativeDocumentInfo info = Read<NativeDocumentInfo>(reply.GetProperty("info"));
        if (expected is not null && info.Target != expected)
            throw new InvalidOperationException("문서가 바뀌었습니다.");

        target = info.Target;
        dirty = info.IsDirty;
        return info;
    }

    private PageInfo ReadPage(ulong handle, DocumentTarget current, ulong fontGeneration, uint page,
        uint pageCount, CancellationToken cancellation)
    {
        if (page >= pageCount)
            throw new ArgumentOutOfRangeException(nameof(page));

        JsonElement sized = Submit(handle, new { op = "pageSize", target = current, page }, cancellation);
        JsonElement size = sized.GetProperty("size");
        double width = size.GetProperty("width").GetDouble();
        double height = size.GetProperty("height").GetDouble();
        if (!double.IsFinite(width) || !double.IsFinite(height) || width <= 0 || height <= 0)
            throw new InvalidDataException("쪽 크기가 올바르지 않습니다.");

        return new PageInfo(current, fontGeneration, page, pageCount, width, height);
    }

    private PageFrame Render(ulong handle, PageInfo info, PageViewport viewport, CancellationToken cancellation)
    {
        JsonElement frame = Submit(handle, new
        {
            op = "renderTiledRegion", target = info.Target, info.FontGeneration, info.Page,
            viewport.Region, viewport.Scale,
        }, cancellation);
        ulong buffer = frame.GetProperty("buffer").GetUInt64();
        try
        {
            CheckClosing();
            cancellation.ThrowIfCancellationRequested();
            if (frame.GetProperty("documentId").GetUInt64() != info.Target.DocumentId ||
                frame.GetProperty("revision").GetUInt64() != info.Target.Revision ||
                frame.GetProperty("fontGeneration").GetUInt64() != info.FontGeneration ||
                frame.GetProperty("pixelFormat").GetString() != "rgba8Premultiplied" ||
                frame.GetProperty("colorSpace").GetString() != "srgb")
            {
                throw new InvalidDataException("문서 또는 픽셀 형식이 렌더링 요청과 일치하지 않습니다.");
            }

            int originX = frame.GetProperty("originX").GetInt32();
            int originY = frame.GetProperty("originY").GetInt32();
            RenderRegion region = viewport.Region;
            double scale = viewport.Scale;
            int pixelWidth = frame.GetProperty("width").GetInt32();
            int pixelHeight = frame.GetProperty("height").GetInt32();
            int stride = frame.GetProperty("stride").GetInt32();
            int length = frame.GetProperty("byteLength").GetInt32();
            if (originX != Math.Floor(region.X * scale) || originY != Math.Floor(region.Y * scale) ||
                originX + (double)pixelWidth != Math.Ceiling((region.X + region.Width) * scale) ||
                originY + (double)pixelHeight != Math.Ceiling((region.Y + region.Height) * scale) ||
                pixelWidth is <= 0 or > PageViewport.MaxRasterDimension ||
                pixelHeight is <= 0 or > PageViewport.MaxRasterDimension ||
                stride < pixelWidth * 4L || length <= 0 || length > MaxFrameBytes ||
                stride * (long)pixelHeight != length)
            {
                throw new InvalidDataException("픽셀 버퍼 크기가 올바르지 않습니다.");
            }

            byte[] bytes = CopyExactBuffer(buffer, length);
            return new PageFrame(pixelWidth, pixelHeight, originX, originY, viewport.DeviceScale,
                ToBgra(bytes, pixelWidth, pixelHeight, stride));
        }
        finally
        {
            ReleaseBuffer(buffer);
        }
    }

    private static byte[] ReadFile(string path)
    {
        using var file = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.Read);
        long length = file.Length;
        if (length <= 0 || length > MaxDocumentBytes)
            throw new InvalidDataException("문서 크기는 1바이트부터 128MiB까지 지원합니다.");

        byte[] bytes = new byte[(int)length];
        file.ReadExactly(bytes);
        if (file.ReadByte() != -1)
            throw new IOException("파일을 읽는 중에 파일 크기가 바뀌었습니다.");

        return bytes;
    }

    private static byte[] ToBgra(byte[] rgba, int width, int height, int stride)
    {
        int rowBytes = width * 4;
        byte[] bgra = stride == rowBytes ? rgba : new byte[rowBytes * height];
        for (int row = 0; row < height; row++)
        {
            if (stride != rowBytes)
                Buffer.BlockCopy(rgba, row * stride, bgra, row * rowBytes, rowBytes);
            int end = (row + 1) * rowBytes;
            for (int i = row * rowBytes; i < end; i += 4)
                (bgra[i], bgra[i + 2]) = (bgra[i + 2], bgra[i]);
        }

        return bgra;
    }

    private JsonElement Submit(ulong handle, object command, CancellationToken cancellation)
    {
        CheckClosing();
        cancellation.ThrowIfCancellationRequested();
        using var submitted = Send(new { op = "workerSubmit", worker = handle, command });
        return GetCompleted(Poll(handle, submitted.RootElement.GetProperty("requestId").GetUInt64(), cancellation));
    }

    private static JsonElement GetCompleted(JsonElement completion)
    {
        RequireSuccess(completion);
        if (completion.GetProperty("kind").GetString() == "cancelled")
            throw new OperationCanceledException();
        return completion;
    }

    private JsonElement Poll(ulong handle, ulong requestId, CancellationToken cancellation)
    {
        bool cancelSent = false;
        while (true)
        {
            CheckClosing();
            // An accepted open may finish despite cancellation. Drain the
            // completion so the next open can replace the actual target.
            if (cancellation.IsCancellationRequested && !cancelSent)
            {
                using var cancelled = Send(new { op = "workerCancel", worker = handle, requestId });
                cancelSent = true;
            }

            using var polled = Send(new { op = "workerPoll", worker = handle });
            JsonElement completion = polled.RootElement.GetProperty("completion");
            if (completion.ValueKind == JsonValueKind.Null)
            {
                Thread.Sleep(PollIntervalMs);
                continue;
            }

            if (completion.GetProperty("requestId").GetUInt64() != requestId)
            {
                if (completion.TryGetProperty("buffer", out JsonElement buffer))
                    ReleaseBuffer(buffer.GetUInt64());
                throw new InvalidDataException("Unexpected native worker request ID.");
            }

            return completion.Clone();
        }
    }

    private void CheckClosing()
    {
        if (closing)
            throw new OperationCanceledException();
    }

    private static void CloseWorker(ulong worker)
    {
        using var closed = Send(new { op = "workerClose", worker });
        bool finished = closed.RootElement.GetProperty("isFinished").GetBoolean();
        while (!finished)
        {
            Thread.Sleep(PollIntervalMs);
            using var status = Send(new { op = "workerStatus", worker });
            finished = status.RootElement.GetProperty("isFinished").GetBoolean();
        }

        using var released = Send(new { op = "workerRelease", worker });
    }

    private static byte[] CopyExactBuffer(ulong buffer, int length)
    {
        byte[] bytes = new byte[length];
        using var copied = ReadSuccess(RhwpNative.SessionBufferCopy(buffer, bytes));
        if (copied.RootElement.GetProperty("byteLength").GetInt32() != length)
            throw new InvalidDataException("네이티브 버퍼가 잘렸습니다.");

        return bytes;
    }

    private static void ReleaseBuffer(ulong buffer)
    {
        using var released = ReadSuccess(RhwpNative.SessionBufferRelease(buffer));
    }

    private static JsonDocument Send(object request, byte[]? bytes = null) =>
        ReadSuccess(RhwpNative.HostRequest(JsonSerializer.Serialize(request, WireJson), bytes));

    private static JsonDocument ReadSuccess(string json)
    {
        JsonDocument result = JsonDocument.Parse(json);
        try
        {
            RequireSuccess(result.RootElement);
            return result;
        }
        catch
        {
            result.Dispose();
            throw;
        }
    }

    private static void RequireSuccess(JsonElement result)
    {
        if (!result.GetProperty("ok").GetBoolean())
            throw new InvalidOperationException($"{result.GetProperty("error").GetString()} ({result.GetProperty("code").GetString()})");
    }
}
