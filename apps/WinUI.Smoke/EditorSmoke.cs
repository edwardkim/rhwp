using Rhwp.WinUI;

internal static class EditorSmoke
{
    public static async Task RunAsync(string publicForm)
    {
        CheckDialogOffsets();
        var worker = new NativeWorkerDispatcher();
        CancellationToken token = CancellationToken.None;
        try
        {
            PageInfo form = await worker.OpenAsync(publicForm, token);
            var applicant = new TextPosition(0, 1, 3, [new(0, 1, 1), new(0, 3, 0)]);
            TextHit hit = await HitAtCaret(worker, form, applicant, token);
            Assert(hit.Position.Paragraph == applicant.Paragraph &&
                hit.Position.CellPath.SequenceEqual(applicant.CellPath) &&
                hit.Position.ScalarOffset == applicant.ScalarOffset, "public form applicant hit-test address");

            PageInfo blank = await worker.NewAsync(token);
            var start = new TextPosition(0, 0, 0, []);
            NativeDocumentInfo editor = await worker.BeginEditAsync(blank.Target, new(start, start), token);
            Assert(editor.Editing && editor.Target == blank.Target, "no document mutation on BeginEdit");
            EditResult seeded = await worker.ReplaceAsync(editor.Target, "가😀나", token);
            Assert(seeded.Outcome.Selection.Focus.ScalarOffset == 3, "insertion counts Unicode scalars");
            PageInfo page = await worker.GetPageAsync(blank with { Target = seeded.Target }, 0, token);
            var beforeEmoji = start with { ScalarOffset = 1 };
            var afterEmoji = start with { ScalarOffset = 2 };
            hit = await HitAtCaret(worker, page, afterEmoji, token);
            Assert(hit.Position.ScalarOffset == 2 && hit.Position.CellPath.Length == 0, "emoji hit results use scalars");

            EditResult selected = await worker.SetSelectionAsync(seeded.Target, new(afterEmoji, beforeEmoji), token);
            Assert(selected.Target == seeded.Target && !selected.Outcome.DocumentChanged,
                "selection preserves revision");
            ImeEditContext context = await worker.ReadImeEditContextAsync(selected.Target, token);
            Assert(context.Context.Text == "가😀나" && context.Context.SelectedText == "😀" &&
                context.Context.CursorScalar == 1 && context.Context.AnchorScalar == 2 &&
                context.FontGeneration == editor.FontGeneration && context.Revision == selected.Target.Revision,
                "typed reversed selection and native IME stamps");

            await Reject(worker.CommitImeAsync(selected.Target, context,
                new(new(2, 3), "한"), null, token), "surrogate pair");
            ImeReplacement replacement = PageEditText.Difference(context.Context.Text, "가한나")
                ?? throw new InvalidOperationException("Missing dialog delta.");
            Assert(replacement.Range == new Utf16Range(1, 3), "dialog edits only the changed scalar");
            EditResult committed = await worker.CommitImeAsync(selected.Target, context,
                replacement, PageEditText.Selection("가한나", 2, 0), token);
            Assert(committed.Outcome.DocumentChanged && committed.Outcome.HistoryChanged &&
                committed.Outcome.Selection.Focus.ScalarOffset == 2, "one UTF-16 IME commit");
            Assert((await worker.ReadImeEditContextAsync(committed.Target, token)).Context.Text == "가한나",
                "same logical character replaced");
            await Reject(worker.ReadImeEditContextAsync(selected.Target, token), "staleRevision");

            NativeDocumentInfo editedVersion = await worker.GetDocumentInfoAsync(committed.Target, token);
            Assert(editedVersion.CanUndo && !editedVersion.CanRedo && editedVersion.IsDirty,
                "toolbar and dirty state come from the exact native revision");
            EditResult undone = await worker.UndoAsync(committed.Target, token);
            Assert((await worker.GetDocumentInfoAsync(undone.Target, token)).CanRedo, "redo available after undo");
            ImeEditContext restored = await worker.ReadImeEditContextAsync(undone.Target, token);
            Assert(restored.Context.Text == "가😀나" && restored.Context.SelectedText == "😀",
                "undo restores text and backwards selection");
            EditResult redone = await worker.RedoAsync(undone.Target, token);
            Assert((await worker.ReadImeEditContextAsync(redone.Target, token)).Context.Text == "가한나",
                "redo restores committed text");
            await Reject(worker.OpenAsync(publicForm, token), "변경 사항");
            await Reject(worker.CloseDocumentAsync(token), "변경 사항");
            string corrupt = Path.GetTempFileName();
            try
            {
                await File.WriteAllTextAsync(corrupt, "invalid HWP", token);
                await Reject(worker.OpenAsync(corrupt, token, discardChanges: true), "documentError");
                Assert((await worker.ReadImeEditContextAsync(redone.Target, token)).Context.Text == "가한나",
                    "failed replacement retains the previous dirty document at the same target");
            }
            finally
            {
                File.Delete(corrupt);
            }
            await worker.CloseDocumentAsync(token, discardChanges: true);
            Assert((await worker.OpenAsync(publicForm, token)).Target != form.Target, "file reopen after explicit discard");
            Console.WriteLine("Editor ABI: public leaf hit-test; scalar selection; UTF-16 IME; undo/redo; dirty close guard.");
        }
        finally
        {
            await worker.CloseAsync();
        }
    }

    private static void CheckDialogOffsets()
    {
        Assert(PageEditText.Utf16Offset("가😀나", 2) == 3, "offset before the last scalar");
        Assert(PageEditText.Difference("같음", "같음") is null, "unchanged dialog");
        Assert(PageEditText.Difference("가😀나", "가😁나") ==
            new ImeReplacement(new(1, 3), "😁"), "shared high surrogate");
        Assert(PageEditText.Difference("😀!", "\U0001FA00!") ==
            new ImeReplacement(new(0, 2), "\U0001FA00"), "shared low surrogate");
        Assert(PageEditText.Difference("문단", "문단 이름") ==
            new ImeReplacement(new(2, 2), " 이름"), "append");
        Assert(PageEditText.Difference("문단 이름", "문단") ==
            new ImeReplacement(new(2, 5), ""), "delete");
        Assert(PageEditText.Selection("가😀나", 2, 0) is null &&
            PageEditText.Selection("가😀나", 1, 2) == new Utf16Selection(1, 3),
            "TextBox selection must remain on UTF-16 boundaries");
    }

    private static async Task<TextHit> HitAtCaret(NativeWorkerDispatcher worker, PageInfo page,
        TextPosition position, CancellationToken token)
    {
        PageCaret caret = await worker.CaretAsync(page.Target, position, token)
            ?? throw new InvalidOperationException("Missing real caret geometry.");
        return await worker.HitTestAsync(page, caret.X, caret.Y + caret.Height / 2, token)
            ?? throw new InvalidOperationException("No text hit at caret.");
    }

    private static async Task Reject(Task request, string message)
    {
        try
        {
            await request;
            throw new Exception("Expected editor request rejection: " + message);
        }
        catch (InvalidOperationException error) when (error.Message.Contains(message, StringComparison.Ordinal))
        {
        }
    }

    private static void Assert(bool condition, string label)
    {
        if (!condition)
            throw new InvalidOperationException("Editor ABI: " + label);
    }
}
