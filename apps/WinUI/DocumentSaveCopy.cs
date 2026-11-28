namespace Rhwp.WinUI;

internal sealed record PublishedDocumentCopy(DocumentFileReceipt File, DocumentSaveStatus Status);

internal static class DocumentSaveCopy
{
    public static async Task<PublishedDocumentCopy?> PublishAsync(NativeWorkerDispatcher worker,
        PreparedDocumentSave prepared, string destination, Func<SaveLossReport, Task<bool>> approveLoss,
        CancellationToken cancellation)
    {
        bool acknowledged = false;
        Exception? error = null;
        try
        {
            cancellation.ThrowIfCancellationRequested();
            if (prepared.Report.Count != 0 && !await approveLoss(prepared.Report))
                return null;

            DocumentFileReceipt receipt = await DocumentFileStore.PublishAsync(destination, prepared.Bytes,
                expectedDestination: null, cancellationToken: cancellation);
            DocumentSaveStatus status;
            try
            {
                // Publication passed its verification; settle the native token
                // even if cancellation has been requested in the meantime.
                status = await worker.AcknowledgeSaveAsync(prepared);
                acknowledged = true;
            }
            catch (Exception acknowledgementError)
            {
                throw new IOException($"파일을 저장했지만 문서의 저장 상태를 갱신하지 못했습니다. {receipt.Path}",
                    acknowledgementError);
            }

            return new PublishedDocumentCopy(receipt, status);
        }
        catch (Exception failure)
        {
            error = failure;
            throw;
        }
        finally
        {
            if (!acknowledged)
            {
                try
                {
                    await worker.CancelSaveAsync(prepared);
                }
                catch (Exception cleanupError) when (error is not null)
                {
                    error.Data["NativeSaveCleanup"] = cleanupError;
                }
            }
        }
    }
}
