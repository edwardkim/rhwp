using System.Security.Cryptography;
using System.Text.Json;
using Rhwp.WinUI;

internal static class LivePdfSmoke
{
    public static async Task RunAsync(bool expectExport, string? multipageDocument = null)
    {
        var page = new PageInfo(new DocumentTarget(7, 3), 8, 1, 2, 100, 100);
        CheckValidators(page);
        var worker = new NativeWorkerDispatcher();
        try
        {
            LivePdfLimits? limits = await worker.GetLivePdfLimitsAsync(CancellationToken.None);
            if (!expectExport)
            {
                Assert(limits is null, "native library must advertise PDF as unavailable");
                try
                {
                    await worker.ExportPagePdfAsync(page, 1024, CancellationToken.None);
                    throw new Exception("Unavailable PDF export was accepted.");
                }
                catch (NotSupportedException)
                {
                }

                Console.WriteLine("Live PDF: capability disabled and invalid receipt/bytes rejected.");
                return;
            }

            Assert(limits is not null && multipageDocument is not null, "P146 Skia library required");
            await CheckNativeExport(worker, limits!, multipageDocument!);
        }
        finally
        {
            await worker.CloseAsync();
        }
    }

    private static void CheckValidators(PageInfo page)
    {
        using var missing = JsonDocument.Parse("{}");
        using var disabled = JsonDocument.Parse("""{"livePdfExport":false,"maxLivePdfPages":32,"maxLivePdfBytes":10000}""");
        using var invalid = JsonDocument.Parse("""{"livePdfExport":true,"maxLivePdfPages":32,"maxLivePdfBytes":"10000"}""");
        using var valid = JsonDocument.Parse("""{"livePdfExport":true,"maxLivePdfPages":32,"maxLivePdfBytes":10000}""");
        Assert(LivePdfContract.Limits(missing.RootElement) is null &&
            LivePdfContract.Limits(disabled.RootElement) is null &&
            LivePdfContract.Limits(invalid.RootElement) is null &&
            LivePdfContract.Limits(valid.RootElement)?.MaxOutputBytes == 10000,
            "PDF availability and negotiated budget");

        JsonElement receipt = Receipt(7, 3, 8, 1, 1, 31);
        Assert(LivePdfContract.ValidateReceipt(receipt, page, 1000) == 31, "valid response metadata");
        foreach (JsonElement invalidReceipt in new[]
        {
            Receipt(6, 3, 8, 1, 1, 31), Receipt(7, 4, 8, 1, 1, 31),
            Receipt(7, 3, 9, 1, 1, 31), Receipt(7, 3, 8, 0, 1, 31),
            Receipt(7, 3, 8, 1, 2, 31), Receipt(7, 3, 8, 1, 1, 0),
            Receipt(7, 3, 8, 1, 1, 1001),
        })
        {
            RejectInvalid(() => LivePdfContract.ValidateReceipt(invalidReceipt, page, 1000));
        }

        LivePdfContract.ValidateBytes("%PDF-1.4\nxref\nstartxref\n9\n%%EOF\r\n"u8);
        RejectInvalid(() => LivePdfContract.ValidateBytes("bogus"u8));
        RejectInvalid(() => LivePdfContract.ValidateBytes("%PDF-1.4\n%%EOF\n"u8));
        RejectInvalid(() => LivePdfContract.ValidateBytes("%PDF-1.4\nxref\nstartxref\n999\n%%EOF\n"u8));
        RejectInvalid(() => LivePdfContract.ValidateBytes("%PDF-1.4\nxref\nstartxref\n9\n"u8));
    }

    private static JsonElement Receipt(ulong document, ulong revision, ulong font, uint firstPage,
        uint pageCount, int byteLength) => JsonSerializer.SerializeToElement(new
    {
        kind = "exportedPdf", target = new { documentId = document, revision },
        fontGeneration = font, firstPage, pageCount, byteLength,
    });

