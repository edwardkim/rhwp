using System.Text.Json;

namespace Rhwp.WinUI;

internal sealed record CellPathEntry(uint Control, uint Cell, uint Paragraph);
internal sealed record TextPosition(uint Section, uint Paragraph, uint ScalarOffset, CellPathEntry[] CellPath);
internal sealed record TextSelection(TextPosition Anchor, TextPosition Focus);
internal sealed record CaretBounds(double X, double Y, double Width, double Height);
internal sealed record PageCaret(uint Page, double X, double Y, double Height, CaretBounds? CellBounds,
    bool CellOverflowed);
internal sealed record TextHit(TextPosition Position, PageCaret? Caret);
internal sealed record NativeDocumentInfo(ulong DocumentId, ulong Revision, ulong FontGeneration,
    uint PageCount, bool IsDirty, bool Editing, TextSelection? Selection, ulong? CompositionId,
    bool CanUndo, bool CanRedo)
{
    public DocumentTarget Target => new(DocumentId, Revision);
}

internal sealed record EditOutcome(ulong Revision, TextSelection Selection, bool DocumentChanged,
    bool HistoryChanged, ulong? CompositionId);
internal sealed record EditResult(DocumentTarget Target, EditOutcome Outcome);

internal sealed record Utf16Range(uint Start, uint End);
internal sealed record Utf16Selection(uint Anchor, uint Focus);
internal sealed record ImeReplacement(Utf16Range Range, string Text);
internal sealed record ImeSurroundingText(string Text, uint CursorScalar, uint? AnchorScalar,
    string SelectedText, uint AbsoluteStartScalar, TextPosition Position, bool TextTruncated,
    bool SelectionTruncated);
internal sealed record ImeEditContext(ulong Revision, ulong FontGeneration, ImeSurroundingText Context,
    TextSelection Selection, ulong? CompositionId, Utf16Range? Preedit);

internal readonly record struct DirectInputChange(string RemovedSuffix, string InsertedSuffix)
{
    public static DirectInputChange Between(string previous, string current)
    {
        if (current.StartsWith(previous, StringComparison.Ordinal))
            return new("", current[previous.Length..]);

        // The existing diff finds the common prefix without splitting a surrogate pair.
        int start = (int)PageEditText.Difference(previous, current)!.Range.Start;
        return new(previous[start..], current[start..]);
    }

    public ImeReplacement ReplacementAtCaret(ImeSurroundingText context)
    {
        int end = PageEditText.Utf16Offset(context.Text, context.CursorScalar);
        int start = end - RemovedSuffix.Length;
        if (start < 0 || !context.Text.AsSpan(start, RemovedSuffix.Length).SequenceEqual(RemovedSuffix.AsSpan()) ||
            PageEditText.Selection(context.Text, start, RemovedSuffix.Length) is null)
            throw new InvalidOperationException("입력기 조합과 문서의 커서 앞 텍스트가 일치하지 않습니다.");

        return new ImeReplacement(new Utf16Range((uint)start, (uint)end), InsertedSuffix);
    }
}

internal sealed partial class NativeWorkerDispatcher
{
    public Task<PageInfo> NewAsync(CancellationToken cancellation, bool discardChanges = false) =>
        Dispatch(token =>
        {
            CheckDirty(discardChanges);
            ulong handle = EnsureWorker();
            PageInfo firstPage = InitialPage(handle, Submit(handle, new { op = "new", replace = target }, token), token);
            newDocumentBaseline = (firstPage.Target, TryReadLosslessHwpx(firstPage.Target, token));
            return firstPage;
        }, cancellation);

    public Task<TextHit?> HitTestAsync(PageInfo page, double x, double y, CancellationToken cancellation) =>
        Query<TextHit>(new { op = "hitTest", target = page.Target, page.Page, x, y }, "hit", cancellation);

    public Task<PageCaret?> CaretAsync(DocumentTarget expected, TextPosition position, CancellationToken cancellation) =>
        Query<PageCaret>(new { op = "caretRect", target = expected, position }, "caret", cancellation);

    public Task<NativeDocumentInfo> GetDocumentInfoAsync(DocumentTarget expected, CancellationToken cancellation) =>
        Dispatch(token => AdoptInfo(Submit(EnsureWorker(), new { op = "info" }, token, interactive: true), expected),
            cancellation);

