using System.Diagnostics;
using System.Text.Json;
using Microsoft.Web.WebView2.Core;

namespace Rhwp.WinUI;

internal sealed record StudioState(bool HasDocument, string FileName, string? Format, bool Dirty,
    long DocumentEpoch, long ChangeSeq, bool CanNativeSave);

internal sealed record StudioExport(int ByteLength, string Format, long DocumentEpoch, long ChangeSeq);
internal sealed record StudioCommand(string Command, string? Format, string? SuggestedName);
internal sealed record StudioChunk(string Bytes);

internal sealed class StudioWebBridge : IDisposable
{
    internal const string HostName = "studio.rhwp.local";
    internal const string StartUri = "https://" + HostName + "/studio/index.html";
    internal const int ChunkBytes = 768 * 1024;
    private const int MaxMessageCharacters = 1024 * 1024 + 4096;
    private const int ProtocolVersion = 1;

    private static readonly JsonSerializerOptions WireJson = new(JsonSerializerDefaults.Web);

    internal const string InstallTransport = """
        (() => {
          if (window.top !== window || location.origin !== 'https://studio.rhwp.local'
              || location.pathname !== '/studio/index.html') return;
          const webview = window.chrome.webview;
          Object.defineProperty(window, 'rhwpDesktopHost', {
            value: Object.freeze({
              version: 1,
              appearance: 'fluent',
              postMessage: message => webview.postMessage(message),
              subscribe: listener => {
                const handler = event => listener(event.data);
                webview.addEventListener('message', handler);
                return () => webview.removeEventListener('message', handler);
              },
            }),
          });
        })();
        """;

    private readonly CoreWebView2 core;
    private readonly Func<StudioCommand, Task<string>> command;
    private readonly Dictionary<int, TaskCompletionSource<JsonElement>> pending = [];
    private readonly TaskCompletionSource ready = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private int nextId;
    private bool disposed;

    public StudioWebBridge(CoreWebView2 core, Func<StudioCommand, Task<string>> command)
    {
        this.core = core;
        this.command = command;
        core.WebMessageReceived += OnMessage;
    }

    public Task Ready => ready.Task;
    public void FailInitialization(Exception error) => ready.TrySetException(error);
    public event Action<StudioState>? StateChanged;

    public async Task<T> RequestAsync<T>(string method, object parameters) where T : class =>
        (await RequestCoreAsync(method, parameters)).Deserialize<T>(WireJson)
        ?? throw new InvalidDataException("편집기 응답이 비어 있습니다.");

    public async Task RequestAsync(string method, object parameters, TimeSpan? timeout = null)
    {
        await RequestCoreAsync(method, parameters, timeout);
    }

    private async Task<JsonElement> RequestCoreAsync(string method, object parameters, TimeSpan? timeout = null)
    {
        ObjectDisposedException.ThrowIf(disposed, this);
        int id = checked(++nextId);
        var response = new TaskCompletionSource<JsonElement>(TaskCreationOptions.RunContinuationsAsynchronously);
        pending.Add(id, response);
        try
        {
            Post(new { version = ProtocolVersion, kind = "request", id, method, @params = parameters });
            return await response.Task.WaitAsync(timeout ?? TimeSpan.FromMinutes(5));
        }
        finally
        {
            pending.Remove(id);
        }
    }

    private async void OnMessage(CoreWebView2 sender, CoreWebView2WebMessageReceivedEventArgs args)
    {
        if (disposed || !IsStudioPage(args.Source))
            return;

        try
        {
            string json = args.WebMessageAsJson;
            if (json.Length > MaxMessageCharacters)
                throw new InvalidDataException("편집기 메시지가 너무 큽니다.");

            using var message = JsonDocument.Parse(json, new JsonDocumentOptions { MaxDepth = 12 });
            JsonElement value = message.RootElement;
            if (value.ValueKind != JsonValueKind.Object ||
                !value.TryGetProperty("version", out JsonElement version) ||
                !version.TryGetInt32(out int protocol) || protocol != ProtocolVersion)
            {
                return;
            }

            switch (value.GetProperty("kind").GetString())
            {
                case "response":
                    CompleteResponse(value);
                    break;
                case "event":
                    string? name = value.GetProperty("event").GetString();
                    if (name == "failed")
                    {
                        FailInitialization(new InvalidOperationException("Studio 실행을 시작할 수 없습니다."));
                        return;
                    }
                    if (name is not ("ready" or "changed"))
                        return;
                    var state = value.GetProperty("value").Deserialize<StudioState>(WireJson)
                        ?? throw new InvalidDataException("편집기 상태를 읽을 수 없습니다.");
                    StateChanged?.Invoke(state);
                    if (name == "ready")
                        ready.TrySetResult();
                    break;
                case "command":
                    await CompleteCommandAsync(value);
                    break;
            }
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
        }
    }

    private void CompleteResponse(JsonElement value)
    {
        if (!value.GetProperty("id").TryGetInt32(out int id) || !pending.Remove(id, out var response))
            return;

        if (value.TryGetProperty("ok", out JsonElement success) && success.ValueKind == JsonValueKind.True &&
            value.TryGetProperty("result", out JsonElement result))
        {
            response.TrySetResult(result.Clone());
        }
        else
        {
            string? error = value.TryGetProperty("error", out JsonElement detail) && detail.ValueKind == JsonValueKind.String
                ? detail.GetString() : null;
            response.TrySetException(new InvalidOperationException(error ?? "편집기 요청에 실패했습니다."));
        }
    }

    private async Task CompleteCommandAsync(JsonElement value)
    {
        if (!value.GetProperty("id").TryGetInt32(out int id) || id <= 0)
            return;

        var requested = value.Deserialize<StudioCommand>(WireJson);
        string result = requested is { Command: "open" or "save" or "saveAs", Format: null or "hwp" or "hwpx" }
            ? await command(requested)
            : "failed";

        if (!disposed)
            Post(new { version = ProtocolVersion, kind = "commandResult", id, ok = true, result });
    }

    private void Post<T>(T message) => core.PostWebMessageAsJson(JsonSerializer.Serialize(message, WireJson));

    public static bool IsStudioPage(string? address) => IsBundledPage(address, "/studio/index.html");

    public static bool IsPrintPage(string? address) => IsBundledPage(address, "/studio/print.html");

    private static bool IsBundledPage(string? address, string path) =>
        Uri.TryCreate(address, UriKind.Absolute, out Uri? uri) &&
        uri.Scheme == Uri.UriSchemeHttps && uri.Host == HostName && uri.IsDefaultPort &&
        uri.UserInfo.Length == 0 && uri.AbsolutePath == path;

    public void Dispose()
    {
        if (disposed)
            return;
        disposed = true;
        core.WebMessageReceived -= OnMessage;
        ready.TrySetCanceled();
        foreach (var response in pending.Values)
            response.TrySetCanceled();
        pending.Clear();
    }
}
