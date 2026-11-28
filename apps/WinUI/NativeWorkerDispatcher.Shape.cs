using System.Text.Json;

namespace Rhwp.WinUI;

internal enum NativeDrawingShape { Rectangle, Ellipse, Line }
internal sealed record NativeShapeLimits(int MinimumSize, int MaximumSize);

internal sealed partial class NativeWorkerDispatcher
{
    private static NativeShapeLimits? ReadShapeLimits(JsonElement capabilities)
    {
        if (!HasCapability(capabilities, "insertShape"))
            return null;

        var limits = new NativeShapeLimits(capabilities.GetProperty("minShapeSize").GetInt32(),
            capabilities.GetProperty("maxShapeSize").GetInt32());
        if (limits.MinimumSize < 1 || limits.MinimumSize > limits.MaximumSize)
            throw new InvalidDataException("Native 도형 크기 제한이 올바르지 않습니다.");

        return limits;
    }

    public Task<EditResult> InsertDrawingShapeAsync(DocumentTarget expected, NativeDrawingShape shape, int width,
        int height, CancellationToken cancellation) => Dispatch(token =>
        {
            string name = JsonNamingPolicy.CamelCase.ConvertName(shape.ToString());
            return AdoptEditOutcome(expected,
                SubmitEditCommand(expected, new { command = "insertShape", shape = name, width, height }, token));
        }, cancellation);
}
