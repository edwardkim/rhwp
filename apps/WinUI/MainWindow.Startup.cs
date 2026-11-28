using Microsoft.UI.Xaml;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private const string UntitledDocumentName = "새 문서";

    internal Task OpenNewDocumentAtStartupAsync() =>
        page is null ? CreateNewDocumentAsync(discardChanges: false) : Task.CompletedTask;

    private async void OnNewDocumentClicked(object sender, RoutedEventArgs args)
    {
        if (!DocumentActionsAvailable)
            return;

        UnsavedDecision decision = await ResolveUnsavedChangesAsync();
        if (decision != UnsavedDecision.Cancel)
            await CreateNewDocumentAsync(discardChanges: decision == UnsavedDecision.Discard);
    }

    private async Task CreateNewDocumentAsync(bool discardChanges)
    {
        bool opened = await ChangeDocumentAsync(
            async token => await worker.NewAsync(token, discardChanges),
            UntitledDocumentName,
            resetZoom: true,
            onCommitted: async result =>
            {
                AdoptSaveLocation(result, null);
                await ClearRecoveryAsync();
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
