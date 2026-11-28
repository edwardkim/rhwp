using System.Text.Json;

namespace Rhwp.WinUI;

internal sealed record NativeEquationLimits(int ScriptBytes, int MinimumSize, int MaximumSize);

internal sealed partial class NativeWorkerDispatcher
{
    private static NativeEquationLimits? ReadEquationLimits(JsonElement capabilities)
    {
        if (!HasCapability(capabilities, "insertEquation"))
            return null;

        var limits = new NativeEquationLimits(capabilities.GetProperty("maxEquationScriptBytes").GetInt32(),
            capabilities.GetProperty("minEquationFontSize").GetInt32(),
            capabilities.GetProperty("maxEquationFontSize").GetInt32());
        if (limits.ScriptBytes < 1 || limits.MinimumSize < 1 || limits.MinimumSize > limits.MaximumSize)
            throw new InvalidDataException("Native 수식 입력 제한이 올바르지 않습니다.");

        return limits;
    }

    public Task<EditResult> InsertEquationAsync(DocumentTarget expected, string script, int fontSize,
        CancellationToken cancellation) =>
        Dispatch(token => AdoptEditOutcome(expected,
            SubmitEditCommand(expected, new { command = "insertEquation", script, fontSize }, token)), cancellation);
}
