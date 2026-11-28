using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.Windows.Storage.Pickers;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private enum UnsavedDecision { Proceed, Discard, Cancel }

    private (ulong DocumentId, DocumentFileLocation Location)? saveLocation;
    private DocumentTarget? SaveTarget => page?.Target ?? editorInfo?.Target;

    private void AdoptSaveLocation(PageInfo? activePage, DocumentFileLocation? location)
    {
        saveLocation = activePage is not null && location is not null
            ? (activePage.Target.DocumentId, location)
            : null;
    }

    private async void OnSaveClicked(object sender, RoutedEventArgs args) => await SaveDocumentAsync(saveAs: false);
    private async void OnSaveCopyClicked(object sender, RoutedEventArgs args) => await SaveDocumentAsync(saveAs: true);

    private async Task<bool> SaveDocumentAsync(bool saveAs)
    {
        await SettleDirectInputAsync();
        if (!DocumentActionsAvailable || SaveTarget is not DocumentTarget target)
            return false;

        DocumentFileLocation? current = saveLocation is { } session && session.DocumentId == target.DocumentId
            ? session.Location
            : null;
        bool replaceOriginal = !saveAs && current?.Format is not null;
        if (replaceOriginal && editorInfo?.IsDirty != true)
            return true;

        editorBusy = true;
        RefreshButtons();
        try
        {
            string? path = replaceOriginal ? current!.Path : await PickSavePathAsync(current?.Format);
            if (path is null)
                return false;
            DocumentFormat format = (replaceOriginal ? current!.Format : DocumentFileLocation.FormatForPath(path))
                ?? throw new IOException("파일 이름에 .hwp 또는 .hwpx 확장자를 지정하세요.");
            StatusText.Text = "저장 중…";
            PreparedDocumentSave prepared = await worker.PrepareSaveAsync(target, format, CancellationToken.None);
            PublishedDocumentCopy? saved = await DocumentSaveCopy.PublishAsync(worker, prepared,
                path, replaceOriginal ? _ => Task.FromResult(false) : ApproveSaveLossAsync,
                CancellationToken.None, replaceOriginal ? current!.Fingerprint : null);
            if (saved is null)
            {
                documentError = replaceOriginal
                    ? "이 형식에서 일부 내용을 보존할 수 없습니다. 다른 이름으로 저장하세요."
                    : null;
                StatusText.Text = documentError ?? "저장을 취소했습니다.";
                return false;
            }

            // Both the file and the native save checkpoint have committed.
            // A later recovery-cleanup error must not change the saved state.
            saveLocation = (target.DocumentId, new DocumentFileLocation(saved.File.Path, saved.File.Fingerprint, format));
            fileName = Path.GetFileName(saved.File.Path);
            if (editorInfo is { } previous && previous.DocumentId == target.DocumentId)
            {
                editorInfo = previous with { Revision = saved.Status.Revision, IsDirty = saved.Status.Dirty };
            }
            documentError = null;
            UpdateTitle();
            StatusText.Text = "저장했습니다. " + fileName;
            if (!saved.Status.Dirty)
            {
                try
                {
                    await ClearRecoveryAsync();
                }
                catch (Exception error)
                {
                    documentError = "저장했지만 복구 파일을 정리하지 못했습니다. " + error.Message;
                    StatusText.Text = documentError;
                }
            }
            return !saved.Status.Dirty;
        }
        catch (DocumentFileConflictException)
        {
            documentError = replaceOriginal
                ? "저장할 파일이 외부에서 변경되었거나 이동되었습니다. 다른 이름으로 저장하세요."
                : "선택한 경로에 파일이 있습니다. 다른 이름으로 저장하세요.";
            StatusText.Text = documentError;
            return false;
        }
        catch (DocumentFilePublicationException error)
        {
            documentError = error.PossibleBackupPath is { } backup && File.Exists(backup)
                ? "저장을 확인할 수 없습니다. 교체 직전 파일의 백업을 보존했습니다. " + backup
                : "저장을 확인할 수 없습니다. 편집 내용을 다른 이름으로 저장하세요. " + error.Message;
            StatusText.Text = documentError;
            return false;
        }
        catch (Exception error)
        {
            documentError = "저장할 수 없습니다. " + error.Message;
            StatusText.Text = documentError;
            return false;
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
        }
    }

    private async Task<string?> PickSavePathAsync(DocumentFormat? currentFormat)
    {
        var picker = new FileSavePicker(AppWindow.Id)
        {
            SuggestedFileName = fileName is null || (saveLocation is null && fileName == UntitledDocumentName)
                ? UntitledDocumentName
                : Path.GetFileNameWithoutExtension(fileName) + " - 사본",
            DefaultFileExtension = currentFormat == DocumentFormat.Hwp ? ".hwp" : ".hwpx",
            ShowOverwritePrompt = false,
        };
        if (currentFormat == DocumentFormat.Hwp)
        {
            picker.FileTypeChoices.Add("한글 문서 (HWP)", new List<string> { ".hwp" });
            picker.FileTypeChoices.Add("한글 문서 (HWPX)", new List<string> { ".hwpx" });
        }
        else
        {
            picker.FileTypeChoices.Add("한글 문서 (HWPX)", new List<string> { ".hwpx" });
            picker.FileTypeChoices.Add("한글 문서 (HWP)", new List<string> { ".hwp" });
        }

        string? path = (await picker.PickSaveFileAsync())?.Path;
        if (path is not null && File.Exists(path))
            throw new DocumentFileConflictException(path);
        return path;
    }

    private async Task<bool> ApproveSaveLossAsync(SaveLossReport report)
    {
        string[] items = report.Losses.Select(loss => $"{loss.Path} — {loss.Code} ({loss.Reason})").ToArray();
        var content = new StackPanel { Spacing = 12 };
        content.Children.Add(new TextBlock
        {
            Text = $"선택한 파일 형식에서 {report.Count}개 항목을 보존할 수 없습니다.",
            TextWrapping = TextWrapping.Wrap,
        });
        content.Children.Add(new ListView
        {
            ItemsSource = items,
            MaxHeight = 240,
            SelectionMode = ListViewSelectionMode.None,
        });

        var dialog = new ContentDialog
        {
            XamlRoot = Root.XamlRoot,
            Title = "저장 중 일부 내용 손실",
            Content = content,
            PrimaryButtonText = "저장",
            CloseButtonText = "취소",
            DefaultButton = ContentDialogButton.Close,
        };
        return await dialog.ShowAsync() == ContentDialogResult.Primary;
    }

    private async Task<UnsavedDecision> ResolveUnsavedChangesAsync()
    {
        await SettleDirectInputAsync();
        if (!DocumentActionsAvailable)
            return UnsavedDecision.Cancel;
        if (editorInfo?.IsDirty != true)
            return UnsavedDecision.Proceed;

        ContentDialogResult result;
        editorBusy = true;
        RefreshButtons();
        try
        {
            var dialog = new ContentDialog
            {
                XamlRoot = Root.XamlRoot,
                Title = "변경 사항을 저장할까요?",
                Content = "현재 문서에 저장하지 않은 변경 사항이 있습니다.",
                PrimaryButtonText = "저장",
                SecondaryButtonText = "폐기",
                CloseButtonText = "취소",
                DefaultButton = ContentDialogButton.Close,
            };
            result = await dialog.ShowAsync();
        }
        catch (Exception error)
        {
            StatusText.Text = "변경 사항을 확인할 수 없습니다. " + error.Message;
            return UnsavedDecision.Cancel;
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
        }

        if (result == ContentDialogResult.Secondary)
            return UnsavedDecision.Discard;
        if (result == ContentDialogResult.Primary && await SaveDocumentAsync(saveAs: false))
            return UnsavedDecision.Proceed;

        return UnsavedDecision.Cancel;
    }
}
