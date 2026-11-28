using Microsoft.UI.Xaml;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private NativeDocumentInfo? editorInfo;
    private bool editorBusy;

    private async void OnUndoClicked(object sender, RoutedEventArgs args) =>
        await RunHistoryActionAsync(redo: false);

    private async void OnRedoClicked(object sender, RoutedEventArgs args) =>
        await RunHistoryActionAsync(redo: true);

    private Task RunHistoryActionAsync(bool redo)
    {
        if (!DocumentActionsAvailable)
            return Task.CompletedTask;

        return EnqueueDirectEdit(() => EditHistoryAsync(redo));
    }

    private async Task EditHistoryAsync(bool redo)
    {
        if (page is not PageInfo current ||
            (redo ? editorInfo?.CanRedo : editorInfo?.CanUndo) != true)
            return;

        CancellationToken token = CancellationToken.None;
        EditResult edit = redo
            ? await worker.RedoAsync(current.Target, token)
            : await worker.UndoAsync(current.Target, token);
        await RefreshEditedPageAsync(current, edit);
        if (directEditing)
            await UpdateDirectGeometryAsync(followCaret: true);
    }

    private async Task RefreshEditedPageAsync(PageInfo previous, EditResult edit)
    {
        if (edit.Target != previous.Target)
        {
            // Keep the last good image while a new layout and raster are requested.
            // Any in-flight image or accessibility response belongs to the old revision.
            renderRequest?.Cancel();
            renderTimer.Stop();
            viewVersion++;
            UpdateAutomationDocument(null);
            DrawSearchHighlight(null);
            ClearDirectGeometry();
        }

        page = previous with { Target = edit.Target };
        CancellationToken token = CancellationToken.None;
        if (editorInfo is NativeDocumentInfo prior && prior.DocumentId == edit.Target.DocumentId)
        {
            // A selection change can keep the same revision. Composition changes
            // sometimes advance the render revision without changing committed history.
            editorInfo = prior with
            {
                Revision = edit.Target.Revision,
                Selection = edit.Outcome.Selection,
                CompositionId = edit.Outcome.CompositionId,
            };
            if (edit.Target == previous.Target && !edit.Outcome.HistoryChanged)
                return;
        }

        NativeDocumentInfo info = await worker.GetDocumentInfoAsync(edit.Target, token);
        editorInfo = info;
        documentError = null;
        UpdateTitle();
        if (edit.Outcome.HistoryChanged && info.IsDirty)
            ScheduleCommittedRecovery();

        if (info.PageCount == 0 || edit.Target == previous.Target)
            return;

        uint index = Math.Min(previous.Page, info.PageCount - 1);
        PageInfo updated = await worker.GetPageAsync(previous with
        {
            Target = info.Target,
            PageCount = info.PageCount,
            FontGeneration = info.FontGeneration,
        }, index, token);
        page = updated;
        SetPageSize(updated);
        QueueRender();
    }

    private void ScheduleCommittedRecovery()
    {
        if (editorInfo is null)
            return;

        pendingRecoveryDocument = editorInfo.DocumentId;
        recoveryTimer.Stop();
        recoveryTimer.Start();
    }

    private void OnRecoveryTick(Microsoft.UI.Dispatching.DispatcherQueueTimer sender, object args)
    {
        if (closing || pendingRecoveryDocument is not ulong expected)
            return;
        if (!DocumentActionsAvailable)
        {
            recoveryTimer.Start();
            return;
        }

        pendingRecoveryDocument = null;
        if (editorInfo is { IsDirty: true, CompositionId: null } info && info.DocumentId == expected)
            EnqueueDirectEdit(() => CheckpointRecoveryAsync(info));
    }

    private void UpdateTitle()
    {
        Title = fileName is null ? "RHWP" : fileName + (editorInfo?.IsDirty == true ? " * — RHWP" : " — RHWP");
    }
}
