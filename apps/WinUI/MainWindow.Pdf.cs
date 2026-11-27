using System.Diagnostics;
using Microsoft.UI.Xaml;
using Microsoft.Windows.Storage.Pickers;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private LivePdfLimits? pdfLimits;
    private CancellationTokenSource? pdfCancellation;

    private async Task LoadPdfCapabilityAsync()
    {
        try
        {
            pdfLimits = await worker.GetLivePdfLimitsAsync(CancellationToken.None);
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
        }

        if (!closing)
            RefreshButtons();
    }

    private async void OnExportPdfClicked(object sender, RoutedEventArgs args)
    {
        if (pdfCancellation is CancellationTokenSource pending)
        {
            pending.Cancel();
            StatusText.Text = "PDF 내보내기 취소 중…";
            RefreshButtons();
            return;
        }

        if (closing || editorBusy || picking || page is not PageInfo current ||
            pdfLimits is not LivePdfLimits limits)
            return;

        using var operation = new CancellationTokenSource();
        pdfCancellation = operation;
        editorBusy = true;
        RefreshButtons();
        try
        {
            var picker = new FileSavePicker(AppWindow.Id)
            {
                SuggestedFileName = $"{Path.GetFileNameWithoutExtension(fileName)} - {current.Page + 1}쪽",
                DefaultFileExtension = ".pdf",
                ShowOverwritePrompt = false,
            };
            picker.FileTypeChoices.Add("PDF 문서", new List<string> { ".pdf" });
            var destination = await picker.PickSaveFileAsync();
            if (destination is null)
                return;

            operation.Token.ThrowIfCancellationRequested();
            if (!string.Equals(Path.GetExtension(destination.Path), ".pdf", StringComparison.OrdinalIgnoreCase))
                throw new IOException("파일 이름에 .pdf 확장자를 지정하세요.");
            if (File.Exists(destination.Path))
                throw new DocumentFileConflictException(destination.Path);

            StatusText.Text = "PDF 내보내는 중…";
            byte[] bytes = await worker.ExportPagePdfAsync(current, limits.MaxOutputBytes, operation.Token);
            if (page != current)
                throw new InvalidOperationException("내보낼 문서 또는 쪽이 바뀌었습니다.");

            DocumentFileReceipt saved = await DocumentFileStore.PublishAsync(destination.Path, bytes,
                expectedDestination: null, cancellationToken: operation.Token);
            StatusText.Text = "PDF를 저장했습니다. " + Path.GetFileName(saved.Path);
        }
        catch (OperationCanceledException) when (operation.IsCancellationRequested)
        {
            StatusText.Text = "PDF 내보내기를 취소했습니다.";
        }
        catch (DocumentFileConflictException)
        {
            StatusText.Text = "선택한 경로에 파일이 있습니다. 새 이름으로 저장하세요.";
        }
        catch (Exception error)
        {
            StatusText.Text = "PDF를 내보낼 수 없습니다. " + error.Message;
        }
        finally
        {
            pdfCancellation = null;
            editorBusy = false;
            RefreshButtons();
        }
    }
}
