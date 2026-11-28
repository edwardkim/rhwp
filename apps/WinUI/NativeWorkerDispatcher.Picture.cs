using System.Text.Json;

namespace Rhwp.WinUI;

internal sealed record NativePictureLimits(int InputBytes, int MaxDimension);
internal sealed record NativePictureAddress(uint Section, uint Paragraph, uint Control);
internal sealed record NativePictureProperties(uint Width, uint Height, uint OriginalWidth, uint OriginalHeight,
    bool TreatAsChar, bool SizeProtect, bool Locked);
internal sealed record NativePictureObject(NativePictureAddress Address, CaretBounds Bounds,
    NativePictureProperties Properties, bool CanResize);

internal sealed partial class NativeWorkerDispatcher
{
    private static NativePictureLimits? ReadPictureLimits(JsonElement capabilities)
    {
        int maxInput = HasCapability(capabilities, "insertPicture")
            ? capabilities.GetProperty("maxPictureBytes").GetInt32() : 0;
        int maxDimension = HasCapability(capabilities, "pictureObjectProperties")
            ? capabilities.GetProperty("maxPictureObjectSize").GetInt32() : 0;
        return maxInput > 0 || maxDimension > 0 ? new NativePictureLimits(maxInput, maxDimension) : null;
    }

    public Task<EditResult> InsertPictureAsync(DocumentTarget expected, byte[] bytes, CancellationToken cancellation) =>
        Dispatch(token =>
        {
            ulong handle = EnsureWorker();
            using var submitted = Send(new { op = "workerInsertPicture", worker = handle, target = expected }, bytes);
            ulong request = submitted.RootElement.GetProperty("requestId").GetUInt64();
            JsonElement reply = GetCompleted(Poll(handle, request, token, interactive: true));
            if (reply.GetProperty("kind").GetString() != "edited")
                throw new InvalidDataException("Native 그림 삽입 응답이 올바르지 않습니다.");
            return AdoptEditOutcome(expected, reply);
        }, cancellation);

    public Task<NativePictureObject?> GetPictureAtAsync(PageInfo page, double x, double y, CancellationToken cancellation) =>
        QueryPictureAsync(page, new { op = "pictureObjectAt", target = page.Target, page.Page, x, y }, cancellation);

    public Task<NativePictureObject?> GetPictureObjectAsync(PageInfo page, NativePictureAddress address,
        CancellationToken cancellation) =>
        QueryPictureAsync(page, new { op = "pictureObject", target = page.Target, page.Page, address }, cancellation);

    private Task<NativePictureObject?> QueryPictureAsync(PageInfo page, object query, CancellationToken cancellation) =>
        Dispatch<NativePictureObject?>(token =>
        {
            JsonElement reply = Submit(EnsureWorker(), query, token, interactive: true);
            if (reply.GetProperty("kind").GetString() != "pictureObject" ||
                Read<DocumentTarget>(reply.GetProperty("target")) != page.Target ||
                reply.GetProperty("fontGeneration").GetUInt64() != page.FontGeneration ||
                reply.GetProperty("page").GetUInt32() != page.Page)
                throw new InvalidDataException("Native 그림의 문서 위치가 일치하지 않습니다.");

            JsonElement value = reply.GetProperty("picture");
            if (value.ValueKind == JsonValueKind.Null)
                return null;
            NativePictureObject picture = Read<NativePictureObject>(value);
            if (picture.Address is null || picture.Properties is null ||
                picture.Bounds is not { Width: > 0, Height: > 0 } bounds ||
                !double.IsFinite(bounds.X) || !double.IsFinite(bounds.Y) ||
                !double.IsFinite(bounds.Width) || !double.IsFinite(bounds.Height))
                throw new InvalidDataException("Native 그림 영역을 읽을 수 없습니다.");
            return picture;
        }, cancellation);

    public Task<EditResult> ResizePictureObjectAsync(DocumentTarget expected, NativePictureAddress picture,
        uint width, uint height, CancellationToken cancellation) =>
        Dispatch(token => AdoptEditOutcome(expected,
            SubmitEditCommand(expected, new { command = "resizePictureObject", picture, width, height }, token)), cancellation);
}
