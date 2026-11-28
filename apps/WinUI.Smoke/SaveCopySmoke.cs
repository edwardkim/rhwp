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
                await CheckOriginalSaveAsync(copy, format, selection, body, token);
            }
            await CheckDisplacedBackupAsync(directory.FullName, token);

            PreparedDocumentSave obsolete = await worker.PrepareSaveAsync(edit.Target, DocumentFormat.Hwpx, token);
            PageInfo latest = await worker.OpenAsync(replacement, token);
            await RejectAcknowledgement(worker, obsolete, "stale or foreign document ID");
            Assert(!(await worker.GetDocumentInfoAsync(latest.Target, token)).IsDirty,
                "old acknowledgement cannot change replacement state");
            Console.WriteLine("Saved and reopened HWP/HWPX copies and originals. External conflicts retained edited and on-disk bytes.");
        }
        finally
        {
            await worker.CloseAsync();
            directory.Delete(recursive: true);
        }
    }

    private static Task<PublishedDocumentCopy?> Publish(NativeWorkerDispatcher worker,
        PreparedDocumentSave prepared, string path, CancellationToken token, DocumentFileFingerprint? expected = null) =>
        DocumentSaveCopy.PublishAsync(worker, prepared, path,
            _ => throw new InvalidOperationException("Unexpected content loss on blank-document edit."),
            token, expected);

    private static async Task CheckOriginalSaveAsync(string path, DocumentFormat format, TextSelection selection,
        string previousText, CancellationToken token)
    {
        var editor = new NativeWorkerDispatcher();
        try
        {
            byte[] original = await File.ReadAllBytesAsync(path, token);
            OpenedDocument opened = await editor.OpenWithLocationAsync(path, token);
            Assert(opened.Location.Path == Path.GetFullPath(path) && opened.Location.Format == format &&
                opened.Location.Fingerprint == DocumentFileFingerprint.FromBytes(original),
                "tracked open fingerprints the bytes handed to Native");
            NativeDocumentInfo editing = await editor.BeginEditAsync(opened.Page.Target, selection, token);
            EditResult change = await editor.ReplaceAsync(editing.Target, "바뀐 ", token);

            byte[] foreign = original.ToArray();
            foreign[^1] ^= 1;
            await File.WriteAllBytesAsync(path, foreign, token);
            PreparedDocumentSave conflicting = await editor.PrepareSaveAsync(change.Target, format, token);
            try
            {
                await Publish(editor, conflicting, path, token, opened.Location.Fingerprint);
                throw new InvalidOperationException("Expected external same-size modification conflict.");
            }
            catch (DocumentFileConflictException)
            {
                Assert((await File.ReadAllBytesAsync(path, token)).SequenceEqual(foreign),
                    "stale fingerprint does not overwrite external bytes");
            }
            await RejectAcknowledgement(editor, conflicting, "save token");
            Assert((await editor.GetDocumentInfoAsync(change.Target, token)).IsDirty,
                "external modification keeps the live document dirty");

            await File.WriteAllBytesAsync(path, original, token);
            PreparedDocumentSave prepared = await editor.PrepareSaveAsync(change.Target, format, token);
            PublishedDocumentCopy saved = await Publish(editor, prepared, path, token, opened.Location.Fingerprint)
                ?? throw new InvalidOperationException("Expected a verified replacement.");
            Assert(!saved.Status.Dirty && saved.Status.SavedRevision == change.Target.Revision,
                "a successful replacement acknowledges the saved revision");
            Assert(await DocumentFileStore.ReadFingerprintAsync(path, token) == saved.File.Fingerprint,
                "replacement records the new fingerprint for the next save");
            await CheckReopenedAsync(path, selection, "바뀐 " + previousText, token);
        }
        finally
        {
            await editor.CloseAsync();
        }
    }

    private static async Task CheckDisplacedBackupAsync(string directory, CancellationToken token)
    {
        // Reproduce the state immediately after a replace displaced a different
        // file from the one verified just before the Windows path operation.
        string path = Path.Combine(directory, "교체 검사.hwpx");
        string backup = Path.Combine(directory, ".교체 직전.bak");
        byte[] saved = [4, 5, 6];
        byte[] foreign = [4, 5, 7];
        await File.WriteAllBytesAsync(path, saved, token);
        await File.WriteAllBytesAsync(backup, foreign, token);
        try
        {
            await DocumentFileStore.VerifyPublicationAsync(path, DocumentFileFingerprint.FromBytes(saved),
                backup, DocumentFileFingerprint.FromBytes([4, 5, 8]));
            throw new InvalidOperationException("Expected displaced-backup conflict.");
        }
        catch (DocumentFilePublicationException error) when (error.PublicationReturnedSuccess &&
            error.PossibleBackupPath == backup && error.InnerException is DocumentFileConflictException)
        {
            Assert((await File.ReadAllBytesAsync(path, token)).SequenceEqual(saved),
                "saved bytes survive a displaced-backup conflict");
            Assert((await File.ReadAllBytesAsync(backup, token)).SequenceEqual(foreign),
                "late external bytes are retained in the displaced backup");
        }
    }

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
