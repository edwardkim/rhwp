using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text.Json;
using System.Threading;
using Rhwp;

if (args.Length is < 2 or > 3 || (args.Length == 3 && args[2] != "--live-pdf"))
{
    Console.Error.WriteLine("Usage: RhwpNativeSmoke <native-library> <public-leave-request.hwpx> [--live-pdf]");
    return 2;
}

var nativeModule = NativeLibrary.Load(Path.GetFullPath(args[0]));
NativeLibrary.SetDllImportResolver(
    typeof(RhwpNative).Assembly,
    (name, _, _) => name == "rhwp_native_ffi" ? nativeModule : IntPtr.Zero);

Exception? failure = null;
var ownerThread = new Thread(() =>
{
    try
    {
        VerifyDocument(File.ReadAllBytes(args[1]), args.Length == 3);
    }
    catch (Exception error)
    {
        failure = error;
    }
});
ownerThread.Start();
ownerThread.Join();
if (failure is not null)
{
    Console.Error.WriteLine(failure);
    return 1;
}

Console.WriteLine(JsonSerializer.Serialize(new
{
    nativeCSharpSmoke = true,
    livePdf = args.Length == 3,
    os = RuntimeInformation.OSDescription,
    architecture = RuntimeInformation.ProcessArchitecture.ToString(),
}));
return 0;

static void VerifyDocument(byte[] input, bool verifyLivePdf)
{
    using var worker = new Worker();
    var target = Target.FromInfo(worker.Open(input).GetProperty("info"));
    var applicantName = FindCell(worker, target, 1, 3);
    var applicantBirthday = FindCell(worker, target, 1, 5);
    var childBirthday = FindCell(worker, target, 4, 3);
    var originalName = applicantName.GetProperty("text").GetString();

    Worker.Check(worker.Submit(new
    {
        op = "beginEdit", target,
        selection = new { anchor = applicantName.GetProperty("start"), focus = applicantName.GetProperty("end") },
    }));
    target = Edit(worker, target, "replace", "가나다");
    Expect(FindCell(worker, target, 1, 3).GetProperty("text").GetString() == "가나다", "applicant name");
    target = Edit(worker, target, "undo");
    Expect(FindCell(worker, target, 1, 3).GetProperty("text").GetString() == originalName, "name after undo");
    target = Edit(worker, target, "redo");

    var birthdayPoint = applicantBirthday.GetProperty("start");
    Worker.Check(worker.Submit(new
    {
        op = "setSelection", target,
        selection = new { anchor = birthdayPoint, focus = birthdayPoint },
    }));
    var expected = Worker.Check(worker.Submit(new { op = "imeEditContext", target })).GetProperty("context");
    var composed = Worker.Check(worker.Submit(new
    {
        op = "applyImeEdit", target, expected,
        edit = new { kind = "commit", replacement = new { range = new { start = 0, end = 0 }, text = "2000년 1월 2일" } },
    }));
    target = target with { revision = composed.GetProperty("outcome").GetProperty("revision").GetUInt64() };
    VerifyFields(worker, target, childBirthday);

    if (verifyLivePdf)
        VerifyLivePdf(worker, target);

    var directory = Directory.CreateTempSubdirectory("rhwp-csharp-smoke-");
    try
    {
        foreach (string format in new[] { "hwpx", "hwp" })
        {
            var path = Path.Combine(directory.FullName, "검증용." + format);
            Publish(worker, target, format, path);

            using var reader = new Worker();
            var reopened = Target.FromInfo(reader.Open(File.ReadAllBytes(path)).GetProperty("info"));
            VerifyFields(reader, reopened, childBirthday);

            string repeated = Path.Combine(directory.FullName, "다시저장." + format);
            Publish(reader, reopened, format, repeated);
            using var verifier = new Worker();
            var reopenedAgain = Target.FromInfo(verifier.Open(File.ReadAllBytes(repeated)).GetProperty("info"));
            VerifyFields(verifier, reopenedAgain, childBirthday);
        }

        var replacement = Target.FromInfo(worker.Open(input, target).GetProperty("info"));
        Expect(replacement.documentId != target.documentId, "open must replace the document ID");
        var stale = worker.Submit(new { op = "replace", target, text = "stale request" });
        Expect(!stale.GetProperty("ok").GetBoolean()
            && stale.GetProperty("code").GetString() == "invalidArgument", "old-document edit rejection");
        Expect(FindCell(worker, replacement, 1, 3).GetProperty("text").GetString() == originalName,
            "stale request must leave the replacement document alone");
        worker.CheckCancelledOrCompleted(new { op = "accessibility", target = replacement, request = new { } });
    }
    finally
    {
        directory.Delete(recursive: true);
    }
}

