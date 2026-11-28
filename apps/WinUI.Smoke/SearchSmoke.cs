using Rhwp.WinUI;

internal static class SearchSmoke
{
    public static async Task RunAsync(string publicForm)
    {
        var worker = new NativeWorkerDispatcher();
        CancellationToken token = CancellationToken.None;
        try
        {
            if (!await worker.CanSearchAsync(token))
            {
                Console.WriteLine("Native search smoke skipped: loaded library does not provide workerTextSearch.");
                return;
            }

            PageInfo form = await worker.OpenAsync(publicForm, token);
            const string query = "생년월일";
            const string newText = "신청일자";
            DocumentSearchResult matches = await worker.SearchAsync(form.Target, query, true, token);
            Assert(matches.Target == form.Target && matches.Matches.Length == 2 && !matches.Truncated &&
                matches.Matches.All(match => match.Editable && match.Selection.Anchor.CellPath.Length == 2),
                "distinct matches in nested table cells");

            SearchMatch first = matches.Matches[0];
            SearchMatch second = matches.Matches[1];
            Assert(!first.Selection.Anchor.CellPath.SequenceEqual(second.Selection.Anchor.CellPath),
                "applicant and child addresses differ");
            Assert((await worker.GetSearchRectanglesAsync(form, matches, 0, token)).Length != 0,
                "native search highlights have page geometry");
            DocumentSearchResult limited = await worker.SearchAsync(form.Target, query, true, token, maxMatches: 1);
            Assert(limited.Matches.Length == 1 && limited.Truncated, "native match limit is exposed");

            NativeDocumentInfo prior = await worker.GetDocumentInfoAsync(form.Target, token);
            Assert(!prior.Editing && !prior.IsDirty, "viewing search matches does not enter the editor");
            await worker.BeginEditAsync(form.Target, second.Selection, token);
            EditResult changed = await worker.ReplaceSearchMatchAsync(matches, 0, newText, token);
            Assert(changed.Outcome.DocumentChanged && changed.Outcome.HistoryChanged && changed.Target != form.Target,
                "explicit search replacement is undoable");
            await RejectStale(worker.ReplaceSearchMatchAsync(matches, 1, newText, token));
            await RejectStale(worker.GetSearchRectanglesAsync(form, matches, 1, token));

            DocumentSearchResult remaining = await worker.SearchAsync(changed.Target, query, true, token);
            Assert(remaining.Matches.Length == 1 &&
                remaining.Matches[0].Selection.Anchor.CellPath.SequenceEqual(second.Selection.Anchor.CellPath),
                "explicit replacement leaves the other hit untouched");
            DocumentSearchResult replaced = await worker.SearchAsync(changed.Target, newText, true, token);
            Assert(replaced.Matches.Length == 1 &&
                replaced.Matches[0].Selection.Anchor.CellPath.SequenceEqual(first.Selection.Anchor.CellPath),
                "new text appears only at the addressed hit");
            EditResult undone = await worker.UndoAsync(changed.Target, token);
            Assert(undone.Outcome.Selection.Anchor.CellPath.SequenceEqual(second.Selection.Anchor.CellPath),
                "undo restores the prior live selection");
            Assert((await worker.SearchAsync(undone.Target, query, true, token)).Matches.Length == 2,
                "undo restores both hits");
            Console.WriteLine("Native search: nested-cell hits and highlights, stale revision rejection, limit, replace and undo.");
        }
        finally
        {
            await worker.CloseAsync();
        }
    }

    private static async Task RejectStale(Task operation)
    {
        try
        {
            await operation;
        }
        catch (InvalidOperationException error) when (error.Message.Contains("staleRevision", StringComparison.Ordinal))
        {
            return;
        }

        throw new InvalidOperationException("Native search accepted an outdated revision.");
    }

    private static void Assert(bool condition, string message)
    {
        if (!condition)
            throw new InvalidOperationException("Native search smoke: " + message);
    }
}
