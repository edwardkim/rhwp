namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private const string UntitledDocumentName = "새 문서";

    internal async Task OpenNewDocumentAtStartupAsync()
    {
        if (page is not null)
            return;

        await ChangeDocumentAsync(
            async token => await worker.NewAsync(token),
            UntitledDocumentName,
            resetZoom: true,
            onCommitted: result =>
            {
                AdoptSaveLocation(result, null);
                return Task.CompletedTask;
            });
    }
}
