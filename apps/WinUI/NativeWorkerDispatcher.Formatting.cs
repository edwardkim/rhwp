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

internal enum NativeCharacterDecoration
{
    Italic,
    Underline,
    Strikethrough,
}

internal sealed record NativeFormattingCapabilities(bool Edit, bool Inspect, bool FontAndColor, bool SystemFonts,
    bool ParagraphStyles, bool Decorations, bool Highlight);
internal sealed record NativeFormattingValue<T>(string Kind, T Value);
internal sealed record NativeSelectionFormatting(NativeFormattingValue<bool> Bold,
    NativeFormattingValue<int> FontSize, string Alignment,
    NativeFormattingValue<string?>[]? FontNames, NativeFormattingValue<string?>? TextColor,
    NativeFormattingValue<bool>? Italic, NativeFormattingValue<bool>? Underline, NativeFormattingValue<bool>? Strikethrough,
    NativeFormattingValue<string?>? ShadeColor);

internal sealed record NativeParagraphStyle(byte Id, string Name)
{
    public string DisplayName => string.IsNullOrWhiteSpace(Name) ? $"스타일 {Id}" : Name;
}

internal sealed record NativeParagraphStyleCatalog(byte? CurrentStyleId, NativeParagraphStyle[] Styles);

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
            HasCapability(report.RootElement, "systemFontFamilies"),
            HasCapability(report.RootElement, "paragraphStyles"),
            HasCapability(report.RootElement, "characterDecorationFormatting"),
            HasCapability(report.RootElement, "characterHighlightFormatting"));
        return formattingCapabilities;
    }

    public Task<NativeSelectionFormatting?> GetSelectionFormattingAsync(DocumentTarget expected, ulong fontGeneration,
        TextSelection selection, CancellationToken cancellation) =>
        Dispatch<NativeSelectionFormatting?>(token =>
        {
            NativeFormattingCapabilities capabilities = ReadFormattingCapabilities();
            if (!capabilities.Inspect)
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
                (capabilities.Decorations && (formatting.Italic is not { Kind: "uniform" or "mixed" } ||
                    formatting.Underline is not { Kind: "uniform" or "mixed" } ||
                    formatting.Strikethrough is not { Kind: "uniform" or "mixed" })) ||
                (capabilities.Highlight && formatting.ShadeColor is not { Kind: "uniform" or "mixed" }) ||
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

    public Task<EditResult> SetCharacterHighlightAsync(DocumentTarget expected, string shadeColor,
        CancellationToken cancellation) =>
        ApplyFormattingCommand(expected, new { command = "formatCharacters", shadeColor }, cancellation);

    public Task<EditResult> FormatCharacterDecorationAsync(DocumentTarget expected, NativeCharacterDecoration decoration,
        bool enabled, CancellationToken cancellation) =>
        ApplyFormattingCommand(expected, new
        {
            command = "formatCharacters",
            italic = decoration == NativeCharacterDecoration.Italic ? enabled : (bool?)null,
            underline = decoration == NativeCharacterDecoration.Underline ? enabled : (bool?)null,
            strikethrough = decoration == NativeCharacterDecoration.Strikethrough ? enabled : (bool?)null,
        }, cancellation);

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

    public Task<NativeParagraphStyleCatalog> GetParagraphStylesAsync(DocumentTarget expected, TextSelection selection,
        CancellationToken cancellation) => Dispatch(token =>
    {
        JsonElement reply = Submit(EnsureWorker(), new { op = "paragraphStyles", target = expected, selection }, token);
        if (reply.GetProperty("kind").GetString() != "paragraphStyles" ||
            Read<DocumentTarget>(reply.GetProperty("target")) != expected ||
            !JsonElement.DeepEquals(reply.GetProperty("selection"), JsonSerializer.SerializeToElement(selection, WireJson)))
            throw new InvalidDataException("Native 문단 양식의 문서 위치가 일치하지 않습니다.");

        return Read<NativeParagraphStyleCatalog>(reply);
    }, cancellation);

    public Task<EditResult> ApplyParagraphStyleAsync(DocumentTarget expected, byte styleId,
        CancellationToken cancellation) =>
        ApplyFormattingCommand(expected, new { command = "applyParagraphStyle", styleId }, cancellation);

    private Task<EditResult> ApplyFormattingCommand(DocumentTarget expected, object command,
        CancellationToken cancellation) =>
        Dispatch(token => AdoptEditOutcome(expected, SubmitEditCommand(expected, command, token)), cancellation);
}
