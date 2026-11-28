using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using Rhwp.WinUI;

internal static class LocalAgentSmoke
{
    private const string Literal = "한글 🌱 \" & | % ^ $(no-shell)";
    private static readonly TimeSpan Timeout = TimeSpan.FromSeconds(10);

    public static async Task RunAsync()
    {
        DirectoryInfo workspace = Directory.CreateTempSubdirectory("rhwp-agent-smoke-");
        try
        {
            CheckLauncherResolution(workspace.FullName);
            string prompt = LocalAgentCli.CreatePrompt(Literal, "새 문단\n" + Literal, rewrite: true);
            ProcessStartInfo echo = Stub(workspace.FullName, "echo", "", Literal);
            string output = await LocalAgentCli.RunProcessAsync(echo, prompt, Timeout, CancellationToken.None);
            Assert(output == prompt, "UTF-8 input and literal arguments round-trip outside the caller's directory");
            using JsonDocument payload = JsonDocument.Parse(output[(output.IndexOf("\n\n", StringComparison.Ordinal) + 2)..]);
            Assert(payload.RootElement.GetProperty("instruction").GetString() == Literal &&
                payload.RootElement.GetProperty("selectedText").GetString() == "새 문단\n" + Literal,
                "prompt and selection remain distinct JSON strings");

            foreach (string mode in new[] { "large-response", "large-stderr", "nontext" })
                await RejectAsync<InvalidDataException>(Stub(workspace.FullName, mode), "prompt", Timeout, CancellationToken.None);
            await RejectAsync<InvalidOperationException>(Stub(workspace.FullName, "failed"), "prompt", Timeout, CancellationToken.None);
            await RejectAsync<TimeoutException>(Stub(workspace.FullName, "wait"), "prompt",
                TimeSpan.FromMilliseconds(250), CancellationToken.None);

            string marker = Path.Combine(workspace.FullName, "ready");
            File.Delete(marker);
            using var request = new CancellationTokenSource();
            using var readiness = new CancellationTokenSource(Timeout);
            Task<string> canceled = LocalAgentCli.RunProcessAsync(Stub(workspace.FullName, "wait"), "prompt", Timeout, request.Token);
            while (!File.Exists(marker))
                await Task.Delay(10, readiness.Token);
            using Process child = Process.GetProcessById(int.Parse(await File.ReadAllTextAsync(marker, readiness.Token)));
            request.Cancel();
            await RejectAsync<OperationCanceledException>(canceled);
            Assert(child.HasExited, "cancellation reaps the CLI process");
            Console.WriteLine("Local agent process contract passed.");
        }
        finally
        {
            workspace.Delete(recursive: true);
        }
    }

    private static void CheckLauncherResolution(string root)
    {
        string prefix = Directory.CreateDirectory(Path.Combine(root, "npm scripts & tools")).FullName;
        string[] entries = [".", $"\"{prefix}\""];
        File.WriteAllText(Path.Combine(prefix, "codex.cmd"), "Batch script fixture");
        Assert(LocalAgentCli.FindExecutable(LocalAgentProvider.Codex, entries, Architecture.X64) is null,
            "batch shims and relative search paths are ignored");

        string codexVendor = Directory.CreateDirectory(Path.Combine(prefix, "node_modules", "@openai",
            "codex-win32-x64", "vendor", "x86_64-pc-windows-msvc", "bin")).FullName;
        string codex = Path.Combine(codexVendor, "codex.exe");
        File.WriteAllText(codex, "Native executable fixture");
        Assert(LocalAgentCli.FindExecutable(LocalAgentProvider.Codex, entries, Architecture.X64) == codex,
            "Codex npm resolves the packaged native executable");

        string claudeBin = Directory.CreateDirectory(Path.Combine(prefix, "node_modules", "@anthropic-ai",
            "claude-code", "bin")).FullName;
        string claude = Path.Combine(claudeBin, "claude.exe");
        File.WriteAllText(claude, "Native executable fixture");
        Assert(LocalAgentCli.FindExecutable(LocalAgentProvider.Claude, entries, Architecture.X64) == claude,
            "Claude Code npm resolves the packaged native executable");
    }

    private static ProcessStartInfo Stub(string workspace, params string[] stubArgs)
    {
        string executable = Environment.ProcessPath ?? throw new InvalidOperationException("Missing test host.");
        string[] launch = Path.GetFileNameWithoutExtension(executable) == "dotnet"
            ? [typeof(LocalAgentSmoke).Assembly.Location] : [];
        return LocalAgentCli.CreateStartInfo(executable, workspace, [.. launch, "--local-agent-stub", .. stubArgs]);
    }

    public static async Task RunStubAsync(string[] args)
    {
        Console.InputEncoding = Console.OutputEncoding = new UTF8Encoding(false);
        string input = await Console.In.ReadToEndAsync();
        switch (args)
        {
            case ["echo", "", Literal]:
                Assert(Path.GetFileName(Environment.CurrentDirectory).StartsWith("rhwp-agent-smoke-", StringComparison.Ordinal),
                    "injected scratch working directory");
                await Console.Error.WriteAsync(new string('e', 90_000));
                await Console.Out.WriteLineAsync(input);
                return;
            case ["large-response"]:
                await Console.Out.WriteAsync(new string('x', LocalAgentCli.MaxResponseLength + 1));
                return;
            case ["large-stderr"]:
                await Console.Error.WriteAsync(new string('e', 150_000));
                return;
            case ["nontext"]:
                await Console.Out.WriteAsync("answer\0more");
                return;
            case ["failed"]:
                Environment.ExitCode = 7;
                return;
            case ["wait"]:
                await File.WriteAllTextAsync("ready.tmp", Environment.ProcessId.ToString());
                File.Move("ready.tmp", "ready", overwrite: true);
                await Task.Delay(System.Threading.Timeout.InfiniteTimeSpan);
                return;
            default:
                throw new ArgumentException("Unexpected literal arguments.");
        }
    }

    private static Task RejectAsync<T>(ProcessStartInfo start, string prompt, TimeSpan timeout, CancellationToken cancellation)
        where T : Exception => RejectAsync<T>(LocalAgentCli.RunProcessAsync(start, prompt, timeout, cancellation));

    private static async Task RejectAsync<T>(Task<string> result) where T : Exception
    {
        try { await result; }
        catch (T) { return; }
        throw new InvalidOperationException($"Expected {typeof(T).Name}.");
    }

    private static void Assert(bool value, string message)
    {
        if (!value)
            throw new InvalidOperationException(message);
    }
}
