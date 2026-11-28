namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private const string UntitledDocumentName = "새 문서";

    internal async Task OpenNewDocumentAtStartupAsync()
    {
        if (page is not null)
            return;

        bool opened = await ChangeDocumentAsync(
            async token => await worker.NewAsync(token),
            UntitledDocumentName,
            resetZoom: true,
            onCommitted: result =>
            {
                AdoptSaveLocation(result, null);
                return Task.CompletedTask;
            });
        if (!opened || page is not PageInfo current)
            return;

        await EnqueueDirectEdit(async () =>
        {
            var caret = new TextPosition(0, 0, 0, []);
            editorInfo = await worker.BeginEditAsync(current.Target, new TextSelection(caret, caret),
                CancellationToken.None);
            directEditing = true;
            await UpdateDirectGeometryAsync(followCaret: false, focus: true);
        });
    }
}
