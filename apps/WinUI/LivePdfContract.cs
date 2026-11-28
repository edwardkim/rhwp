using System.Buffers.Text;
using System.Text.Json;

namespace Rhwp.WinUI;

internal sealed record LivePdfLimits(int MaxOutputBytes);

internal static class LivePdfContract
{
    private const int MaxUiOutputBytes = 32 * 1024 * 1024;

    public static LivePdfLimits? Limits(JsonElement capabilities)
    {
        if (!capabilities.TryGetProperty("livePdfExport", out JsonElement available) ||
            available.ValueKind != JsonValueKind.True ||
            !capabilities.TryGetProperty("maxLivePdfPages", out JsonElement pages) ||
            pages.ValueKind != JsonValueKind.Number || !pages.TryGetInt32(out int maxPages) || maxPages < 1 ||
            !capabilities.TryGetProperty("maxLivePdfBytes", out JsonElement bytes) ||
            bytes.ValueKind != JsonValueKind.Number || !bytes.TryGetInt32(out int maxBytes) || maxBytes < 1)
            return null;

        return new LivePdfLimits(Math.Min(maxBytes, MaxUiOutputBytes));
    }

    public static int ValidateReceipt(JsonElement receipt, PageInfo expected, int maxOutputBytes)
    {
        JsonElement target = receipt.GetProperty("target");
        int length = receipt.GetProperty("byteLength").GetInt32();
        if (receipt.GetProperty("kind").GetString() != "exportedPdf" ||
            target.GetProperty("documentId").GetUInt64() != expected.Target.DocumentId ||
            target.GetProperty("revision").GetUInt64() != expected.Target.Revision ||
            receipt.GetProperty("fontGeneration").GetUInt64() != expected.FontGeneration ||
            receipt.GetProperty("firstPage").GetUInt32() != expected.Page ||
            receipt.GetProperty("pageCount").GetUInt32() != 1 ||
            length <= 0 || length > maxOutputBytes)
        {
            throw new InvalidDataException("PDF 응답이 요청한 문서 버전, 쪽 또는 크기와 일치하지 않습니다.");
        }

        return length;
    }

    public static void ValidateBytes(ReadOnlySpan<byte> bytes)
    {
        const int headerLength = 8;
        ReadOnlySpan<byte> trailer = TrimEnd(bytes);
        if (trailer.Length <= headerLength || !bytes.StartsWith("%PDF-"u8) ||
            !IsDigit(bytes[5]) || bytes[6] != '.' || !IsDigit(bytes[7]) ||
            !trailer.EndsWith("%%EOF"u8))
            throw new InvalidDataException("PDF 헤더 또는 끝 표식이 올바르지 않습니다.");

        // Check the final cross-reference offset before publishing. The
        // native Skia writer supplies the PDF structure itself.
        ReadOnlySpan<byte> beforeEof = trailer[..^5];
        int marker = beforeEof.LastIndexOf("startxref"u8);
        if (marker < headerLength)
            throw new InvalidDataException("PDF 교차 참조 위치가 없습니다.");

        ReadOnlySpan<byte> number = TrimEnd(beforeEof[(marker + 9)..]);
        while (!number.IsEmpty && IsWhitespace(number[0]))
            number = number[1..];
        if (!Utf8Parser.TryParse(number, out int offset, out int consumed) ||
            consumed != number.Length || offset < headerLength || offset >= marker)
            throw new InvalidDataException("PDF 교차 참조 위치가 올바르지 않습니다.");
    }

    private static ReadOnlySpan<byte> TrimEnd(ReadOnlySpan<byte> bytes)
    {
        while (!bytes.IsEmpty && IsWhitespace(bytes[^1]))
            bytes = bytes[..^1];
        return bytes;
    }

    private static bool IsDigit(byte value) => value is >= (byte)'0' and <= (byte)'9';

    private static bool IsWhitespace(byte value) => value is 0 or 9 or 10 or 12 or 13 or 32;
}
