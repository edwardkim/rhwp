using System.Diagnostics;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using Microsoft.Windows.Storage.Pickers;
using Windows.Data.Pdf;
using Windows.Security.Cryptography;
using Windows.Storage.Streams;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private const uint MaxPdfPreviewPixels = 1536;
    private const ulong MaxPdfPreviewImageBytes = 12 * 1024 * 1024;
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
            StatusText.Text = "PDF 작업 취소 중…";
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
            CancellationToken token = operation.Token;
            StatusText.Text = "PDF 미리보기를 만드는 중…";
            byte[] pdf = await worker.ExportPagePdfAsync(current, limits.MaxOutputBytes, token);
            ReadOnlyMemory<byte> prepared = pdf;
            BitmapImage image = await RenderPdfPreviewAsync(CryptographicBuffer.CreateFromByteArray(pdf), token);
            token.ThrowIfCancellationRequested();
            if (page != current)
                throw new InvalidOperationException("미리 볼 문서 또는 쪽이 바뀌었습니다.");

            var preview = new ContentDialog
            {
                XamlRoot = Root.XamlRoot,
                Title = $"{current.Page + 1}쪽 PDF 미리보기",
                Content = new Image { Source = image, MaxWidth = 480, MaxHeight = 520, Stretch = Stretch.Uniform },
                PrimaryButtonText = "새 PDF 저장",
                CloseButtonText = "닫기",
                DefaultButton = ContentDialogButton.Close,
            };
            if (await preview.ShowAsync() != ContentDialogResult.Primary)
            {
                StatusText.Text = "PDF 미리보기를 닫았습니다.";
                return;
            }

            token.ThrowIfCancellationRequested();
            var picker = new FileSavePicker(AppWindow.Id)
            {
                SuggestedFileName = $"{Path.GetFileNameWithoutExtension(fileName)} - {current.Page + 1}쪽",
                DefaultFileExtension = ".pdf",
                ShowOverwritePrompt = false,
            };
            picker.FileTypeChoices.Add("PDF 문서", new List<string> { ".pdf" });
            var destination = await picker.PickSaveFileAsync();
            if (destination is null)
            {
                StatusText.Text = "PDF 저장을 취소했습니다.";
                return;
            }

            token.ThrowIfCancellationRequested();
            if (page != current)
                throw new InvalidOperationException("저장할 문서 또는 쪽이 바뀌었습니다.");
            if (!string.Equals(Path.GetExtension(destination.Path), ".pdf", StringComparison.OrdinalIgnoreCase))
                throw new IOException("파일 이름에 .pdf 확장자를 지정하세요.");
            if (File.Exists(destination.Path))
                throw new DocumentFileConflictException(destination.Path);

            StatusText.Text = "PDF 저장 중…";
            DocumentFileReceipt saved = await DocumentFileStore.PublishAsync(destination.Path, prepared,
                expectedDestination: null, cancellationToken: token);
            StatusText.Text = "PDF를 저장했습니다. " + Path.GetFileName(saved.Path);
        }
        catch (OperationCanceledException) when (operation.IsCancellationRequested)
        {
            StatusText.Text = "PDF 작업을 취소했습니다.";
        }
        catch (DocumentFileConflictException)
        {
            StatusText.Text = "선택한 경로에 파일이 있습니다. 새 이름으로 저장하세요.";
        }
        catch (Exception error)
        {
            StatusText.Text = "PDF 작업을 완료할 수 없습니다. " + error.Message;
        }
        finally
        {
            pdfCancellation = null;
            editorBusy = false;
            RefreshButtons();
        }
    }

    private static async Task<BitmapImage> RenderPdfPreviewAsync(IBuffer bytes, CancellationToken token)
    {
        token.ThrowIfCancellationRequested();
        using var input = new InMemoryRandomAccessStream();
        if (await input.WriteAsync(bytes) != bytes.Length)
            throw new IOException("미리보기 스트림에 전체 PDF를 쓰지 못했습니다.");

        token.ThrowIfCancellationRequested();
        input.Seek(0);
        PdfDocument pdf = await PdfDocument.LoadFromStreamAsync(input);
        if (pdf.PageCount != 1)
            throw new InvalidDataException("미리보기에는 한 쪽 PDF가 필요합니다.");

        token.ThrowIfCancellationRequested();
        using PdfPage page = pdf.GetPage(0);
        var size = page.Size;
        if (!double.IsFinite(size.Width) || !double.IsFinite(size.Height) || size.Width <= 0 || size.Height <= 0)
            throw new InvalidDataException("PDF 쪽 크기가 올바르지 않습니다.");

        double scale = Math.Min(2.0, MaxPdfPreviewPixels / Math.Max((double)size.Width, size.Height));
        var options = new PdfPageRenderOptions
        {
            DestinationWidth = (uint)Math.Clamp(Math.Ceiling(size.Width * scale), 1, MaxPdfPreviewPixels),
            DestinationHeight = (uint)Math.Clamp(Math.Ceiling(size.Height * scale), 1, MaxPdfPreviewPixels),
        };

        using var raster = new InMemoryRandomAccessStream();
        await page.RenderToStreamAsync(raster, options);
        token.ThrowIfCancellationRequested();
        if (raster.Size == 0 || raster.Size > MaxPdfPreviewImageBytes)
            throw new InvalidDataException("PDF 미리보기 이미지 크기가 허용 범위를 벗어났습니다.");

        raster.Seek(0);
        var image = new BitmapImage();
        await image.SetSourceAsync(raster);
        token.ThrowIfCancellationRequested();
        if (image.PixelWidth <= 0 || image.PixelHeight <= 0 ||
            image.PixelWidth > MaxPdfPreviewPixels || image.PixelHeight > MaxPdfPreviewPixels)
            throw new InvalidDataException("PDF 미리보기의 픽셀 크기가 올바르지 않습니다.");

        return image;
    }
}