static void Publish(Worker worker, Target target, string format, string path)
{
    var prepared = Worker.Check(worker.Submit(new { op = "prepareSave", target, format }));
    var content = Worker.CopyBuffer(prepared);
    Expect(prepared.GetProperty("report").GetProperty("count").GetInt32() == 0, "content loss");
    using (var output = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.None))
    {
        output.Write(content);
        output.Flush(flushToDisk: true);
    }
    var token = prepared.GetProperty("token").GetUInt64();
    Worker.Check(worker.Submit(new { op = "acknowledgeSave", target.documentId, token }));
}

static void VerifyLivePdf(Worker worker, Target target)
{
    var before = Worker.Check(worker.Submit(new { op = "info" })).GetProperty("info");
    Expect(before.GetProperty("isDirty").GetBoolean(), "document must be dirty before live PDF");
    ulong fontGeneration = before.GetProperty("fontGeneration").GetUInt64();
    var result = Worker.Check(worker.Submit(new
    {
        op = "exportPdf", target, fontGeneration, firstPage = 0, pageCount = 1,
        maxOutputBytes = 4 * 1024 * 1024,
    }));
    var pdf = Worker.CopyBuffer(result);
    Expect(result.GetProperty("kind").GetString() == "exportedPdf", "PDF completion kind");
    Expect(Target.FromInfo(result.GetProperty("target")) == target, "PDF target revision");
    Expect(pdf.AsSpan().StartsWith("%PDF-"u8), "PDF header");
    Expect(System.Text.Encoding.ASCII.GetString(pdf.AsSpan(Math.Max(0, pdf.Length - 16)))
        .TrimEnd().EndsWith("%%EOF", StringComparison.Ordinal), "PDF trailer");
    var after = Worker.Check(worker.Submit(new { op = "info" })).GetProperty("info");
    Expect(before.GetRawText() == after.GetRawText(), "PDF leaves editing state intact");
}

static Target Edit(Worker worker, Target target, string op, string? text = null)
{
    var edited = Worker.Check(text is null ? worker.Submit(new { op, target }) : worker.Submit(new { op, target, text }));
    return target with { revision = edited.GetProperty("outcome").GetProperty("revision").GetUInt64() };
}

static JsonElement FindCell(Worker worker, Target target, int tableParagraph, int cell)
{
    var snapshot = Worker.Check(worker.Submit(new { op = "accessibility", target, request = new { } }))
        .GetProperty("snapshot");
    return snapshot.GetProperty("nodes").EnumerateArray().Single(node =>
    {
        var position = node.GetProperty("start");
        var path = position.GetProperty("cellPath");
        return position.GetProperty("section").GetInt32() == 0
            && position.GetProperty("paragraph").GetInt32() == 1
            && path.GetArrayLength() == 2
            && path[0].GetProperty("control").GetInt32() == 0
            && path[0].GetProperty("cell").GetInt32() == 1
            && path[0].GetProperty("paragraph").GetInt32() == tableParagraph
            && path[1].GetProperty("control").GetInt32() == 0
            && path[1].GetProperty("cell").GetInt32() == cell
            && path[1].GetProperty("paragraph").GetInt32() == 0;
    });
}

static void VerifyFields(Worker worker, Target target, JsonElement originalChildBirthday)
{
    Expect(FindCell(worker, target, 1, 3).GetProperty("text").GetString() == "가나다", "saved applicant name");
    Expect(FindCell(worker, target, 1, 5).GetProperty("text").GetString() == "2000년 1월 2일", "saved birthday");
    Expect(FindCell(worker, target, 4, 3).GetProperty("text").GetString()
        == originalChildBirthday.GetProperty("text").GetString(), "child birthday");
}

