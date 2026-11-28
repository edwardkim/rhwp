using System.Text.Json;

namespace Rhwp.WinUI;

internal enum DocumentFormat { Hwp, Hwpx }
internal sealed record DocumentFileLocation(string Path, DocumentFileFingerprint Fingerprint, DocumentFormat? Format)
{
    private static ReadOnlySpan<byte> HwpContainerMagic => [0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1];

    public static DocumentFormat? FormatForPath(string path) =>
        System.IO.Path.GetExtension(path).ToLowerInvariant() switch
        {
            ".hwp" => DocumentFormat.Hwp,
            ".hwpx" => DocumentFormat.Hwpx,
            _ => null,
        };

    public static DocumentFileLocation FromOpenedBytes(string path, ReadOnlySpan<byte> contents)
    {
        DocumentFormat? format = FormatForPath(path) switch
        {
            DocumentFormat.Hwp when contents.StartsWith(HwpContainerMagic) => DocumentFormat.Hwp,
            DocumentFormat.Hwpx when contents.StartsWith("PK\x03\x04"u8) => DocumentFormat.Hwpx,
            _ => null,
        };
        return new DocumentFileLocation(path, DocumentFileFingerprint.FromBytes(contents), format);
    }
}

internal sealed record OpenedDocument(PageInfo Page, DocumentFileLocation Location);
internal sealed record SaveLoss(string Code, string Subject, string Path, string Reason, uint? ResourceId);
internal sealed record SaveLossReport(uint SchemaVersion, string OutputFormat, int Count, SaveLoss[] Losses);
internal sealed record PreparedDocumentSave(DocumentTarget Target, ulong Token, byte[] Bytes, SaveLossReport Report);
internal sealed record DocumentSaveStatus(ulong Revision, ulong? SavedRevision, bool Dirty);

internal sealed partial class NativeWorkerDispatcher
{
    public Task<PreparedDocumentSave> PrepareSaveAsync(DocumentTarget expected, DocumentFormat format,
        CancellationToken cancellation) =>
        // The operation must see its own accepted result to clean up a token
        // produced after cancellation was requested.
        Dispatch(_ => PrepareSave(expected, format, cancellation), CancellationToken.None);

    public Task<DocumentSaveStatus> AcknowledgeSaveAsync(PreparedDocumentSave prepared) =>
        CompleteSaveAsync(prepared, "acknowledgeSave");

    public Task<DocumentSaveStatus> CancelSaveAsync(PreparedDocumentSave prepared) =>
        CompleteSaveAsync(prepared, "cancelSave");

    private PreparedDocumentSave PrepareSave(DocumentTarget expected, DocumentFormat format,
        CancellationToken cancellation)
    {
        CheckClosing();
        cancellation.ThrowIfCancellationRequested();
        ulong handle = EnsureWorker();
        string formatName = format switch
        {
            DocumentFormat.Hwp => "hwp",
            DocumentFormat.Hwpx => "hwpx",
            _ => throw new ArgumentOutOfRangeException(nameof(format)),
        };
        using var submitted = Send(new
        {
            op = "workerSubmit",
            worker = handle,
            command = new { op = "prepareSave", target = expected, format = formatName },
        });
        JsonElement result = Poll(handle, submitted.RootElement.GetProperty("requestId").GetUInt64(), cancellation);
        if (result.TryGetProperty("saveCleanup", out JsonElement cleanup))
        {
            RequireSuccess(cleanup);
            GetCompleted(Poll(handle, cleanup.GetProperty("requestId").GetUInt64(), CancellationToken.None));
        }
        GetCompleted(result);

        ulong buffer = result.GetProperty("buffer").GetUInt64();
        ulong token = result.GetProperty("token").GetUInt64();
        try
        {
            try
            {
                cancellation.ThrowIfCancellationRequested();
                int length = result.GetProperty("byteLength").GetInt32();
                if (result.GetProperty("documentId").GetUInt64() != expected.DocumentId ||
                    result.GetProperty("revision").GetUInt64() != expected.Revision ||
                    length <= 0 || length > MaxDocumentBytes)
                    throw new InvalidDataException("저장할 문서의 버전 또는 바이트 수가 올바르지 않습니다.");

                SaveLossReport report = Read<SaveLossReport>(result.GetProperty("report"));
                if (report.SchemaVersion != 1 || report.OutputFormat != formatName || report.Count != report.Losses.Length)
                    throw new InvalidDataException("저장 중 손실 보고를 읽을 수 없습니다.");

                byte[] bytes = CopyExactBuffer(buffer, length);
                cancellation.ThrowIfCancellationRequested();
                return new PreparedDocumentSave(expected, token, bytes, report);
            }
            finally
            {
                ReleaseBuffer(buffer);
            }
        }
        catch (Exception error)
        {
            try
            {
                CompleteSave(handle, "cancelSave", expected.DocumentId, token);
            }
            catch (Exception cleanupError)
            {
                error.Data["NativeSaveCleanup"] = cleanupError;
            }

            throw;
        }
    }

    private Task<DocumentSaveStatus> CompleteSaveAsync(PreparedDocumentSave prepared, string command) =>
        Dispatch(_ => CompleteSave(EnsureWorker(), command, prepared.Target.DocumentId, prepared.Token),
            CancellationToken.None);

    private DocumentSaveStatus CompleteSave(ulong handle, string command, ulong documentId, ulong token)
    {
        JsonElement result = Submit(handle, new { op = command, documentId, token }, CancellationToken.None);
        if (result.GetProperty("documentId").GetUInt64() != documentId)
            throw new InvalidDataException("저장 응답에 다른 문서가 포함되었습니다.");

        DocumentSaveStatus status = Read<DocumentSaveStatus>(result.GetProperty("status"));
        target = new DocumentTarget(documentId, status.Revision);
        if (command == "acknowledgeSave" && status.SavedRevision is ulong saved &&
            saved >= committedRevision)
            dirty = false;

        return status with { Dirty = dirty };
    }
}
