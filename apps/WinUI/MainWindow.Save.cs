using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.Windows.Storage.Pickers;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private enum UnsavedDecision { Proceed, Discard, Cancel }

    private async void OnSaveCopyClicked(object sender, RoutedEventArgs args) => await SaveCopyAsync();

    private async Task<bool> SaveCopyAsync()
    {
        DocumentTarget? target = page?.Target ?? editorInfo?.Target;
        if (target is null || editorBusy || closing || picking || InlineEditing)
            return false;

        editorBusy = true;
        RefreshButtons();
        try
        {
            var picker = new FileSavePicker(AppWindow.Id)
            {
                SuggestedFileName = Path.GetFileNameWithoutExtension(fileName) + " - 사본",
                DefaultFileExtension = ".hwpx",
                ShowOverwritePrompt = false,
            };
            picker.FileTypeChoices.Add("한글 문서 (HWPX)", new List<string> { ".hwpx" });
            picker.FileTypeChoices.Add("한글 문서 (HWP)", new List<string> { ".hwp" });
            var destination = await picker.PickSaveFileAsync();
            if (destination is null)
                return false;
            if (File.Exists(destination.Path))
            {
                StatusText.Text = "다른 파일 이름을 선택하세요. 이 저장은 새 파일을 만듭니다.";
                return false;
            }

            DocumentFormat format = Path.GetExtension(destination.Path).ToLowerInvariant() switch
            {
                ".hwp" => DocumentFormat.Hwp,
                ".hwpx" => DocumentFormat.Hwpx,
                _ => throw new IOException("파일 이름에 .hwp 또는 .hwpx 확장자를 지정하세요."),
            };
            StatusText.Text = "저장 중…";
            PreparedDocumentSave prepared = await worker.PrepareSaveAsync(target, format, CancellationToken.None);
            PublishedDocumentCopy? saved = await DocumentSaveCopy.PublishAsync(worker, prepared,
                destination.Path, ApproveSaveLossAsync, CancellationToken.None);
            if (saved is null)
            {
                StatusText.Text = "저장을 취소했습니다.";
                return false;
            }

            fileName = Path.GetFileName(saved.File.Path);
            editorInfo = await worker.GetDocumentInfoAsync(target, CancellationToken.None);
            documentError = null;
            UpdateTitle();
            StatusText.Text = "저장했습니다. " + fileName;
            return !saved.Status.Dirty;
        }
        catch (DocumentFileConflictException)
        {
            StatusText.Text = "선택한 경로에 파일이 있습니다. 새 이름으로 저장하세요.";
            return false;
        }
        catch (Exception error)
        {
            StatusText.Text = "저장할 수 없습니다. " + error.Message;
            return false;
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
        }
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
        if (InlineEditing)
        {
            StatusText.Text = "현재 문단을 적용하거나 취소하세요.";
            return UnsavedDecision.Cancel;
        }
        if (editorBusy)
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
                PrimaryButtonText = "새 파일에 저장",
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
        if (result == ContentDialogResult.Primary && await SaveCopyAsync())
            return UnsavedDecision.Proceed;

        return UnsavedDecision.Cancel;
    }
}
