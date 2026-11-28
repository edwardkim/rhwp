using System.Diagnostics;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private DocumentRecoveryStore? recoveryStore;
    private DocumentRecoveryStore.RecoveryLease? recoveryLease;
    private DocumentTarget? checkpointTarget;
    private bool recoveryChecked;

    internal async Task<bool> RestoreRecoveryAtStartupAsync()
    {
        if (recoveryChecked || !DocumentActionsAvailable || page is not null)
            return false;

        recoveryChecked = true;
        editorBusy = true;
        RefreshButtons();
        RecoveryCandidate? candidate = null;
        try
        {
            candidate = await GetRecoveryStore().TryClaimLatestAsync();
            if (candidate is null)
                return false;

            string name = DocumentRecoveryStore.DisplayName(candidate.Header.Name);
            var dialog = new ContentDialog
            {
                XamlRoot = Root.XamlRoot,
                Title = "저장하지 않은 문서 복구",
                Content = new TextBlock
                {
                    Text = $"{name}의 복구 사본이 있습니다. 복구본은 새 문서로 열립니다.",
                    TextWrapping = TextWrapping.Wrap,
                },
                PrimaryButtonText = "복구",
                SecondaryButtonText = "복구본 삭제",
                CloseButtonText = "건너뛰기",
                DefaultButton = ContentDialogButton.Close,
            };
            ContentDialogResult choice = await dialog.ShowAsync();
            if (choice == ContentDialogResult.Secondary)
                await candidate.Lease.DeleteAsync();
            if (choice != ContentDialogResult.Primary)
                return false;

            RecoveryCandidate selected = candidate;
            byte[] bytes = await selected.Lease.LoadAsync(selected.Header);
            editorBusy = false;
            RecoveredDocument? recovered = null;
            return await ChangeDocumentAsync(async token =>
            {
                recovered = await worker.OpenRecoveryAsync(bytes, token);
                return recovered.Page;
            }, name, resetZoom: true, onCommitted: restoredPage =>
            {
                if (recovered is not null && restoredPage == recovered.Page)
                {
                    editorInfo = recovered.Info;
                    AdoptSaveLocation(restoredPage, null);
                    recoveryLease = selected.Lease;
                    checkpointTarget = recovered.Page.Target;
                }
                return Task.CompletedTask;
            });
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
            StatusText.Text = "복구 사본을 열거나 정리할 수 없습니다. " + error.Message;
            return false;
        }
        finally
        {
            if (candidate is not null && !ReferenceEquals(recoveryLease, candidate.Lease))
                candidate.Lease.Dispose();
            editorBusy = false;
            RefreshButtons();
        }
    }

    private async Task CheckpointRecoveryAsync(NativeDocumentInfo info)
    {
        if (!info.IsDirty)
        {
            await ClearRecoveryAsync();
            return;
        }
        if (checkpointTarget == info.Target)
            return;

        try
        {
            if (await TryCheckpointRecoveryAsync(info.Target, DocumentFormat.Hwpx))
                return;
            if (string.Equals(Path.GetExtension(fileName), ".hwp", StringComparison.OrdinalIgnoreCase) &&
                await TryCheckpointRecoveryAsync(info.Target, DocumentFormat.Hwp))
            {
                return;
            }
            throw new IOException("현재 문서에 복구 사본으로 보존할 수 없는 내용이 있습니다.");
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
            documentError = "복구 사본을 갱신할 수 없습니다. " + error.Message;
            StatusText.Text = documentError;
        }
    }

    private async Task<bool> TryCheckpointRecoveryAsync(DocumentTarget target, DocumentFormat format)
    {
        PreparedDocumentSave prepared = await worker.PrepareSaveAsync(target, format, CancellationToken.None);
        Exception? failure = null;
        try
        {
            if (prepared.Report.Count != 0)
                return false;

            recoveryLease ??= await GetRecoveryStore().CreateAsync();
            await recoveryLease.SaveAsync(target.Revision, format, fileName, prepared.Bytes);
            checkpointTarget = target;
            return true;
        }
        catch (Exception error)
        {
            failure = error;
            throw;
        }
        finally
        {
            try
            {
                // A recovery checkpoint keeps the worker's document dirty.
                await worker.CancelSaveAsync(prepared);
            }
            catch (Exception error) when (failure is not null)
            {
                failure.Data["NativeRecoveryCleanup"] = error;
            }
        }
    }

    private async Task ClearRecoveryAsync()
    {
        DocumentRecoveryStore.RecoveryLease? previous = recoveryLease;
        recoveryLease = null;
        checkpointTarget = null;
        if (previous is null)
            return;

        using (previous)
        {
            try
            {
                await previous.DeleteAsync();
            }
            catch (Exception error) when (error is IOException or UnauthorizedAccessException)
            {
                Debug.WriteLine(error);
                documentError = "복구 사본을 정리할 수 없습니다. " + error.Message;
                StatusText.Text = documentError;
            }
        }
    }

    private DocumentRecoveryStore GetRecoveryStore()
    {
        if (recoveryStore is not null)
            return recoveryStore;

        string root = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        if (string.IsNullOrEmpty(root))
            throw new DirectoryNotFoundException("사용자 복구 폴더를 찾을 수 없습니다.");
        recoveryStore = new DocumentRecoveryStore(Path.Combine(root, "rhwp", "WinUI", "Recovery"));
        return recoveryStore;
    }
}
