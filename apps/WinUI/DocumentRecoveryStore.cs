using System.Buffers.Binary;
using System.Security.Cryptography;
using System.Text.Json;

namespace Rhwp.WinUI;

/// <summary>
/// A per-user checkpoint directory. An exclusive lease protects each live
/// document's checkpoint from another window or process.
/// </summary>
internal sealed class DocumentRecoveryStore(string directory)
{
    private const string Suffix = ".rhwp-recovery";
    private const string StagedSuffix = ".tmp";
    private const int MaxCheckpoints = 32;
    private const long MaxDirectoryBytes = 256L * 1024 * 1024;
    private string DirectoryPath { get; } = directory;

    public Task<RecoveryLease> CreateAsync() => Task.Run(() =>
    {
        EnsureDirectory(create: true);
        string id = Guid.NewGuid().ToString("N");
        return OpenLease(id) ?? throw new IOException("복구 사본을 예약할 수 없습니다.");
    });

    public Task<RecoveryCandidate?> TryClaimLatestAsync() => Task.Run<RecoveryCandidate?>(() =>
    {
        if (!EnsureDirectory(create: false))
            return null;

        foreach (string id in CheckpointIds().OrderByDescending(id => File.GetLastWriteTimeUtc(ReadPath(id))))
        {
            RecoveryLease? lease = OpenLease(id);
            if (lease is null)
                continue;

            try
            {
                return new RecoveryCandidate(lease, lease.ReadHeader());
            }
            catch (Exception error) when (error is IOException or InvalidDataException or UnauthorizedAccessException or JsonException)
            {
                // Leave unreadable files untouched and try another checkpoint.
                lease.Dispose();
            }
        }
        return null;
    });

    private bool EnsureDirectory(bool create)
    {
        if (create)
            Directory.CreateDirectory(DirectoryPath);
        else if (!Directory.Exists(DirectoryPath))
            return false;

        if ((File.GetAttributes(DirectoryPath) & (FileAttributes.Directory | FileAttributes.ReparsePoint))
            != FileAttributes.Directory)
        {
            throw new IOException("복구 위치가 일반 폴더가 아닙니다.");
        }

        return true;
    }

    private string[] CheckpointIds()
    {
        var ids = new HashSet<string>(StringComparer.Ordinal);
        foreach (string path in Directory.EnumerateFiles(DirectoryPath, "*" + Suffix + "*"))
        {
            string name = Path.GetFileName(path);
            if (name.Length != 32 + Suffix.Length && name.Length != 32 + Suffix.Length + StagedSuffix.Length)
                continue;
            if (name[32..] != Suffix && name[32..] != Suffix + StagedSuffix)
                continue;
            if (Guid.TryParseExact(name[..32], "N", out _))
                ids.Add(name[..32]);
        }
        return ids.ToArray();
    }

    private string CheckpointPath(string id) => Path.Combine(DirectoryPath, id + Suffix);

    private string ReadPath(string id)
    {
        string path = CheckpointPath(id);
        return File.Exists(path) ? path : path + StagedSuffix;
    }

    private RecoveryLease? OpenLease(string id)
    {
        string path = CheckpointPath(id) + ".lock";
        try
        {
            var lease = new FileStream(path, FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None,
                bufferSize: 1, FileOptions.DeleteOnClose);
            return new RecoveryLease(this, id, lease);
        }
        catch (IOException)
        {
            // Another instance still owns this checkpoint.
            return null;
        }
    }

    private void CheckBudget(string id, long byteLength)
    {
        string[] ids = CheckpointIds();
        if (ids.Length >= MaxCheckpoints && !ids.Contains(id))
            throw new IOException("복구 사본 개수 한도에 도달했습니다.");

        long total = byteLength;
        foreach (string other in ids)
        {
            if (other == id)
                continue;
            string path = CheckpointPath(other);
            var published = new FileInfo(path);
            var staged = new FileInfo(path + StagedSuffix);
            if (published.Exists)
                total = checked(total + published.Length);
            if (staged.Exists)
                total = checked(total + staged.Length);
            if (total > MaxDirectoryBytes)
                throw new IOException("복구 사본을 저장할 공간이 부족합니다.");
        }
    }

    internal sealed class RecoveryLease(DocumentRecoveryStore store, string id, FileStream lease) : IDisposable
    {
        private static ReadOnlySpan<byte> Magic => "RHWP-R1\n"u8;
        private const int PrefixBytes = 12;
        private const int MaxHeaderBytes = 64 * 1024;
        private const int MaxPayloadBytes = 128 * 1024 * 1024;
        private static readonly JsonSerializerOptions JsonOptions = new(JsonSerializerDefaults.Web);

        public Task SaveAsync(ulong revision, DocumentFormat format, string? name, byte[] bytes) =>
            Task.Run(() => Save(revision, format, name, bytes));

