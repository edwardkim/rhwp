using System.Text.Json;

namespace Rhwp.WinUI;

internal sealed record TableObjectAddress(uint Section, uint Paragraph, uint Control);

internal sealed record NativeTableObjectProperties(uint TableWidth, uint TableHeight, short CellSpacing,
    short PaddingLeft, short PaddingRight, short PaddingTop, short PaddingBottom,
    short OuterLeft, short OuterRight, short OuterTop, short OuterBottom, byte PageBreak, bool RepeatHeader);

internal sealed record TableObjectPropertyChanges(short CellSpacing, short PaddingLeft, short PaddingRight,
    short PaddingTop, short PaddingBottom, short OuterLeft, short OuterRight, short OuterTop, short OuterBottom,
    byte PageBreak, bool RepeatHeader);

internal sealed record NativeTableObject(TableObjectAddress Address, ushort RowCount, ushort ColumnCount,
    CaretBounds? Bounds, NativeTableObjectProperties Properties, bool Editable, bool CanResize);

internal sealed partial class NativeWorkerDispatcher
{
    private bool? tableObjectAvailable;

    public Task<bool> SupportsTableObjectsAsync(CancellationToken cancellation) => Dispatch(_ =>
    {
        if (tableObjectAvailable is bool supported)
            return supported;
        using JsonDocument reply = ReadSuccess(RhwpNative.SessionCapabilities());
        supported = HasCapability(reply.RootElement, "tableObjectProperties");
        tableObjectAvailable = supported;
        return supported;
    }, cancellation);

    public Task<NativeTableObject> GetTableObjectAsync(PageInfo current, TextPosition position,
        CancellationToken cancellation) => Dispatch(token =>
    {
        if (position.CellPath is not [{ } cell])
            throw new InvalidDataException("표 전체를 선택하려면 표 셀을 선택하세요.");
        JsonElement reply = Submit(EnsureWorker(),
            new { op = "tableObject", target = current.Target, current.Page, position }, token, interactive: true);
        if (reply.GetProperty("kind").GetString() != "tableObject" ||
            Read<DocumentTarget>(reply.GetProperty("target")) != current.Target ||
            reply.GetProperty("fontGeneration").GetUInt64() != current.FontGeneration ||
            reply.GetProperty("page").GetUInt32() != current.Page)
            throw new InvalidDataException("표 위치를 조회하는 동안 문서가 바뀌었습니다.");

        NativeTableObject table = Read<NativeTableObject>(reply.GetProperty("table"));
        if (table.Address != new TableObjectAddress(position.Section, position.Paragraph, cell.Control) ||
            table.RowCount == 0 || table.ColumnCount == 0 || table.Properties.PageBreak > 2 ||
            table.Bounds is CaretBounds bounds &&
                (!double.IsFinite(bounds.X) || !double.IsFinite(bounds.Y) ||
                 !double.IsFinite(bounds.Width) || !double.IsFinite(bounds.Height) ||
                 bounds.Width < 0 || bounds.Height < 0))
            throw new InvalidDataException("Native 표 위치 또는 속성을 읽을 수 없습니다.");
        return table;
    }, cancellation);

    public Task<EditResult> SetTableObjectPropertiesAsync(DocumentTarget expected, TableObjectAddress table,
        TableObjectPropertyChanges properties, CancellationToken cancellation) =>
        Dispatch(token => AdoptEditOutcome(expected, SubmitEditCommand(expected,
            new { command = "setTableObjectProperties", table, properties }, token)), cancellation);

    public Task<EditResult> ResizeTableObjectAsync(DocumentTarget expected, TableObjectAddress table,
        uint width, uint height, CancellationToken cancellation) =>
        Dispatch(token => AdoptEditOutcome(expected, SubmitEditCommand(expected,
            new { command = "resizeTableObject", table, width, height }, token)), cancellation);
}
