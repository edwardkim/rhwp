using Rhwp.WinUI;

internal static class RecoverySmoke
{
    public static async Task RunAsync()
    {
        const string editedText = "저장하지 않은 가나다😀";
        CancellationToken token = CancellationToken.None;
        string directory = Directory.CreateTempSubdirectory("rhwp-recovery-").FullName;
        var store = new DocumentRecoveryStore(directory);
        var source = new NativeWorkerDispatcher();
        try
        {
            using (var live = await store.CreateAsync())
            {
                PageInfo page = await source.NewAsync(token);
                var start = new TextPosition(0, 0, 0, []);
                await source.BeginEditAsync(page.Target, new(start, start), token);
                EditResult edit = await source.ReplaceAsync(page.Target, editedText, token);
                PreparedDocumentSave prepared = await source.PrepareSaveAsync(edit.Target, DocumentFormat.Hwpx, token);
                try
                {
                    Assert(prepared.Report.Count == 0, "fixture has a lossless HWPX snapshot");
                    await live.SaveAsync(edit.Target.Revision, DocumentFormat.Hwpx, "원본.hwpx", prepared.Bytes);
                    Assert(await store.TryClaimLatestAsync() is null, "a second window cannot claim a live checkpoint");

                    try
                    {
                        await live.SaveAsync(edit.Target.Revision + 1, DocumentFormat.Hwpx, "원본.hwpx", []);
                        throw new InvalidOperationException("An empty snapshot replaced a verified checkpoint.");
                    }
                    catch (InvalidDataException)
                    {
                    }
                }
                finally
                {
                    DocumentSaveStatus status = await source.CancelSaveAsync(prepared);
                    Assert(status.Dirty && status.Revision == edit.Target.Revision,
                        "recovery never acknowledges the user's save");
                }
            }
            await source.CloseAsync();

            RecoveryCandidate candidate = await store.TryClaimLatestAsync()
                ?? throw new InvalidOperationException("A complete abandoned checkpoint was not found.");
            using (candidate.Lease)
            {
                Assert(candidate.Header.Revision == 1 && candidate.Header.Name == "원본.hwpx",
                    "the last verified checkpoint survives a failed update");
                byte[] bytes = await candidate.Lease.LoadAsync(candidate.Header);
                var recovered = new NativeWorkerDispatcher();
                try
                {
                    RecoveredDocument restored = await recovered.OpenRecoveryAsync(bytes, token);
                    Assert(restored.Info.IsDirty, "restored bytes open as an unsaved document");
                    var start = new TextPosition(0, 0, 0, []);
                    await recovered.BeginEditAsync(restored.Page.Target, new(start, start), token);
                    ImeEditContext context = await recovered.ReadImeEditContextAsync(restored.Page.Target, token);
                    Assert(context.Context.Text == editedText, "restored text matches the unacknowledged edit");
                }
                finally
                {
                    await recovered.CloseAsync();
                }

                string checkpoint = Directory.GetFiles(directory, "*.rhwp-recovery").Single();
                using (var file = new FileStream(checkpoint, FileMode.Open, FileAccess.ReadWrite))
                {
                    file.Position = file.Length - 1;
                    int last = file.ReadByte();
                    file.Position--;
                    file.WriteByte((byte)(last ^ 1));
                }
                try
                {
                    await candidate.Lease.LoadAsync(candidate.Header);
                    throw new InvalidOperationException("A corrupted recovery payload was accepted.");
                }
                catch (InvalidDataException)
                {
                }

                await candidate.Lease.DeleteAsync();
            }
            Assert(await store.TryClaimLatestAsync() is null, "confirmed cleanup removes the owned checkpoint");
            Console.WriteLine("Recovery: live lease, failed update, dirty Native restore, checksum and cleanup passed.");
        }
        finally
        {
            await source.CloseAsync();
            Directory.Delete(directory, recursive: true);
        }
    }

    private static void Assert(bool condition, string message)
    {
        if (!condition)
            throw new InvalidOperationException(message);
    }
}
