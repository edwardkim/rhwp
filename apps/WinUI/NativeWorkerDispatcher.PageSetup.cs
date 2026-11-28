using System.Text.Json;

namespace Rhwp.WinUI;

internal sealed record NativePageSettings(uint Width, uint Height, uint MarginLeft, uint MarginRight,
    uint MarginTop, uint MarginBottom, uint MarginHeader, uint MarginFooter, uint MarginGutter,
    bool Landscape, uint Binding);

internal sealed record NativePageSetup(uint Section, NativePageSettings Settings, TextPosition? StartPosition);

internal sealed partial class NativeWorkerDispatcher
{
    public Task<bool> SupportsPageSetupAsync(CancellationToken cancellation) => Dispatch(_ =>
    {
        using JsonDocument report = ReadSuccess(RhwpNative.SessionCapabilities());
        return HasCapability(report.RootElement, "pageSetup");
    }, cancellation);

    public Task<NativePageSetup> GetPageSetupAsync(PageInfo current, CancellationToken cancellation) => Dispatch(token =>
    {
        JsonElement reply = Submit(EnsureWorker(), new { op = "pageSetup", target = current.Target, page = current.Page }, token);
        if (reply.GetProperty("kind").GetString() != "pageSetup" ||
            Read<DocumentTarget>(reply.GetProperty("target")) != current.Target)
            throw new InvalidDataException("Native 쪽 설정의 문서가 바뀌었습니다.");

        NativePageSetup setup = Read<NativePageSetup>(reply.GetProperty("pageSetup"));
        if (setup.Settings is not { Width: > 0, Height: > 0, Binding: <= 2 } ||
            (setup.StartPosition is not null && (setup.StartPosition.Section != setup.Section ||
                setup.StartPosition.CellPath is not { Length: 0 })))
            throw new InvalidDataException("Native 쪽 설정을 읽을 수 없습니다.");

        return setup;
    }, cancellation);

    public Task<EditResult> SetPageSetupAsync(DocumentTarget expected, uint section, NativePageSettings settings,
        CancellationToken cancellation) => Dispatch(token => AdoptEditOutcome(expected, SubmitEditCommand(expected,
            new { command = "setPageSetup", section, settings }, token)), cancellation);
}
