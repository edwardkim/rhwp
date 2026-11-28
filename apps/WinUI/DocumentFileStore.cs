using System;
using System.IO;
using System.Security.Cryptography;
using System.Threading;
using System.Threading.Tasks;

namespace Rhwp.WinUI;

internal sealed record DocumentFileFingerprint(long ByteLength, string Sha256)
{
    public static DocumentFileFingerprint FromBytes(ReadOnlySpan<byte> bytes) =>
        new(bytes.Length, Convert.ToHexString(SHA256.HashData(bytes)));
}

internal sealed record DocumentFileReceipt(string Path, DocumentFileFingerprint Fingerprint,
    string? RetainedBackupPath);

internal sealed class DocumentFileConflictException(string path, Exception? inner = null)
    : IOException($"The destination changed since it was selected: {path}", inner)
{
}

internal sealed class DocumentFilePublicationException(
    string path, bool publicationReturnedSuccess, string? possibleBackupPath, Exception inner)
    : IOException(publicationReturnedSuccess
        ? $"The file was published but could not be verified: {path}"
        : $"The file replacement failed and may need recovery: {path}", inner)
{
    public bool PublicationReturnedSuccess { get; } = publicationReturnedSuccess;
    public string? PossibleBackupPath { get; } = possibleBackupPath;
}

/// <summary>
/// Publishes prepared document bytes. The caller keeps the native save token
/// and acknowledges it after receiving a verified receipt.
/// </summary>
/// <remarks>
/// The fingerprint identifies the file's contents by length and SHA-256.
/// Existing-file checks detect observed changes and serialize app publications.
/// Windows keeps a read handle open through replacement, blocking writes to
/// that file while still allowing renames. The displaced file is verified
/// before its backup can be deleted, since another process can still rename
/// a different file into the path.
/// A flush persists staged data subject to the filesystem's guarantees. This
/// layer does not flush the parent directory or manage crash recovery.
/// </remarks>
internal static class DocumentFileStore
{
    private static readonly SemaphoreSlim PublicationGate = new(1, 1);

    public static Task<DocumentFileFingerprint?> ReadFingerprintAsync(string path,
        CancellationToken cancellationToken = default)
    {
        var destination = FullFilePath(path);
        return Task.Run(() => ReadFingerprintCoreAsync(destination, null, cancellationToken),
            cancellationToken);
    }

    /// <summary>
    /// Pass null for exclusive creation. For replacement, pass the fingerprint
    /// of the bytes read on open or of the destination the user approved.
    /// Cancellation is accepted before the filesystem publication step.
    /// </summary>
    public static Task<DocumentFileReceipt> PublishAsync(string path, ReadOnlyMemory<byte> bytes,
        DocumentFileFingerprint? expectedDestination, CancellationToken cancellationToken = default)
    {
        var destination = FullFilePath(path);
        return Task.Run(() => PublishCoreAsync(destination, bytes, expectedDestination,
            cancellationToken), cancellationToken);
    }

    private static async Task<DocumentFileReceipt> PublishCoreAsync(string destination,
        ReadOnlyMemory<byte> bytes, DocumentFileFingerprint? expected, CancellationToken cancellationToken)
    {
        await RequireExpectedAsync(destination, expected, cancellationToken).ConfigureAwait(false);
        var directory = Path.GetDirectoryName(destination)!;
        var staged = Path.Combine(directory, $".rhwp-save-{Guid.NewGuid():N}.tmp");
        var ownsStagingFile = false;
        try
        {
            var stagingOptions = new FileStreamOptions
            {
                Mode = FileMode.CreateNew,
                Access = FileAccess.ReadWrite,
                Share = FileShare.None,
                BufferSize = 64 * 1024,
                Options = FileOptions.Asynchronous,
            };
            if (!OperatingSystem.IsWindows())
                stagingOptions.UnixCreateMode = UnixFileMode.UserRead | UnixFileMode.UserWrite;

            DocumentFileFingerprint fingerprint;
            await using (var file = new FileStream(staged, stagingOptions))
            {
                ownsStagingFile = true;
                await file.WriteAsync(bytes, cancellationToken).ConfigureAwait(false);
                cancellationToken.ThrowIfCancellationRequested();
                file.Flush(flushToDisk: true);
                file.Position = 0;
                var hash = await SHA256.HashDataAsync(file, cancellationToken).ConfigureAwait(false);
                fingerprint = new DocumentFileFingerprint(file.Length, Convert.ToHexString(hash));
            }

            await PublicationGate.WaitAsync(cancellationToken).ConfigureAwait(false);
            try
            {
                await using var readLock = expected is not null && OperatingSystem.IsWindows()
                    ? await LockReplacementDestinationAsync(destination, expected, cancellationToken)
                        .ConfigureAwait(false)
                    : null;
                if (readLock is null)
                    await RequireExpectedAsync(destination, expected, cancellationToken).ConfigureAwait(false);
                cancellationToken.ThrowIfCancellationRequested();

                string? backup = null;
                if (expected is null)
                {
                    try
                    {
                        File.Move(staged, destination, overwrite: false);
                    }
                    catch (IOException error) when (ReadAttributesOrMissing(destination).HasValue)
                    {
                        throw new DocumentFileConflictException(destination, error);
                    }
                }
                else
                {
                    // ReplaceFile can fail after moving the old destination.
                    // Keep that original in a sibling backup on any failure.
                    backup = Path.Combine(directory, $".rhwp-save-{Guid.NewGuid():N}.bak");
                    try
                    {
                        File.Replace(staged, destination, backup);
                    }
                    catch (Exception error) when (error is IOException or UnauthorizedAccessException)
                    {
                        throw new DocumentFilePublicationException(destination, false, backup, error);
                    }
                }
                ownsStagingFile = false;

                return await VerifyPublicationAsync(destination, fingerprint, backup, expected).ConfigureAwait(false);
            }
            finally
            {
                PublicationGate.Release();
            }
        }
        catch (Exception error)
        {
            if (ownsStagingFile && TryDelete(staged) is { } cleanupError)
            {
                error.Data["RetainedStagingPath"] = staged;
                error.Data["StagingCleanupError"] = cleanupError;
            }
            throw;
        }
    }