    private static async Task CheckNativeExport(NativeWorkerDispatcher worker, LivePdfLimits limits,
        string multipageDocument)
    {
        CancellationToken token = CancellationToken.None;
        PageInfo blank = await worker.NewAsync(token);
        var start = new TextPosition(0, 0, 0, []);
        await worker.BeginEditAsync(blank.Target, new TextSelection(start, start), token);
        const string unsaved = "Unsaved Worker PDF 한글";
        EditResult edit = await worker.ReplaceAsync(blank.Target, unsaved, token);
        NativeDocumentInfo before = await worker.GetDocumentInfoAsync(edit.Target, token);
        PageInfo current = await worker.GetPageAsync(blank with { Target = edit.Target }, 0, token);
        PreparedDocumentSave pending = await worker.PrepareSaveAsync(edit.Target, DocumentFormat.Hwpx, token);
        Assert(before.IsDirty && before.CanUndo && !before.CanRedo, "editable unsaved revision");
        await RejectNative(worker.ExportPagePdfAsync(blank, limits.MaxOutputBytes, token), "staleRevision");
        await RejectNative(worker.ExportPagePdfAsync(current with { FontGeneration = current.FontGeneration + 1 },
            limits.MaxOutputBytes, token), "stale font generation");
        await RejectNative(worker.ExportPagePdfAsync(current, 1, token), "limitExceeded");

        using var cancelled = new CancellationTokenSource();
        cancelled.Cancel();
        try
        {
            await worker.ExportPagePdfAsync(current, limits.MaxOutputBytes, cancelled.Token);
            throw new Exception("Cancelled PDF export was accepted.");
        }
        catch (OperationCanceledException)
        {
        }

        byte[] pdf = await worker.ExportPagePdfAsync(current, limits.MaxOutputBytes, token);
        await CheckPublishedAsync(pdf, token);
        NativeDocumentInfo after = await worker.GetDocumentInfoAsync(edit.Target, token);
        Assert(after.Target == before.Target && after.FontGeneration == before.FontGeneration &&
            after.IsDirty == before.IsDirty && after.CanUndo == before.CanUndo && after.CanRedo == before.CanRedo,
            "export preserves revision, font, dirty state, and history");
        Assert((await worker.ReadImeEditContextAsync(edit.Target, token)).Context.Text == unsaved,
            "unsaved text remains live");
        DocumentSaveStatus cancelledSave = await worker.CancelSaveAsync(pending);
        Assert(cancelledSave.Dirty && cancelledSave.Revision == edit.Target.Revision,
            "export leaves unrelated pending save token usable");
        EditResult undo = await worker.UndoAsync(edit.Target, token);
        Assert((await worker.ReadImeEditContextAsync(undo.Target, token)).Context.Text.Length == 0,
            "undo still targets the actual edit");

        await worker.CloseDocumentAsync(token, discardChanges: true);
        PageInfo first = await worker.OpenAsync(multipageDocument, token);
        PageInfo second = await worker.GetPageAsync(first, 1, token);
        byte[] next = await worker.ExportPagePdfAsync(second, limits.MaxOutputBytes, token);
        Assert(next.Length > 0, "export the current second page");
        Console.WriteLine($"Live PDF: current second page; unpublished edit export SHA256 {Convert.ToHexString(SHA256.HashData(pdf))}");
    }

    private static async Task CheckPublishedAsync(byte[] pdf, CancellationToken token)
    {
        DirectoryInfo directory = Directory.CreateTempSubdirectory("rhwp-live-pdf-");
        try
        {
            string destination = Path.Combine(directory.FullName, "현재 쪽.pdf");
            DocumentFileReceipt receipt = await DocumentFileStore.PublishAsync(destination, pdf,
                expectedDestination: null, cancellationToken: token);
            Assert(receipt.Fingerprint == DocumentFileFingerprint.FromBytes(pdf) &&
                (await File.ReadAllBytesAsync(destination, token)).AsSpan().SequenceEqual(pdf),
                "verified exact new PDF bytes");
            try
            {
                await DocumentFileStore.PublishAsync(destination, pdf,
                    expectedDestination: null, cancellationToken: token);
                throw new Exception("Existing PDF path was overwritten.");
            }
            catch (DocumentFileConflictException)
            {
                Assert(await DocumentFileStore.ReadFingerprintAsync(destination, token) == receipt.Fingerprint,
                    "existing PDF remains intact");
            }
        }
        finally
        {
            directory.Delete(recursive: true);
        }
    }

    private static void RejectInvalid(Action action)
    {
        try
        {
            action();
            throw new Exception("Invalid PDF was accepted.");
        }
        catch (InvalidDataException)
        {
        }
    }

    private static async Task RejectNative(Task action, string failure)
    {
        try
        {
            await action;
            throw new Exception("Expected PDF rejection: " + failure);
        }
        catch (InvalidOperationException error) when (error.Message.Contains(failure, StringComparison.Ordinal))
        {
        }
    }

    private static void Assert(bool condition, string check)
    {
        if (!condition)
            throw new InvalidOperationException("Live PDF: " + check);
    }
}
