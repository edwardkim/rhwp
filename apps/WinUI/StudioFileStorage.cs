namespace Rhwp.WinUI;

internal sealed record StudioOpenedFile(DocumentFileLocation Location, byte[] Bytes);

internal static class StudioFileStorage
{
    public const int MaxDocumentBytes = 128 * 1024 * 1024;

    public static Task<StudioOpenedFile> OpenAsync(string path, CancellationToken cancellationToken = default)
    {
        string fullPath = Path.GetFullPath(path);
        return Task.Run(async () =>
        {
            cancellationToken.ThrowIfCancellationRequested();
            if ((File.GetAttributes(fullPath) & (FileAttributes.Directory | FileAttributes.Device)) != 0)
                throw new IOException("문서는 일반 파일이어야 합니다.");

            await using var file = new FileStream(fullPath, FileMode.Open, FileAccess.Read, FileShare.Read,
                64 * 1024, FileOptions.Asynchronous | FileOptions.SequentialScan);
            long length = file.Length;
            ValidateLength(length);
            DateTime lastWrite = File.GetLastWriteTimeUtc(file.SafeFileHandle);
            var bytes = new byte[(int)length];
            await file.ReadExactlyAsync(bytes, cancellationToken).ConfigureAwait(false);
            if (file.Length != length || File.GetLastWriteTimeUtc(file.SafeFileHandle) != lastWrite)
                throw new DocumentFileConflictException(fullPath);

            DocumentFileLocation location = DocumentFileLocation.FromOpenedBytes(fullPath, bytes);
            cancellationToken.ThrowIfCancellationRequested();
            return new StudioOpenedFile(location, bytes);
        }, cancellationToken);
    }

    /// <summary>
    /// Pass the location from OpenAsync for an in-place save, or null to create
    /// a new file. In either case settle the editor revision only after this
    /// method returns a verified disk receipt.
    /// </summary>
    public static Task<DocumentFileReceipt> SaveAsync(string path, DocumentFormat format, ReadOnlyMemory<byte> bytes,
        DocumentFileLocation? original, CancellationToken cancellationToken = default)
    {
        cancellationToken.ThrowIfCancellationRequested();
        string destination = Path.GetFullPath(path);
        ValidateLength(bytes.Length);
        if (DocumentFileLocation.FormatForBytes(destination, bytes.Span) != format)
            throw new InvalidDataException("저장 형식, 파일 확장자와 내보낸 문서 바이트가 일치하지 않습니다.");

        if (original is not null &&
            (!StringComparer.Ordinal.Equals(original.Path, destination) || original.Format != format))
        {
            throw new DocumentFileConflictException(destination);
        }

        return DocumentFileStore.PublishAsync(destination, bytes, original?.Fingerprint, cancellationToken);
    }

    private static void ValidateLength(long length)
    {
        if (length is <= 0 or > MaxDocumentBytes)
            throw new InvalidDataException("문서 크기는 1바이트부터 128MiB까지 지원합니다.");
    }
}