    internal static async Task<DocumentFileReceipt> VerifyPublicationAsync(string destination,
        DocumentFileFingerprint fingerprint, string? backup, DocumentFileFingerprint? expected)
    {
        // Publication has happened. Verification is independent of cancellation,
        // and a late conflicting writer's displaced bytes remain in the backup.
        try
        {
            await RequireExpectedAsync(destination, fingerprint, CancellationToken.None).ConfigureAwait(false);
            if (backup is not null && expected is not null)
                await RequireExpectedAsync(backup, expected, CancellationToken.None).ConfigureAwait(false);
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException)
        {
            throw new DocumentFilePublicationException(destination, true, backup, error);
        }

        var retainedBackup = backup is not null && TryDelete(backup) is not null ? backup : null;
        return new DocumentFileReceipt(destination, fingerprint, retainedBackup);
    }

    private static async Task RequireExpectedAsync(string path, DocumentFileFingerprint? expected,
        CancellationToken cancellationToken)
    {
        cancellationToken.ThrowIfCancellationRequested();
        if (expected is null)
        {
            if (ReadAttributesOrMissing(path).HasValue)
                throw new DocumentFileConflictException(path);
        }
        else if (await ReadFingerprintCoreAsync(path, expected.ByteLength, cancellationToken)
                     .ConfigureAwait(false) != expected)
        {
            throw new DocumentFileConflictException(path);
        }
    }

    private static async Task<DocumentFileFingerprint?> ReadFingerprintCoreAsync(string path,
        long? expectedLength, CancellationToken cancellationToken)
    {
        cancellationToken.ThrowIfCancellationRequested();
        var attributes = ReadAttributesOrMissing(path);
        if (attributes is null)
            return null;
        EnsureRegularFile(path, attributes.Value);

        try
        {
            await using var file = OpenForFingerprint(path, FileShare.Read);
            return await FingerprintAsync(file, path, expectedLength, cancellationToken).ConfigureAwait(false);
        }
        catch (FileNotFoundException)
        {
            return null;
        }
    }

    private static async Task<FileStream> LockReplacementDestinationAsync(string path,
        DocumentFileFingerprint expected, CancellationToken cancellationToken)
    {
        var attributes = ReadAttributesOrMissing(path);
        if (attributes is null)
            throw new DocumentFileConflictException(path);
        EnsureRegularFile(path, attributes.Value);
        FileStream file;
        try
        {
            file = OpenForFingerprint(path, FileShare.Read | FileShare.Delete);
        }
        catch (FileNotFoundException error)
        {
            throw new DocumentFileConflictException(path, error);
        }

        try
        {
            if (await FingerprintAsync(file, path, expected.ByteLength, cancellationToken).ConfigureAwait(false) != expected)
                throw new DocumentFileConflictException(path);
            return file;
        }
        catch
        {
            await file.DisposeAsync().ConfigureAwait(false);
            throw;
        }
    }

    private static void EnsureRegularFile(string path, FileAttributes attributes)
    {
        if ((attributes & (FileAttributes.Directory | FileAttributes.ReparsePoint | FileAttributes.Device)) != 0)
            throw new IOException($"The destination must be a regular file: {path}");
    }

    private static FileStream OpenForFingerprint(string path, FileShare share) =>
        new(path, FileMode.Open, FileAccess.Read, share, 64 * 1024,
            FileOptions.Asynchronous | FileOptions.SequentialScan);

    private static async Task<DocumentFileFingerprint> FingerprintAsync(FileStream file, string path,
        long? expectedLength, CancellationToken cancellationToken)
    {
        var length = file.Length;
        var lastWrite = File.GetLastWriteTimeUtc(file.SafeFileHandle);
        if (expectedLength is long expected && length != expected)
            throw new DocumentFileConflictException(path);
        var hash = await SHA256.HashDataAsync(file, cancellationToken).ConfigureAwait(false);
        if (length != file.Length || lastWrite != File.GetLastWriteTimeUtc(file.SafeFileHandle))
            throw new DocumentFileConflictException(path);
        return new DocumentFileFingerprint(length, Convert.ToHexString(hash));
    }

    private static FileAttributes? ReadAttributesOrMissing(string path)
    {
        try
        {
            return File.GetAttributes(path);
        }
        catch (FileNotFoundException)
        {
            return null;
        }
    }

    private static string FullFilePath(string path)
    {
        var fullPath = Path.GetFullPath(path);
        if (Path.GetDirectoryName(fullPath) is null || Path.GetFileName(fullPath).Length == 0)
            throw new ArgumentException("Choose a document filename.", nameof(path));
        return fullPath;
    }

    private static Exception? TryDelete(string path)
    {
        try
        {
            File.Delete(path);
            return null;
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException)
        {
            return error;
        }
    }
}
