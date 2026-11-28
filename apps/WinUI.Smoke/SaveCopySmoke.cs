using Rhwp.WinUI;

internal static class SaveCopySmoke
{
    public static async Task RunAsync(string replacement)
    {
        DirectoryInfo directory = Directory.CreateTempSubdirectory("rhwp-winui-save-");
        var worker = new NativeWorkerDispatcher();
        CancellationToken token = CancellationToken.None;
        const string body = "저장한😀한글";
        var start = new TextPosition(0, 0, 0, []);
        var selection = new TextSelection(start, start);
        try
        {
            PageInfo blank = await worker.NewAsync(token);
            NativeDocumentInfo editing = await worker.BeginEditAsync(blank.Target, selection, token);
            EditResult edit = await worker.ReplaceAsync(editing.Target, body, token);
            Assert((await worker.GetDocumentInfoAsync(edit.Target, token)).IsDirty, "edit dirties the document");

            string existing = Path.Combine(directory.FullName, "기존.hwpx");
            await File.WriteAllTextAsync(existing, "keep original", token);
            PreparedDocumentSave conflict = await worker.PrepareSaveAsync(edit.Target, DocumentFormat.Hwpx, token);
            try
            {
                await Publish(worker, conflict, existing, token);
                throw new Exception("Expected an existing-destination error.");
            }
            catch (DocumentFileConflictException)
            {
                Assert(await File.ReadAllTextAsync(existing, token) == "keep original", "existing file preserved");
            }
            await RejectAcknowledgement(worker, conflict, "save token");

            string cancelledPath = Path.Combine(directory.FullName, "취소.hwpx");
            PreparedDocumentSave cancelled = await worker.PrepareSaveAsync(edit.Target, DocumentFormat.Hwpx, token);
            using var cancellation = new CancellationTokenSource();
            cancellation.Cancel();
            try
            {
                await Publish(worker, cancelled, cancelledPath, cancellation.Token);
                throw new Exception("Expected a cancelled write.");
            }
            catch (OperationCanceledException)
            {
                Assert(!File.Exists(cancelledPath), "cancelled file never published");
            }
            await RejectAcknowledgement(worker, cancelled, "save token");
            string declinedPath = Path.Combine(directory.FullName, "손실 승인 취소.hwpx");
            PreparedDocumentSave declined = await worker.PrepareSaveAsync(edit.Target, DocumentFormat.Hwpx, token);
            // The blank document serializes losslessly. Exercise the report
            // boundary with one reported item before attempting publication.
            SaveLoss declared = new("controlOmitted", "control", "body/paragraph[0]", "unsupportedByOutputFormat", null);
            PreparedDocumentSave decision = declined with
            {
                Report = declined.Report with { Count = 1, Losses = [declared] },
            };
            bool offeredLoss = false;
            var rejected = await DocumentSaveCopy.PublishAsync(worker, decision, declinedPath,
                report =>
                {
                    offeredLoss = report.Count == 1 && report.Losses[0] == declared;
                    return Task.FromResult(false);
                }, token);
            Assert(offeredLoss && rejected is null && !File.Exists(declinedPath), "loss approval precedes writing");
            await RejectAcknowledgement(worker, declined, "save token");

            Assert((await worker.GetDocumentInfoAsync(edit.Target, token)).IsDirty, "failures keep dirty revision");

            foreach (DocumentFormat format in Enum.GetValues<DocumentFormat>())
            {
                string extension = format.ToString().ToLowerInvariant();
                string copy = Path.Combine(directory.FullName, $"한글 사본.{extension}");
                PreparedDocumentSave prepared = await worker.PrepareSaveAsync(edit.Target, format, token);
                Assert(prepared.Report.Count == 0 &&
                    prepared.Report.OutputFormat == extension, "explicit content loss report");
                PublishedDocumentCopy saved = await Publish(worker, prepared, copy, token)
                    ?? throw new Exception("Expected a published copy.");
                Assert(!saved.Status.Dirty && saved.Status.SavedRevision == edit.Target.Revision,
                    "verified publication is acknowledged at the prepared revision");
                Assert(await DocumentFileStore.ReadFingerprintAsync(copy, token) == saved.File.Fingerprint,
                    "published bytes match verified fingerprint");
                await RejectAcknowledgement(worker, prepared, "save token");
                await CheckReopenedAsync(copy, selection, body, token);
            }

            PreparedDocumentSave obsolete = await worker.PrepareSaveAsync(edit.Target, DocumentFormat.Hwpx, token);
            PageInfo latest = await worker.OpenAsync(replacement, token);
            await RejectAcknowledgement(worker, obsolete, "stale or foreign document ID");
            Assert(!(await worker.GetDocumentInfoAsync(latest.Target, token)).IsDirty,
                "old acknowledgement cannot change replacement state");
            Console.WriteLine("Save copies: verified HWP/HWPX publication and reopen; conflict/cancel/old token rejected.");
        }
        finally
        {
            await worker.CloseAsync();
            directory.Delete(recursive: true);
        }
    }

    private static Task<PublishedDocumentCopy?> Publish(NativeWorkerDispatcher worker,
        PreparedDocumentSave prepared, string path, CancellationToken token) =>
        DocumentSaveCopy.PublishAsync(worker, prepared, path,
            _ => throw new InvalidOperationException("Unexpected content loss on blank-document edit."), token);

    private static async Task CheckReopenedAsync(string path, TextSelection selection, string expected,
        CancellationToken token)
    {
        var reader = new NativeWorkerDispatcher();
        try
        {
            PageInfo page = await reader.OpenAsync(path, token);
            NativeDocumentInfo info = await reader.BeginEditAsync(page.Target, selection, token);
            ImeEditContext result = await reader.ReadImeEditContextAsync(info.Target, token);
            Assert(result.Context.Text == expected && !info.IsDirty, "reopened content and clean state");
        }
        finally
        {
            await reader.CloseAsync();
        }
    }

    private static async Task RejectAcknowledgement(NativeWorkerDispatcher worker,
        PreparedDocumentSave prepared, string message)
    {
        try
        {
            await worker.AcknowledgeSaveAsync(prepared);
            throw new Exception("Expected token rejection.");
        }
        catch (InvalidOperationException error) when (error.Message.Contains(message, StringComparison.Ordinal))
        {
        }
    }

    private static void Assert(bool condition, string check)
    {
        if (!condition)
            throw new InvalidOperationException("Native save copy: " + check);
    }
}