    public Task<NativeDocumentInfo> BeginEditAsync(DocumentTarget expected, TextSelection selection,
        CancellationToken cancellation) =>
        Dispatch(token => AdoptInfo(Submit(EnsureWorker(),
            new { op = "beginEdit", target = expected, selection }, token, interactive: true)), cancellation);

    public Task<EditResult> SetSelectionAsync(DocumentTarget expected, TextSelection selection,
        CancellationToken cancellation) =>
        ApplyEdit(expected, new { op = "setSelection", target = expected, selection }, cancellation);

    public Task<EditResult> ReplaceAsync(DocumentTarget expected, string text, CancellationToken cancellation) =>
        ApplyEdit(expected, new { op = "replace", target = expected, text }, cancellation);

    public Task<EditResult> UndoAsync(DocumentTarget expected, CancellationToken cancellation) =>
        ApplyEdit(expected, new { op = "undo", target = expected }, cancellation);

    public Task<EditResult> RedoAsync(DocumentTarget expected, CancellationToken cancellation) =>
        ApplyEdit(expected, new { op = "redo", target = expected }, cancellation);

    public Task<ImeEditContext> ReadImeEditContextAsync(DocumentTarget expected, CancellationToken cancellation) =>
        Dispatch(token => Read<ImeEditContext>(Submit(EnsureWorker(),
            new { op = "imeEditContext", target = expected }, token, interactive: true).GetProperty("context")), cancellation);

    public Task<EditResult> CommitImeAsync(DocumentTarget expected, ImeEditContext context,
        ImeReplacement replacement, Utf16Selection? selection, CancellationToken cancellation) =>
        ApplyEdit(expected, new
        {
            op = "applyImeEdit", target = expected, expected = context,
            edit = new { kind = "commit", replacement, selection },
        }, cancellation);

    public Task<EditResult> CommitParagraphAsync(DocumentTarget expected, ImeEditContext context,
        IReadOnlyList<ImeReplacement> replacements, Utf16Selection? selection, CancellationToken cancellation)
    {
        if (replacements.Count <= 1)
        {
            ImeReplacement replacement = replacements.Count == 0
                ? new ImeReplacement(new Utf16Range(0, 0), string.Empty) : replacements[0];
            return CommitImeAsync(expected, context, replacement, selection, cancellation);
        }

        return ApplyEdit(expected, new
        {
            op = "applyImeEdit", target = expected, expected = context,
            edit = new { kind = "commitBatch", replacements, selection },
        }, cancellation);
    }

    private Task<T?> Query<T>(object command, string field, CancellationToken cancellation) where T : class =>
        Dispatch(token => Submit(EnsureWorker(), command, token, interactive: true).GetProperty(field).Deserialize<T>(WireJson),
            cancellation);

    private Task<EditResult> ApplyEdit(DocumentTarget expected, object command, CancellationToken cancellation) =>
        Dispatch(token =>
        {
            JsonElement reply = Submit(EnsureWorker(), command, token, interactive: true);
            if (reply.GetProperty("kind").GetString() != "edited")
                throw new InvalidDataException("Native 편집 응답이 올바르지 않습니다.");

            return AdoptEditOutcome(expected, reply);
        }, cancellation);

    private EditResult AdoptEditOutcome(DocumentTarget expected, JsonElement reply)
    {
        EditOutcome outcome = Read<EditOutcome>(reply.GetProperty("outcome"));
        if (outcome.Revision < expected.Revision)
            throw new InvalidDataException("Native 편집 응답의 문서 버전이 올바르지 않습니다.");

        DocumentTarget current = new(expected.DocumentId, outcome.Revision);
        // A preview may change the render revision. An accepted completion
        // advances the dispatcher target even when a caller canceled its wait.
        target = current;
        if (outcome.HistoryChanged)
        {
            committedRevision = outcome.Revision;
            dirty = true;
        }

        return new EditResult(current, outcome);
    }

    private static T Read<T>(JsonElement element) where T : class =>
        element.Deserialize<T>(WireJson) ?? throw new InvalidDataException("Native document response is empty.");
}
