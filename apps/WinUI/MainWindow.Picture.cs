using Microsoft.UI.Xaml;
using Microsoft.Windows.Storage.Pickers;
using Windows.Foundation;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private NativePictureLimits? pictureLimits;

    private async void OnInsertPictureClicked(object sender, RoutedEventArgs args)
    {
        await SettleDirectInputAsync();
        if (!DocumentActionsAvailable || pictureLimits is not { InputBytes: > 0 } ||
            !TryGetBodyInsertionCaret(out PageInfo? requested, out TextPosition? caret))
            return;

        string? path = null;
        picking = true;
        RefreshButtons();
        try
        {
            var picker = new FileOpenPicker(AppWindow.Id);
            foreach (string extension in FileActivation.PictureExtensions)
                picker.FileTypeFilter.Add(extension);
            path = (await picker.PickSingleFileAsync())?.Path;
        }
        catch (Exception error)
        {
            StatusText.Text = "그림 파일을 열 수 없습니다. " + error.Message;
        }
        finally
        {
            picking = false;
            if (!closing)
                RefreshButtons();
        }

        if (path is not null && !closing)
            await QueuePictureInsertionAsync(path, requested, requestedCaret: caret);
    }

    private Task QueuePictureInsertionAsync(string path, PageInfo requested, Point? dropLocation = null,
        TextPosition? requestedCaret = null) => EnqueueDirectEdit(async () =>
    {
        if (pictureLimits is not { InputBytes: > 0 } limits || page is not { } before ||
            before.Target != requested.Target || before.FontGeneration != requested.FontGeneration)
            return;

        if ((dropLocation is Point location && !await MovePointerSelectionAsync(requested, location, extend: false)) ||
            !TryGetBodyInsertionCaret(out PageInfo? current, out TextPosition? caret) ||
            (requestedCaret is not null && !SamePosition(caret, requestedCaret)))
        {
            StatusText.Text = "본문의 텍스트 위치에 그림을 넣을 수 있습니다.";
            return;
        }

        await using var stream = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.Read,
            bufferSize: 4096, FileOptions.Asynchronous | FileOptions.SequentialScan);
        if (stream.Length <= 0 || stream.Length > limits.InputBytes)
        {
            StatusText.Text = $"그림은 {limits.InputBytes / (1024 * 1024)}MB 이하여야 합니다.";
            return;
        }
        var bytes = new byte[(int)stream.Length];
        await stream.ReadExactlyAsync(bytes);
        if (stream.ReadByte() != -1)
            throw new IOException("읽는 중에 그림 파일의 크기가 바뀌었습니다.");

        EditResult edit = await worker.InsertPictureAsync(current.Target, bytes, CancellationToken.None);
        await RefreshEditedPageAsync(current, edit);
        await UpdateDirectGeometryAsync(followCaret: true, focus: true);
    });
}
