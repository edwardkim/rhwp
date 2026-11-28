using System.Text.Json;

namespace Rhwp.WinUI;

internal sealed record NativeTableLimits(int Rows, int Columns, int Cells)
{
    public bool Contains(double rows, double columns) => rows >= 1 && rows <= Rows && columns >= 1 &&
        columns <= Columns && Math.Truncate(rows) == rows && Math.Truncate(columns) == columns &&
        rows * columns <= Cells;
}

internal sealed partial class NativeWorkerDispatcher
{
    public Task<(NativeTableLimits? Table, NativeEquationLimits? Equation, NativeShapeLimits? Shape,
        NativePictureLimits? Picture)> GetInsertLimitsAsync(CancellationToken cancellation) =>
        Dispatch(_ =>
        {
            using var report = ReadSuccess(RhwpNative.SessionCapabilities());
            var capabilities = report.RootElement;
            return (ReadTableLimits(capabilities), ReadEquationLimits(capabilities), ReadShapeLimits(capabilities),
                ReadPictureLimits(capabilities));
        }, cancellation);

    private static NativeTableLimits? ReadTableLimits(JsonElement capabilities)
    {
        if (!HasCapability(capabilities, "createTable"))
            return null;

        NativeTableLimits limits = new(capabilities.GetProperty("maxCreateTableRows").GetInt32(),
            capabilities.GetProperty("maxCreateTableColumns").GetInt32(),
            capabilities.GetProperty("maxCreateTableCells").GetInt32());
        if (limits.Rows is < 1 or > ushort.MaxValue || limits.Columns is < 1 or > ushort.MaxValue || limits.Cells < 1)
            throw new InvalidDataException("Native 표 만들기 제한이 올바르지 않습니다.");

        return limits;
    }

    public Task<EditResult> CreateTableAsync(DocumentTarget expected, int rows, int columns, CancellationToken cancellation) =>
        Dispatch(token => AdoptEditOutcome(expected,
            SubmitEditCommand(expected, new { command = "createTable", rows, columns }, token)), cancellation);
}
