using System.Text;

namespace Rhwp.WinUI;

internal static class PageEditText
{
    private const int NativeContextScalars = 4096;

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

    public static IReadOnlyList<ImeReplacement> Differences(string original, string current)
    {
        ImeReplacement? changed = Difference(original, current);
        if (changed is null)
            return [];
        if (changed.Range.Start == changed.Range.End || changed.Text.Length == 0)
            return [changed];

        int start = (int)changed.Range.Start;
        string oldText = original[start..(int)changed.Range.End];
        string newText = changed.Text;
        var (before, beforeOffsets) = Scalars(oldText, NativeContextScalars);
        // The final text can contain both a full old context and a full insertion.
        var (after, afterOffsets) = Scalars(newText, NativeContextScalars * 2);
        var common = new ushort[before.Length + 1, after.Length + 1];
        for (int oldIndex = before.Length - 1; oldIndex >= 0; oldIndex--)
        {
            for (int newIndex = after.Length - 1; newIndex >= 0; newIndex--)
            {
                common[oldIndex, newIndex] = before[oldIndex] == after[newIndex]
                    ? (ushort)(common[oldIndex + 1, newIndex + 1] + 1)
                    : Math.Max(common[oldIndex + 1, newIndex], common[oldIndex, newIndex + 1]);
            }
        }

        // Retained scalars separate replacement ranges, so their native
        // character styles and other per-character data are never replaced.
        var changes = new List<ImeReplacement>();
        int oldOffset = 0;
        int newOffset = 0;
        while (oldOffset < before.Length || newOffset < after.Length)
        {
            if (Matches())
            {
                oldOffset++;
                newOffset++;
                continue;
            }

            int oldStart = oldOffset;
            int newStart = newOffset;
            do
            {
                if (newOffset == after.Length ||
                    oldOffset < before.Length && common[oldOffset + 1, newOffset] >= common[oldOffset, newOffset + 1])
                    oldOffset++;
                else
                    newOffset++;
            } while ((oldOffset < before.Length || newOffset < after.Length) && !Matches());

            changes.Add(new ImeReplacement(
                new Utf16Range((uint)(start + beforeOffsets[oldStart]), (uint)(start + beforeOffsets[oldOffset])),
                newText[afterOffsets[newStart]..afterOffsets[newOffset]]));
        }
        return changes;

        bool Matches() =>
            oldOffset < before.Length && newOffset < after.Length && before[oldOffset] == after[newOffset];
    }

    private static (Rune[] Values, int[] Offsets) Scalars(string text, int limit)
    {
        if (text.Length > limit * 2)
            throw new ArgumentException("문단 편집 범위가 너무 큽니다.", nameof(text));
        var values = new List<Rune>(text.Length);
        var offsets = new List<int>(text.Length + 1);
        for (int offset = 0; offset < text.Length;)
        {
            if (values.Count == limit)
                throw new ArgumentException("문단 편집 범위가 너무 큽니다.", nameof(text));
            Rune value = Rune.GetRuneAt(text, offset);
            values.Add(value);
            offsets.Add(offset);
            offset += value.Utf16SequenceLength;
        }
        offsets.Add(text.Length);
        return (values.ToArray(), offsets.ToArray());
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
