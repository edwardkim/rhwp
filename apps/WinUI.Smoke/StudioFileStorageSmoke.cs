using Rhwp.WinUI;

internal static class StudioFileStorageSmoke
{
    public static async Task RunAsync(string hwpPath, string hwpxPath)
    {
        string directory = Directory.CreateTempSubdirectory("rhwp-studio-files-").FullName;
        try
        {
            foreach (var (fixture, format) in new[] { (hwpPath, DocumentFormat.Hwp), (hwpxPath, DocumentFormat.Hwpx) })
            {
                string extension = Path.GetExtension(fixture);
                string path = Path.Combine(directory, "original" + extension);
                string copyPath = Path.Combine(directory, "copy" + extension);
                File.Copy(fixture, path);
                StudioOpenedFile opened = await StudioFileStorage.OpenAsync(path);
                Assert(opened.Location.Format == format && opened.Location.Path == path, "opened path and format");
                Assert(opened.Location.Fingerprint == DocumentFileFingerprint.FromBytes(opened.Bytes),
                    "the initial fingerprint identifies the bytes returned to Studio");

                DocumentFileReceipt receipt = await StudioFileStorage.SaveAsync(path, format, opened.Bytes,
                    opened.Location);
                Assert(receipt.Fingerprint == opened.Location.Fingerprint, "in-place save returns verified fingerprint");

                var externalBytes = (byte[])opened.Bytes.Clone();
                externalBytes[^1] ^= 1;
                await File.WriteAllBytesAsync(path, externalBytes);
                await RejectAsync<DocumentFileConflictException>(() =>
                    StudioFileStorage.SaveAsync(path, format, opened.Bytes, opened.Location));
                Assert((await File.ReadAllBytesAsync(path)).AsSpan().SequenceEqual(externalBytes),
                    "the outside edit survives a conflicting in-place save");

                await StudioFileStorage.SaveAsync(copyPath, format, opened.Bytes, original: null);
                StudioOpenedFile copy = await StudioFileStorage.OpenAsync(copyPath);
                Assert(copy.Bytes.AsSpan().SequenceEqual(opened.Bytes), "the new file reopens verbatim");
                await RejectAsync<DocumentFileConflictException>(() =>
                    StudioFileStorage.SaveAsync(copyPath, format, opened.Bytes, opened.Location));
                await RejectAsync<DocumentFileConflictException>(() =>
                    StudioFileStorage.SaveAsync(copyPath, format, opened.Bytes, original: null));
            }

            string invalidPath = Path.Combine(directory, "invalid.hwp");
            StudioOpenedFile hwpx = await StudioFileStorage.OpenAsync(hwpxPath);
            await RejectAsync<InvalidDataException>(() =>
                StudioFileStorage.SaveAsync(invalidPath, DocumentFormat.Hwp, hwpx.Bytes, original: null));
            Assert(!File.Exists(invalidPath), "an export with incompatible format creates no file");
            Console.WriteLine("Studio file storage: HWP/HWPX identity, in-place and new saves, external conflicts passed.");
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    private static async Task RejectAsync<T>(Func<Task> action) where T : Exception
    {
        try
        {
            await action();
        }
        catch (T)
        {
            return;
        }

        throw new InvalidOperationException($"Expected {typeof(T).Name}.");
    }

    private static void Assert(bool condition, string message)
    {
        if (!condition)
            throw new InvalidOperationException(message);
    }
}