static void Expect(bool condition, string description)
{
    if (!condition)
        throw new InvalidDataException(description);
}

internal readonly record struct Target(ulong documentId, ulong revision)
{
    public static Target FromInfo(JsonElement info) =>
        new(info.GetProperty("documentId").GetUInt64(), info.GetProperty("revision").GetUInt64());
}

internal sealed class Worker : IDisposable
{
    private static readonly TimeSpan Timeout = TimeSpan.FromSeconds(30);
    private readonly ulong handle = Request(new { op = "workerCreate", config = new { } }).GetProperty("worker").GetUInt64();

    public JsonElement Open(byte[] input, Target? replace = null) =>
        Check(Wait(Request(new { op = "workerOpen", worker = handle, replace }, input)));

    public JsonElement Submit<T>(T command) =>
        Wait(Request(new { op = "workerSubmit", worker = handle, command }));

    public void CheckCancelledOrCompleted<T>(T command)
    {
        var submitted = Request(new { op = "workerSubmit", worker = handle, command });
        var requestId = submitted.GetProperty("requestId").GetUInt64();
        var outcome = Request(new { op = "workerCancel", worker = handle, requestId })
            .GetProperty("outcome").GetString();
        var completion = Check(Wait(submitted)).GetProperty("kind").GetString();
        if ((outcome, completion) is not (("cancelled", "cancelled") or ("tooLate", "accessibility")))
            throw new InvalidDataException($"Unexpected cancellation and completion: {outcome}, {completion}");
    }

    public void Dispose()
    {
        var state = Request(new { op = "workerClose", worker = handle });
        var timer = Stopwatch.StartNew();
        while (!state.GetProperty("isFinished").GetBoolean())
        {
            if (timer.Elapsed >= Timeout)
                throw new TimeoutException("Native worker is still closing.");
            Thread.Sleep(1);
            state = Request(new { op = "workerStatus", worker = handle });
        }
        Request(new { op = "workerRelease", worker = handle });
        var released = Parse(RhwpNative.HostRequest(JsonSerializer.Serialize(new { op = "workerStatus", worker = handle })));
        if (released.GetProperty("ok").GetBoolean() || released.GetProperty("code").GetString() != "invalidArgument")
            throw new InvalidDataException("Native worker handle remained registered after release.");
    }

    private JsonElement Wait(JsonElement submitted)
    {
        var request = submitted.GetProperty("requestId").GetUInt64();
        var timer = Stopwatch.StartNew();
        while (timer.Elapsed < Timeout)
        {
            var result = Request(new { op = "workerPoll", worker = handle }).GetProperty("completion");
            if (result.ValueKind != JsonValueKind.Null)
            {
                if (result.GetProperty("requestId").GetUInt64() != request)
                    throw new InvalidDataException("Unexpected completion for a serial request.");
                return result;
            }
            Thread.Sleep(1);
        }
        throw new TimeoutException("Native request is still pending or running.");
    }

    private static JsonElement Request<T>(T request, byte[]? data = null) =>
        Check(Parse(RhwpNative.HostRequest(JsonSerializer.Serialize(request), data)));

    public static byte[] CopyBuffer(JsonElement response)
    {
        var buffer = response.GetProperty("buffer").GetUInt64();
        try
        {
            var content = new byte[response.GetProperty("byteLength").GetInt32()];
            Check(Parse(RhwpNative.SessionBufferCopy(buffer, content)));
            return content;
        }
        finally
        {
            Check(Parse(RhwpNative.SessionBufferRelease(buffer)));
        }
    }

    public static JsonElement Parse(string response)
    {
        using var document = JsonDocument.Parse(response);
        return document.RootElement.Clone();
    }

    public static JsonElement Check(JsonElement response)
    {
        if (!response.GetProperty("ok").GetBoolean())
            throw new InvalidDataException(response.GetRawText());
        return response;
    }
}
