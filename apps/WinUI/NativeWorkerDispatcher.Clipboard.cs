using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Rhwp.WinUI;

internal sealed record ClipboardSelection([property: JsonRequired] string PlainText, string? Html);

internal sealed partial class NativeWorkerDispatcher
{
    private const int MaxNativeClipboardBytes = 1024 * 1024;

    public Task<ClipboardSelection> CopySelectionAsync(DocumentTarget expected, CancellationToken cancellation) =>
        Dispatch(token =>
        {
            JsonElement reply = Submit(EnsureWorker(), new { op = "copySelection", target = expected }, token);
            if (reply.GetProperty("kind").GetString() != "clipboard" ||
                reply.GetProperty("outcome").ValueKind != JsonValueKind.Null)
                throw new InvalidDataException("Native 복사 응답이 올바르지 않습니다.");

            ClipboardSelection copied = Read<ClipboardSelection>(reply.GetProperty("payload"));
            if (copied.PlainText is null || Encoding.UTF8.GetByteCount(copied.PlainText) +
                (long)Encoding.UTF8.GetByteCount(copied.Html ?? "") > MaxNativeClipboardBytes)
                throw new InvalidDataException("Native 클립보드 데이터의 크기 또는 형식이 올바르지 않습니다.");

            return copied;
        }, cancellation);

    // The host publishes the copied content first. Delete the captured range,
    // since selection changes can leave the document revision unchanged.
    public Task<EditResult> CutSelectionAsync(DocumentTarget expected, TextSelection copiedSelection,
        CancellationToken cancellation) =>
        ApplyEdit(expected, new { op = "replaceRange", target = expected, selection = copiedSelection, text = "" },
            cancellation);

    public Task<EditResult> PasteTextAsync(DocumentTarget expected, string text, CancellationToken cancellation)
    {
        ValidatePasteSize(text);
        return ApplyEdit(expected, new { op = "pasteText", target = expected, text }, cancellation);
    }

    public Task<EditResult> PasteHtmlAsync(DocumentTarget expected, string html, CancellationToken cancellation)
    {
        ValidatePasteSize(html);
        return ApplyEdit(expected, new { op = "pasteHtml", target = expected, html }, cancellation);
    }

    private static void ValidatePasteSize(string content)
    {
        if (Encoding.UTF8.GetByteCount(content) > MaxNativeClipboardBytes)
            throw new ArgumentException("클립보드 내용은 1 MiB까지 붙여넣을 수 있습니다.", nameof(content));
    }
}
