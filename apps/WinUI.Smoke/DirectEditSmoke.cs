using Rhwp.WinUI;

internal static class DirectEditSmoke
{
    private static readonly CancellationToken Token = CancellationToken.None;

    public static async Task RunAsync()
    {
        var worker = new NativeWorkerDispatcher();
        string savedFile = Path.GetTempFileName();
        try
        {
            PageInfo page = await worker.NewAsync(Token);
            await PersistAsync(worker, page.Target, savedFile);
            var start = new TextPosition(0, 0, 0, []);
            await worker.BeginEditAsync(page.Target, new(start, start), Token);

            EditResult begun = await worker.BeginCompositionAsync(page.Target, Token);
            ImeEditContext empty = await worker.ReadImeEditContextAsync(begun.Target, Token);
            EditResult preview = await worker.ContinueCompositionAsync(begun.Target, empty,
                new(new(0, 0), "ㅎ"), new(1, 1), Token);
            Assert(preview.Target.Revision > begun.Target.Revision && preview.Outcome.DocumentChanged &&
                !preview.Outcome.HistoryChanged, "preview advances rendering without a committed edit");
            Assert(!(await worker.GetDocumentInfoAsync(preview.Target, Token)).IsDirty,
                "reading document info during preedit preserves the committed clean flag");
            await Reject(worker.ContinueCompositionAsync(preview.Target, empty, new(new(0, 0), "하"), null, Token),
                "stale IME context");
            await Reject(worker.CancelCompositionAsync(preview.Target, preview.Outcome.CompositionId!.Value + 1, Token),
                "composition");

            EditResult cancelled = await worker.CancelCompositionAsync(preview.Target,
                preview.Outcome.CompositionId!.Value, Token);
            NativeDocumentInfo restored = await worker.GetDocumentInfoAsync(cancelled.Target, Token);
            Assert(cancelled.Outcome.DocumentChanged && !cancelled.Outcome.HistoryChanged &&
                !restored.IsDirty && !restored.CanUndo && restored.CompositionId is null &&
                (await worker.ReadImeEditContextAsync(cancelled.Target, Token)).Context.Text.Length == 0,
                "cancel restores the editor and introduces no dirty state or history");
            PreparedDocumentSave abandoned = await worker.PrepareSaveAsync(cancelled.Target, DocumentFormat.Hwpx, Token);
            Assert(!(await worker.CancelSaveAsync(abandoned)).Dirty,
                "discarding a save token after composition cancellation preserves the clean document");

            begun = await worker.BeginCompositionAsync(cancelled.Target, Token);
            empty = await worker.ReadImeEditContextAsync(begun.Target, Token);
            preview = await worker.ContinueCompositionAsync(begun.Target, empty,
                new(new(0, 0), "한"), new(1, 1), Token);
            EditResult committed = await worker.CommitCompositionAsync(preview.Target, preview.Outcome.CompositionId!.Value, Token);
            Assert(committed.Target == preview.Target && !committed.Outcome.DocumentChanged &&
                committed.Outcome.HistoryChanged && (await worker.GetDocumentInfoAsync(committed.Target, Token)).IsDirty,
                "commit records the unchanged preview as exactly one edit");

            PreparedDocumentSave prepared = await worker.PrepareSaveAsync(committed.Target, DocumentFormat.Hwpx, Token);
            await File.WriteAllBytesAsync(savedFile, prepared.Bytes, Token);
            begun = await worker.BeginCompositionAsync(committed.Target, Token);
            ImeEditContext boundary = await worker.ReadImeEditContextAsync(begun.Target, Token);
            preview = await worker.ContinueCompositionAsync(begun.Target, boundary,
                new(new(1, 1), "ㄱ"), new(2, 2), Token);
            Assert(!(await worker.AcknowledgeSaveAsync(prepared)).Dirty,
                "an acknowledged save covers the latest commit even when the preview has advanced rendering");
            cancelled = await worker.CancelCompositionAsync(preview.Target, preview.Outcome.CompositionId!.Value, Token);
            Assert(!(await worker.GetDocumentInfoAsync(cancelled.Target, Token)).IsDirty &&
                (await worker.ReadImeEditContextAsync(cancelled.Target, Token)).Context.Text == "한",
                "cancel after a delayed save retains the saved commit");
            EditResult undone = await worker.UndoAsync(cancelled.Target, Token);
            Assert((await worker.ReadImeEditContextAsync(undone.Target, Token)).Context.Text.Length == 0,
                "one undo restores the original text before the committed Hangul composition");

            EditResult seeded = await worker.ReplaceAsync(undone.Target, "가😀나", Token);
            NativeCommandResult home = await worker.ExecuteEditCommandAsync(seeded.Target, NativeEditKey.Home, Token);
            NativeCommandResult right = await worker.ExecuteEditCommandAsync(home.Edit.Target, NativeEditKey.MoveRight, Token);
            NativeCommandResult selected = await worker.ExecuteEditCommandAsync(right.Edit.Target, NativeEditKey.MoveRight,
                Token, extendSelection: true);
            TextSelection range = selected.Edit.Outcome.Selection;
            Assert(range.Anchor.ScalarOffset == 1 && range.Focus.ScalarOffset == 2 &&
                selected.Edit.Target == seeded.Target, "body commands extend the selection over an emoji without mutating");
            SelectionRectangle[] rects = await worker.GetSelectionRectanglesAsync(page with { Target = seeded.Target }, range, Token);
            Assert(rects.Any(rect => rect.Page == 0 && rect.Bounds.Width > 0 && rect.Bounds.Height > 0),
                "the selected range has page-local geometry");
            NativeCommandResult deleted = await worker.ExecuteEditCommandAsync(seeded.Target, NativeEditKey.DeleteBackward, Token);
            Assert((await worker.ReadImeEditContextAsync(deleted.Edit.Target, Token)).Context.Text == "가나",
                "delete respects the selected Unicode scalar range");
            await Reject(worker.GetSelectionRectanglesAsync(page with { Target = seeded.Target }, range, Token), "staleRevision");
            await Reject(worker.ExecuteEditCommandAsync(seeded.Target, NativeEditKey.MoveLeft, Token), "staleRevision");

            NativeCommandResult split = await worker.ExecuteEditCommandAsync(deleted.Edit.Target, NativeEditKey.SplitParagraph,
                Token);
            NativeCommandResult up = await worker.ExecuteEditCommandAsync(split.Edit.Target, NativeEditKey.MoveUp, Token);
            Assert(up.PreferredX is double && up.Edit.Outcome.Selection.Focus.Paragraph == 0,
                "vertical movement returns a preferred column and updated position");

            if (await worker.SupportsBasicTextFormattingAsync(Token))
            {
                var first = new TextPosition(0, 0, 0, []);
                EditResult formatSelection = await worker.SetSelectionAsync(up.Edit.Target,
                    new(first, first with { ScalarOffset = 1 }), Token);
                EditResult bold = await worker.FormatCharactersAsync(formatSelection.Target, true, null, Token);
                EditResult size = await worker.FormatCharactersAsync(bold.Target, null, 1800, Token);
                EditResult align = await worker.SetParagraphAlignmentAsync(size.Target, NativeParagraphAlignment.Right,
                    Token);
                Assert(bold.Target.Revision > formatSelection.Target.Revision &&
                    size.Target.Revision > bold.Target.Revision && align.Target.Revision > size.Target.Revision &&
                    align.Outcome.Selection.Anchor.ScalarOffset == 0 && align.Outcome.Selection.Focus.ScalarOffset == 1,
                    "consecutive formatting commands accept and adopt each revision without losing the selection");
                await PersistAsync(worker, align.Target, savedFile);
                Console.WriteLine("Formatting: bold, size and alignment advance revisions, retain selection and save.");
            }

            Console.WriteLine("Direct edit: revision-safe movement and geometry, Hangul preview/cancel/commit, undo and committed saves.");
        }
        finally
        {
            await worker.CloseAsync();
            File.Delete(savedFile);
        }
    }

    private static async Task PersistAsync(NativeWorkerDispatcher worker, DocumentTarget target, string destination)
    {
        PreparedDocumentSave prepared = await worker.PrepareSaveAsync(target, DocumentFormat.Hwpx, Token);
        await File.WriteAllBytesAsync(destination, prepared.Bytes, Token);
        Assert(!(await worker.AcknowledgeSaveAsync(prepared)).Dirty, "acknowledged save is clean");
    }

    private static async Task Reject(Task operation, string message)
    {
        try
        {
            await operation;
        }
        catch (InvalidOperationException error) when (error.Message.Contains(message, StringComparison.OrdinalIgnoreCase))
        {
            return;
        }

        throw new InvalidOperationException("Direct edit accepted an outdated or invalid request.");
    }

    private static void Assert(bool condition, string message)
    {
        if (!condition)
            throw new InvalidOperationException("Direct edit smoke: " + message);
    }
}