        private void Save(ulong revision, DocumentFormat format, string? name, byte[] bytes)
        {
            if (bytes.Length is 0 or > MaxPayloadBytes)
                throw new InvalidDataException("복구 사본의 크기가 허용 범위를 벗어났습니다.");

            string digest = Convert.ToHexString(SHA256.HashData(bytes)).ToLowerInvariant();
            string formatName = format == DocumentFormat.Hwp ? "hwp" : "hwpx";
            var metadata = new RecoveryHeader(1, formatName, revision, DisplayName(name), bytes.Length, digest,
                new RecoveryReport(0));
            byte[] json = JsonSerializer.SerializeToUtf8Bytes(metadata, JsonOptions);
            if (json.Length is 0 or > MaxHeaderBytes)
                throw new InvalidDataException("복구 사본 헤더의 크기가 허용 범위를 벗어났습니다.");

            using var budget = new FileStream(Path.Combine(store.DirectoryPath, "quota.lock"), FileMode.OpenOrCreate,
                FileAccess.ReadWrite, FileShare.None, bufferSize: 1);
            store.CheckBudget(id, PrefixBytes + (long)json.Length + bytes.Length);

            string path = store.CheckpointPath(id);
            string staged = path + StagedSuffix;
            DeleteRegularFile(staged);
            try
            {
                var options = new FileStreamOptions
                {
                    Mode = FileMode.CreateNew,
                    Access = FileAccess.Write,
                    Share = FileShare.None,
                };
                if (!OperatingSystem.IsWindows())
                    options.UnixCreateMode = UnixFileMode.UserRead | UnixFileMode.UserWrite;

                using (var file = new FileStream(staged, options))
                {
                    Span<byte> prefix = stackalloc byte[PrefixBytes];
                    Magic.CopyTo(prefix);
                    BinaryPrimitives.WriteInt32LittleEndian(prefix[Magic.Length..], json.Length);
                    file.Write(prefix);
                    file.Write(json);
                    file.Write(bytes);
                    file.Flush(flushToDisk: true);
                }

                // Both paths are adjacent on the same volume and under this
                // lease. Readers can only see the complete old or new file.
                File.Move(staged, path, overwrite: true);
            }
            catch
            {
                // A verified staged file can still be offered if the process
                // died before publication and there was no prior checkpoint.
                // An ordinary caught failure keeps only the last committed one.
                try { DeleteRegularFile(staged); }
                catch (IOException) { }
                catch (UnauthorizedAccessException) { }
                throw;
            }
        }

        internal RecoveryHeader ReadHeader()
        {
            using FileStream file = OpenCheckpoint();
            return ReadHeader(file);
        }

        public Task<byte[]> LoadAsync(RecoveryHeader expected) => Task.Run(() =>
        {
            using FileStream file = OpenCheckpoint();
            RecoveryHeader metadata = ReadHeader(file);
            if (metadata != expected)
                throw new IOException("확인한 후 복구 사본이 바뀌었습니다.");

            byte[] bytes = new byte[metadata.ByteLength];
            file.ReadExactly(bytes);
            string digest = Convert.ToHexString(SHA256.HashData(bytes));
            if (!digest.Equals(metadata.Digest, StringComparison.OrdinalIgnoreCase))
                throw new InvalidDataException("복구 사본 내용이 손상되었습니다.");
            return bytes;
        });

        public Task DeleteAsync() => Task.Run(() =>
        {
            string path = store.CheckpointPath(id);
            DeleteRegularFile(path + StagedSuffix);
            DeleteRegularFile(path);
        });

        public void Dispose() => lease.Dispose();

        private FileStream OpenCheckpoint()
        {
            string path = store.ReadPath(id);
            if ((File.GetAttributes(path) & (FileAttributes.Directory | FileAttributes.ReparsePoint)) != 0)
                throw new IOException("복구 사본이 일반 파일이 아닙니다.");
            return new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.Read);
        }

        private static RecoveryHeader ReadHeader(FileStream file)
        {
            if (file.Length < PrefixBytes || file.Length > PrefixBytes + (long)MaxHeaderBytes + MaxPayloadBytes)
                throw new InvalidDataException("복구 사본의 크기가 올바르지 않습니다.");

            Span<byte> prefix = stackalloc byte[PrefixBytes];
            file.ReadExactly(prefix);
            int headerLength = BinaryPrimitives.ReadInt32LittleEndian(prefix[Magic.Length..]);
            if (!prefix[..Magic.Length].SequenceEqual(Magic) || headerLength is <= 0 or > MaxHeaderBytes)
                throw new InvalidDataException("복구 사본 형식이 올바르지 않습니다.");

            byte[] json = new byte[headerLength];
            file.ReadExactly(json);
            var metadata = JsonSerializer.Deserialize<RecoveryHeader>(json, JsonOptions);
            if (metadata is not { Version: 1, Report.Count: 0, Digest.Length: 64 } ||
                metadata.Format is not ("hwp" or "hwpx") ||
                metadata.ByteLength is <= 0 or > MaxPayloadBytes ||
                file.Length != PrefixBytes + (long)headerLength + metadata.ByteLength)
            {
                throw new InvalidDataException("복구 사본의 기록이 올바르지 않습니다.");
            }
            return metadata;
        }

        private static void DeleteRegularFile(string path)
        {
            if (File.Exists(path))
                File.Delete(path);
        }
    }

    internal static string DisplayName(string? name) =>
        !string.IsNullOrWhiteSpace(name) && name.Length <= 255 && name.IndexOfAny(Path.GetInvalidFileNameChars()) < 0
            ? name
            : "복구 문서.hwpx";
}

internal sealed record RecoveryReport(int Count);

internal sealed record RecoveryHeader(int Version, string Format, ulong Revision, string Name, int ByteLength,
    string Digest, RecoveryReport Report);

internal sealed record RecoveryCandidate(DocumentRecoveryStore.RecoveryLease Lease, RecoveryHeader Header);
