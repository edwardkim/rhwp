namespace Rhwp.WinUI;

internal sealed record DocumentAutomationSnapshot(
    IReadOnlyList<DocumentAutomationParagraph> Paragraphs,
    bool Truncated);

internal sealed record DocumentAutomationParagraph(
    string Key,
    string Text,
    bool TextComplete,
    IReadOnlyList<DocumentAutomationCell> CellPath);

internal sealed record DocumentAutomationCell(
    string TableKey,
    string CellKey,
    int? Row = null,
    int? Column = null,
    int? RowSpan = null,
    int? ColumnSpan = null,
    int? RowCount = null,
    int? ColumnCount = null);

internal enum DocumentAutomationRole
{
    Document,
    Table,
    Cell,
    Paragraph,
}

internal sealed class DocumentAutomationNode(
    DocumentAutomationRole role,
    string key,
    string text = "",
    bool textComplete = true)
{
    internal DocumentAutomationRole Role { get; } = role;
    internal string Key { get; } = key;
    internal string Text { get; } = text;
    internal bool TextComplete { get; } = textComplete;
    internal DocumentAutomationCell? Layout { get; private set; }
    internal List<DocumentAutomationNode> Children { get; } = [];

    internal static DocumentAutomationNode Build(DocumentAutomationSnapshot snapshot)
    {
        var document = new DocumentAutomationNode(DocumentAutomationRole.Document, "document");
        var groups = new Dictionary<(DocumentAutomationNode Parent, DocumentAutomationRole Role, string Key),
            DocumentAutomationNode>();

        foreach (DocumentAutomationParagraph paragraph in snapshot.Paragraphs)
        {
            DocumentAutomationNode parent = document;
            foreach (DocumentAutomationCell item in paragraph.CellPath)
            {
                DocumentAutomationNode table = FindOrAddGroup(parent, DocumentAutomationRole.Table, item.TableKey);
                if (table.Layout is null && item is { RowCount: not null, ColumnCount: not null })
                    table.Layout = item;

                parent = FindOrAddGroup(table, DocumentAutomationRole.Cell, item.CellKey);
                if (parent.Layout is null && item is { Row: not null, Column: not null })
                    parent.Layout = item;
            }

            parent.Children.Add(new DocumentAutomationNode(DocumentAutomationRole.Paragraph,
                $"paragraph:{paragraph.Key}", paragraph.Text, paragraph.TextComplete));
        }

        return document;

        DocumentAutomationNode FindOrAddGroup(DocumentAutomationNode parent, DocumentAutomationRole role, string key)
        {
            var identity = (parent, role, key);
            if (groups.TryGetValue(identity, out DocumentAutomationNode? node))
                return node;

            node = new DocumentAutomationNode(role, $"{parent.Key}/{role}:{key}");
            parent.Children.Add(node);
            groups.Add(identity, node);
            return node;
        }
    }

    internal string Name => Role switch
    {
        DocumentAutomationRole.Table when Layout is { RowCount: int rows, ColumnCount: int columns } =>
            $"표, {rows}행 {columns}열",
        DocumentAutomationRole.Table => "표",
        DocumentAutomationRole.Cell when Layout is { Row: int row, Column: int column } =>
            $"{row + 1}행 {column + 1}열 셀",
        DocumentAutomationRole.Cell => "표 셀",
        DocumentAutomationRole.Paragraph when Text.Length != 0 => Text,
        DocumentAutomationRole.Paragraph => TextComplete ? "빈 문단" : "문단",
        _ => "문서 내용",
    };

    internal string HelpText => Role switch
    {
        DocumentAutomationRole.Cell when Layout is { RowSpan: int rows, ColumnSpan: int columns }
            && (rows > 1 || columns > 1) => $"{rows}행 {columns}열에 걸친 병합 셀",
        DocumentAutomationRole.Paragraph when !TextComplete => "문단의 일부만 제공됩니다.",
        _ => "",
    };
}
