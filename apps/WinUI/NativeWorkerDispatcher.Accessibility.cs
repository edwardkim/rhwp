using System.Text;
using System.Text.Json;

namespace Rhwp.WinUI;

internal sealed partial class NativeWorkerDispatcher
{
    private const int MaxAutomationNodes = 512;
    private const int MaxAutomationTextBytes = 512 * 1024;
    private const int MaxAutomationDepth = 8;
    private bool? accessibilitySnapshots;
    private bool accessibilityCellLayout;

    public Task<DocumentAutomationSnapshot?> GetAutomationDocumentAsync(PageInfo page,
        CancellationToken cancellation) => Dispatch<DocumentAutomationSnapshot?>(token =>
    {
        if (accessibilitySnapshots is null)
        {
            using JsonDocument capabilities = ReadSuccess(RhwpNative.SessionCapabilities());
            accessibilitySnapshots = HasCapability(capabilities.RootElement, "accessibilitySnapshots");
            accessibilityCellLayout = HasCapability(capabilities.RootElement, "accessibilityCellLayout");
        }

        if (accessibilitySnapshots != true)
            return null;

        JsonElement reply = Submit(EnsureWorker(), new
        {
            op = "accessibility", target = page.Target,
            request = new
            {
                geometry = false,
                includeCellLayout = accessibilityCellLayout,
                maxNodes = MaxAutomationNodes,
                maxTextBytes = MaxAutomationTextBytes,
                maxDepth = MaxAutomationDepth,
            },
        }, token);

        return ReadAutomationDocument(reply, page);
    }, cancellation);

    private static bool HasCapability(JsonElement report, string name) =>
        report.TryGetProperty(name, out JsonElement value) && value.ValueKind == JsonValueKind.True;

    private static DocumentAutomationSnapshot ReadAutomationDocument(JsonElement reply, PageInfo page)
    {
        if (reply.GetProperty("kind").GetString() != "accessibility" ||
            Read<DocumentTarget>(reply.GetProperty("target")) != page.Target)
            throw new InvalidDataException("Native 접근성 응답의 문서가 바뀌었습니다.");

        AutomationData snapshot = Read<AutomationData>(reply.GetProperty("snapshot"));
        if (snapshot.Revision != page.Target.Revision || snapshot.FontGeneration != page.FontGeneration ||
            snapshot.Scope != "bodyAndTableCellText" || snapshot.Nodes is not { Length: <= MaxAutomationNodes })
            throw new InvalidDataException("Native 접근성 응답의 범위가 올바르지 않습니다.");

        var paragraphs = new List<DocumentAutomationParagraph>(snapshot.Nodes.Length);
        var keys = new HashSet<string>(StringComparer.Ordinal);
        int textBytes = 0;
        foreach (AutomationNode node in snapshot.Nodes)
        {
            if (node.Start?.CellPath is not { Length: <= MaxAutomationDepth } path ||
                node.Text is null || string.IsNullOrEmpty(node.Key) || !keys.Add(node.Key) ||
                node.Role != (path.Length == 0 ? "bodyParagraph" : "tableCellParagraph"))
                throw new InvalidDataException("Native 접근성 문단의 주소가 올바르지 않습니다.");

            textBytes += Encoding.UTF8.GetByteCount(node.Text);
            if (textBytes > MaxAutomationTextBytes)
                throw new InvalidDataException("Native 접근성 텍스트 한도를 초과했습니다.");

            var cells = new DocumentAutomationCell[path.Length];
            string container = FormattableString.Invariant($"s{node.Start.Section}/p{node.Start.Paragraph}");
            for (int depth = 0; depth < cells.Length; depth++)
            {
                CellPathEntry entry = path[depth];
                string tableKey = FormattableString.Invariant($"{container}/t{entry.Control}");
                string cellKey = FormattableString.Invariant($"{tableKey}/c{entry.Cell}");
                GridCellData? grid = depth == cells.Length - 1 ? node.CellLayout : null;
                if (grid is { } value && !value.IsValid)
                    throw new InvalidDataException("Native 접근성 셀의 행·열 범위가 올바르지 않습니다.");

                cells[depth] = new DocumentAutomationCell(tableKey, cellKey, grid?.Row, grid?.Column,
                    grid?.RowSpan, grid?.ColumnSpan, grid?.RowCount, grid?.ColumnCount);
                container = FormattableString.Invariant($"{cellKey}/p{entry.Paragraph}");
            }

            paragraphs.Add(new DocumentAutomationParagraph(node.Key, node.Text, node.TextComplete, cells));
        }

        return new DocumentAutomationSnapshot(paragraphs, snapshot.Truncated || !snapshot.Complete);
    }

    private sealed record AutomationData(ulong Revision, ulong FontGeneration, string Scope,
        AutomationNode[] Nodes, bool Complete, bool Truncated);

    private sealed record AutomationNode(string Key, string Role, TextPosition Start, string Text,
        bool TextComplete, GridCellData? CellLayout);

    private sealed record GridCellData(int Row, int Column, int RowSpan, int ColumnSpan,
        int RowCount, int ColumnCount)
    {
        public bool IsValid => Row >= 0 && Row < RowCount && RowSpan > 0 && RowSpan <= RowCount - Row &&
                               Column >= 0 && Column < ColumnCount && ColumnSpan > 0 &&
                               ColumnSpan <= ColumnCount - Column;
    }
}
