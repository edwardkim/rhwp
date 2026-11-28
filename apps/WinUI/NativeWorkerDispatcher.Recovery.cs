using System.Text.Json;

namespace Rhwp.WinUI;

internal sealed record RecoveredDocument(PageInfo Page, NativeDocumentInfo Info);

internal sealed partial class NativeWorkerDispatcher
{
    public Task<RecoveredDocument> OpenRecoveryAsync(byte[] bytes, CancellationToken cancellation) =>
        Dispatch(token =>
        {
            CheckDirty(discardChanges: false);
            if (bytes.Length is 0 or > MaxDocumentBytes)
                throw new InvalidDataException("복구 문서의 크기가 허용 범위를 벗어났습니다.");

            ulong handle = EnsureWorker();
            using var submitted = Send(new { op = "workerOpenRecovery", worker = handle, replace = target }, bytes);
            JsonElement opened = Poll(handle, submitted.RootElement.GetProperty("requestId").GetUInt64(), token);
            JsonElement completed = GetCompleted(opened);
            var info = Read<NativeDocumentInfo>(completed.GetProperty("info"));
            if (!info.IsDirty)
                throw new InvalidDataException("복구 문서의 미저장 상태를 읽을 수 없습니다.");
            return new RecoveredDocument(InitialPage(handle, completed, token), info);
        }, cancellation);
}
