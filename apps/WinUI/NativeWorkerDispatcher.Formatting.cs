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

internal sealed partial class NativeWorkerDispatcher
{
    private bool? basicTextFormatting;

    public Task<bool> SupportsBasicTextFormattingAsync(CancellationToken cancellation) =>
        Dispatch(_ =>
        {
            if (basicTextFormatting is bool supported)
                return supported;

            using var report = ReadSuccess(RhwpNative.SessionCapabilities());
            supported = HasCapability(report.RootElement, "basicTextFormatting");
            basicTextFormatting = supported;
            return supported;
        }, cancellation);

    public Task<EditResult> FormatCharactersAsync(DocumentTarget expected, bool? bold, int? fontSize,
        CancellationToken cancellation) =>
        ApplyEdit(expected, new
        {
            op = "executeCommand", target = expected,
            command = new { command = "formatCharacters", bold, fontSize },
        }, cancellation);

    public Task<EditResult> SetParagraphAlignmentAsync(DocumentTarget expected, NativeParagraphAlignment alignment,
        CancellationToken cancellation) =>
        ApplyEdit(expected, new
        {
            op = "executeCommand", target = expected,
            command = new { command = "setParagraphAlignment", alignment = alignment.ToString().ToLowerInvariant() },
        }, cancellation);
}
