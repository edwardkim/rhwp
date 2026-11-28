using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Rhwp.WinUI;

internal sealed record SearchMatch(TextSelection Selection, [property: JsonRequired] bool Editable);
internal sealed record DocumentSearchResult(DocumentTarget Target, SearchMatch[] Matches, bool Truncated,
    bool OmittedContent);
internal sealed record SearchRectangle([property: JsonRequired] uint Page, CaretBounds Bounds);

internal sealed partial class NativeWorkerDispatcher
{
    internal const int MaxSearchQueryBytes = 256;
    private const int MaxSearchMatches = 128;
    private const int MaxNativeCellDepth = 32;
    private bool? workerTextSearch;

    public Task<bool> CanSearchAsync(CancellationToken cancellation) => Dispatch(token =>
    {
        if (workerTextSearch is bool supported)
            return supported;

        using JsonDocument capabilities = ReadSuccess(RhwpNative.SessionCapabilities());
        workerTextSearch = HasCapability(capabilities.RootElement, "workerTextSearch");
        return workerTextSearch.Value;
    }, cancellation);

    public Task<DocumentSearchResult> SearchAsync(DocumentTarget expected, string query, bool caseSensitive,
        CancellationToken cancellation, int maxMatches = MaxSearchMatches) => Dispatch(token =>
    {
        if (Encoding.UTF8.GetByteCount(query) is < 1 or > MaxSearchQueryBytes)
            throw new ArgumentException($"검색어는 UTF-8 기준 1~{MaxSearchQueryBytes}바이트여야 합니다.", nameof(query));
        if (maxMatches is < 1 or > MaxSearchMatches)
            throw new ArgumentOutOfRangeException(nameof(maxMatches));

        JsonElement reply = Submit(EnsureWorker(),
            new { op = "searchText", target = expected, request = new { query, caseSensitive, maxMatches } }, token);
        RequireSearchTarget(reply, "searchText", expected);
        SearchData result = Read<SearchData>(reply.GetProperty("result"));
        if (result.Matches is null || result.Matches.Length > maxMatches ||
            result.Matches.Any(match => match?.Selection is not TextSelection span ||
                span.Anchor?.CellPath is not { Length: <= MaxNativeCellDepth } address ||
                span.Focus?.CellPath is not { } focusAddress ||
                span.Anchor.Section != span.Focus.Section || span.Anchor.Paragraph != span.Focus.Paragraph ||
                address.Any(entry => entry is null) || !address.SequenceEqual(focusAddress) ||
                span.Anchor.ScalarOffset >= span.Focus.ScalarOffset))
            throw new InvalidDataException("Native 검색 결과의 텍스트 범위가 올바르지 않습니다.");

        return new DocumentSearchResult(expected, result.Matches, result.Truncated, result.OmittedContent);
    }, cancellation);

    public Task<SearchRectangle[]> GetSearchRectanglesAsync(PageInfo page, DocumentSearchResult result, int index,
        CancellationToken cancellation) => Dispatch(token =>
    {
        if (page.Target != result.Target)
            throw new InvalidOperationException("문서가 변경되었습니다. 다시 검색하세요.");

        JsonElement reply = Submit(EnsureWorker(),
            new { op = "selectionRects", target = result.Target, selection = result.Matches[index].Selection }, token);
        RequireSearchTarget(reply, "selectionRects", result.Target);
        SearchRectangle[] rectangles = Read<SearchRectangle[]>(reply.GetProperty("rects"));
        if (rectangles.Length > MaxSearchQueryBytes || rectangles.Any(rectangle =>
            rectangle is null || rectangle.Page >= page.PageCount ||
            rectangle.Bounds is not { } bounds ||
            !double.IsFinite(bounds.X) || !double.IsFinite(bounds.Y) ||
            !double.IsFinite(bounds.Width) || !double.IsFinite(bounds.Height) ||
            bounds.Width <= 0 || bounds.Height <= 0))
            throw new InvalidDataException("Native 검색 결과의 쪽 위치가 올바르지 않습니다.");

        return rectangles;
    }, cancellation);

    public Task<EditResult> ReplaceSearchMatchAsync(DocumentSearchResult result, int index, string text,
        CancellationToken cancellation)
    {
        SearchMatch match = result.Matches[index];
        if (!match.Editable)
            throw new InvalidOperationException("이 검색 결과는 편집할 수 없습니다.");

        return ApplyEdit(result.Target,
            new { op = "replaceRange", target = result.Target, selection = match.Selection, text }, cancellation);
    }

    private static void RequireSearchTarget(JsonElement reply, string operation, DocumentTarget expected)
    {
        if (reply.GetProperty("kind").GetString() != operation ||
            Read<DocumentTarget>(reply.GetProperty("target")) != expected)
            throw new InvalidDataException("Native 검색 응답의 문서가 바뀌었습니다.");
    }

    private sealed record SearchData(SearchMatch[] Matches, [property: JsonRequired] bool Truncated,
        [property: JsonRequired] bool OmittedContent);
}
