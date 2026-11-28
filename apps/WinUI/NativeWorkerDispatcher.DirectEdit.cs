using System.Text.Json;
using System.Text.Json.Serialization;

namespace Rhwp.WinUI;

internal enum NativeEditKey
{
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Home,
    End,
    DeleteBackward,
    DeleteForward,
    SplitParagraph,
    JoinPreviousParagraph,
    SelectAll,
}

internal sealed record NativeCommandResult(EditResult Edit, double? PreferredX);
internal sealed record SelectionRectangle([property: JsonRequired] uint Page, CaretBounds Bounds);

internal sealed partial class NativeWorkerDispatcher
{
    public Task<SelectionRectangle[]> GetSelectionRectanglesAsync(PageInfo page, TextSelection selection,
        CancellationToken cancellation) => Dispatch(token =>
    {
        JsonElement reply = Submit(EnsureWorker(),
            new { op = "selectionRects", target = page.Target, selection }, token);
        if (reply.GetProperty("kind").GetString() != "selectionRects" ||
            Read<DocumentTarget>(reply.GetProperty("target")) != page.Target)
            throw new InvalidDataException("Native 선택 영역 응답의 문서가 바뀌었습니다.");

        SelectionRectangle[] rectangles = Read<SelectionRectangle[]>(reply.GetProperty("rects"));
        if (rectangles.Any(rectangle =>
            rectangle is null || rectangle.Page >= page.PageCount ||
            rectangle.Bounds is not { } bounds ||
            !double.IsFinite(bounds.X) || !double.IsFinite(bounds.Y) ||
            !double.IsFinite(bounds.Width) || !double.IsFinite(bounds.Height) ||
            bounds.Width < 0 || bounds.Height < 0))
            throw new InvalidDataException("Native 선택 영역의 위치가 올바르지 않습니다.");

        return rectangles;
    }, cancellation);

    public Task<NativeCommandResult> ExecuteEditCommandAsync(DocumentTarget expected, NativeEditKey key,
        CancellationToken cancellation, bool extendSelection = false, double? preferredX = null) => Dispatch(token =>
    {
        string command = JsonNamingPolicy.CamelCase.ConvertName(key.ToString());
        object arguments = key switch
        {
            NativeEditKey.MoveUp or NativeEditKey.MoveDown => new { command, extendSelection, preferredX },
            NativeEditKey.MoveLeft or NativeEditKey.MoveRight or NativeEditKey.Home or NativeEditKey.End =>
                new { command, extendSelection },
            NativeEditKey.DeleteBackward or NativeEditKey.DeleteForward or NativeEditKey.SplitParagraph or
                NativeEditKey.JoinPreviousParagraph or NativeEditKey.SelectAll => new { command },
            _ => throw new ArgumentOutOfRangeException(nameof(key)),
        };

        JsonElement reply = SubmitEditCommand(expected, arguments, token);
        EditResult edit = AdoptEditOutcome(expected, reply);
        JsonElement x = reply.GetProperty("preferredX");
        double? nextPreferredX = x.ValueKind == JsonValueKind.Null ? null : x.GetDouble();
        if (nextPreferredX is double value && !double.IsFinite(value))
            throw new InvalidDataException("Native 캐럿의 기준 열이 올바르지 않습니다.");

        return new NativeCommandResult(edit, nextPreferredX);
    }, cancellation);

    private JsonElement SubmitEditCommand(DocumentTarget expected, object command, CancellationToken token)
    {
        JsonElement reply = Submit(EnsureWorker(), new { op = "executeCommand", target = expected, command }, token);
        if (reply.GetProperty("kind").GetString() != "command")
            throw new InvalidDataException("Native 편집 명령의 응답이 올바르지 않습니다.");
        return reply;
    }

    public Task<EditResult> BeginCompositionAsync(DocumentTarget expected, CancellationToken cancellation) =>
        ApplyEdit(expected, new { op = "beginComposition", target = expected }, cancellation);

    public Task<EditResult> ContinueCompositionAsync(DocumentTarget expected, ImeEditContext context,
        ImeReplacement replacement, Utf16Selection? selection, CancellationToken cancellation) =>
        ApplyEdit(expected, new
        {
            op = "applyImeEdit", target = expected, expected = context,
            edit = new { kind = "continue", replacement, selection },
        }, cancellation);

    public Task<EditResult> CommitCompositionAsync(DocumentTarget expected, ulong compositionId,
        CancellationToken cancellation) =>
        ApplyEdit(expected, new { op = "commitComposition", target = expected, compositionId }, cancellation);

    public Task<EditResult> CancelCompositionAsync(DocumentTarget expected, ulong compositionId,
        CancellationToken cancellation) =>
        ApplyEdit(expected, new { op = "cancelComposition", target = expected, compositionId }, cancellation);
}
