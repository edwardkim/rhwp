using System.Text.Json;

namespace Rhwp.WinUI;

internal enum NativeParagraphAlignment
{
    Left,
    Center,
    Right,
    Justify,
    Distribute,
    Split,
}

internal sealed record NativeFormattingCapabilities(bool Edit, bool Inspect, bool FontAndColor, bool SystemFonts);
internal sealed record NativeFormattingValue<T>(string Kind, T Value);
internal sealed record NativeSelectionFormatting(NativeFormattingValue<bool> Bold,
    NativeFormattingValue<int> FontSize, string Alignment,
    NativeFormattingValue<string?>[]? FontNames, NativeFormattingValue<string?>? TextColor);

internal sealed partial class NativeWorkerDispatcher
{
    private NativeFormattingCapabilities? formattingCapabilities;

    public Task<NativeFormattingCapabilities> GetFormattingCapabilitiesAsync(CancellationToken cancellation) =>
        Dispatch(_ => ReadFormattingCapabilities(), cancellation);

    public Task<bool> SupportsBasicTextFormattingAsync(CancellationToken cancellation) =>
        Dispatch(_ => ReadFormattingCapabilities().Edit, cancellation);

    private NativeFormattingCapabilities ReadFormattingCapabilities()
    {
        if (formattingCapabilities is not null)
            return formattingCapabilities;

        using var report = ReadSuccess(RhwpNative.SessionCapabilities());
        formattingCapabilities = new(HasCapability(report.RootElement, "basicTextFormatting"),
            HasCapability(report.RootElement, "selectionFormatting"),
            HasCapability(report.RootElement, "characterFontColorFormatting"),
            HasCapability(report.RootElement, "systemFontFamilies"));
        return formattingCapabilities;
    }

    public Task<NativeSelectionFormatting?> GetSelectionFormattingAsync(DocumentTarget expected, ulong fontGeneration,
        TextSelection selection, CancellationToken cancellation) =>
        Dispatch<NativeSelectionFormatting?>(token =>
        {
            if (!ReadFormattingCapabilities().Inspect)
                return null;

            JsonElement reply = Submit(EnsureWorker(),
                new { op = "selectionFormatting", target = expected, fontGeneration, selection }, token);
            if (reply.GetProperty("kind").GetString() != "selectionFormatting" ||
                Read<DocumentTarget>(reply.GetProperty("target")) != expected ||
                reply.GetProperty("fontGeneration").GetUInt64() != fontGeneration ||
                !JsonElement.DeepEquals(reply.GetProperty("selection"), JsonSerializer.SerializeToElement(selection, WireJson)))
                throw new InvalidDataException("Native 서식 조회의 문서 위치가 일치하지 않습니다.");

            NativeSelectionFormatting formatting = Read<NativeSelectionFormatting>(reply.GetProperty("formatting"));
            if (formatting.Bold is not { Kind: "uniform" or "mixed" } ||
                formatting.FontSize is not ({ Kind: "uniform", Value: > 0 } or { Kind: "mixed" }) ||
                !Enum.TryParse(formatting.Alignment, ignoreCase: true, out NativeParagraphAlignment alignment) ||
                !Enum.IsDefined(alignment))
                throw new InvalidDataException("Native 서식 조회 결과가 올바르지 않습니다.");

            return formatting;
        }, cancellation);

    public Task<EditResult> FormatCharactersAsync(DocumentTarget expected, bool? bold, int? fontSize,
        CancellationToken cancellation) =>
        ApplyFormattingCommand(expected, new { command = "formatCharacters", bold, fontSize }, cancellation);

    public Task<EditResult> FormatCharacterAppearanceAsync(DocumentTarget expected, string? fontName, string? textColor,
        CancellationToken cancellation) =>
        ApplyFormattingCommand(expected, new { command = "formatCharacters", fontName, textColor }, cancellation);

    public Task<string[]> GetSystemFontFamiliesAsync(DocumentTarget expected, CancellationToken cancellation) =>
        Dispatch(token =>
        {
            JsonElement reply = Submit(EnsureWorker(), new { op = "systemFontFamilies", target = expected }, token);
            if (reply.GetProperty("kind").GetString() != "systemFontFamilies" ||
                Read<DocumentTarget>(reply.GetProperty("target")) != expected)
                throw new InvalidDataException("Native 글꼴 목록의 문서가 일치하지 않습니다.");

            return Read<string[]>(reply.GetProperty("families"));
        }, cancellation);

    public Task<EditResult> SetParagraphAlignmentAsync(DocumentTarget expected, NativeParagraphAlignment alignment,
        CancellationToken cancellation) =>
        ApplyFormattingCommand(expected,
            new { command = "setParagraphAlignment", alignment = alignment.ToString().ToLowerInvariant() },
            cancellation);

    private Task<EditResult> ApplyFormattingCommand(DocumentTarget expected, object command,
        CancellationToken cancellation) =>
        Dispatch(token => AdoptEditOutcome(expected, SubmitEditCommand(expected, command, token)), cancellation);
}
