using System.Text;

namespace Rhwp.WinUI;

internal static class PageEditText
{
    public static int Utf16Offset(string text, uint scalarOffset)
    {
        int utf16 = 0;
        foreach (Rune rune in text.EnumerateRunes())
        {
            if (scalarOffset == 0)
                return utf16;
            scalarOffset--;
            utf16 += rune.Utf16SequenceLength;
        }

        return scalarOffset == 0 ? utf16 : throw new ArgumentOutOfRangeException(nameof(scalarOffset));
    }

    public static ImeReplacement? Difference(string original, string current)
    {
        if (original == current)
            return null;

        int start = 0;
        while (start < original.Length && start < current.Length && original[start] == current[start])
            start++;
        if (SplitsSurrogate(original, start) || SplitsSurrogate(current, start))
            start--;

        int originalEnd = original.Length;
        int currentEnd = current.Length;
        while (originalEnd > start && currentEnd > start &&
            original[originalEnd - 1] == current[currentEnd - 1])
        {
            originalEnd--;
            currentEnd--;
        }
        if (SplitsSurrogate(original, originalEnd) || SplitsSurrogate(current, currentEnd))
        {
            originalEnd++;
            currentEnd++;
        }

        return new ImeReplacement(new Utf16Range((uint)start, (uint)originalEnd), current[start..currentEnd]);
    }

    public static Utf16Selection? Selection(string text, int start, int length)
    {
        if (start < 0 || start > text.Length || length < 0 || length > text.Length - start)
            return null;

        int end = start + length;
        return SplitsSurrogate(text, start) || SplitsSurrogate(text, end)
            ? null
            : new Utf16Selection((uint)start, (uint)end);
    }

    private static bool SplitsSurrogate(string text, int offset) =>
        offset > 0 && offset < text.Length && char.IsSurrogatePair(text[offset - 1], text[offset]);
}
