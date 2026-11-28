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
/// Other processes can still change the path between the check and replacement.
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

                // Publication has happened. Finish verification even if the
                // caller cancels while the OS is replacing the destination.
                try
                {
                    await RequireExpectedAsync(destination, fingerprint, CancellationToken.None)
                        .ConfigureAwait(false);
                }
                catch (Exception error) when (error is IOException or UnauthorizedAccessException)
                {
                    throw new DocumentFilePublicationException(destination, true, backup, error);
                }

                var retainedBackup = backup is not null && TryDelete(backup) is not null ? backup : null;
                return new DocumentFileReceipt(destination, fingerprint, retainedBackup);
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
        if ((attributes.Value & (FileAttributes.Directory | FileAttributes.ReparsePoint | FileAttributes.Device)) != 0)
            throw new IOException($"The destination must be a regular file: {path}");

        try
        {
            await using var file = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.Read,
                64 * 1024, FileOptions.Asynchronous | FileOptions.SequentialScan);
            var length = file.Length;
            var lastWrite = File.GetLastWriteTimeUtc(file.SafeFileHandle);
            if (expectedLength is long expected && length != expected)
                throw new DocumentFileConflictException(path);
            var hash = await SHA256.HashDataAsync(file, cancellationToken).ConfigureAwait(false);
            if (length != file.Length || lastWrite != File.GetLastWriteTimeUtc(file.SafeFileHandle))
                throw new DocumentFileConflictException(path);
            return new DocumentFileFingerprint(length, Convert.ToHexString(hash));
        }
        catch (FileNotFoundException)
        {
            return null;
        }
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
