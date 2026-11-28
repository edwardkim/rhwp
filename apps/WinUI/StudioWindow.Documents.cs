using System.Diagnostics;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml.Controls;
using Microsoft.Windows.Storage.Pickers;

namespace Rhwp.WinUI;

public sealed partial class StudioWindow
{
    private static readonly string[] DocumentExtensions = [".hwp", ".hwpx", ".hml", ".pdf"];
    private (long Epoch, DocumentFileLocation Location)? currentFile;
    private (long Epoch, long ChangeSeq) lastState;

    internal static string? FindStartupDocument(IEnumerable<string> paths) =>
        paths.FirstOrDefault(path => !string.IsNullOrWhiteSpace(path) &&
            DocumentExtensions.Contains(Path.GetExtension(path), StringComparer.OrdinalIgnoreCase));

    internal async Task OpenStartupDocumentAsync(string path)
    {
        try
        {
            await initialized.Task;
            await RunDocumentActionAsync(async () =>
            {
                StudioState state = await GetStudioStateAsync();
                // A restored edit remains active after the Studio startup dialog.
                return state.Dirty ? "cancelled" : await LoadFileAsync(path, state);
            });
        }
        catch (OperationCanceledException)
        {
        }
    }

    private Task<string> HandleCommandAsync(StudioCommand command) => RunDocumentActionAsync(
        () => command.Command == "open"
            ? PickAndOpenAsync()
            : SaveStudioDocumentAsync(command.Command == "saveAs", command.Format, command.SuggestedName));

    private async Task<string> RunDocumentActionAsync(Func<Task<string>> action)
    {
        if (busy || closing || !initialized.Task.IsCompletedSuccessfully)
            return "cancelled";

        busy = true;
        StudioStatus.IsOpen = false;
        try
        {
            return await action();
        }
        catch (DocumentFileConflictException)
        {
            ShowError("파일이 다른 곳에서 수정되었거나 이동했습니다. 다른 이름으로 저장하세요.");
            return "failed";
        }
        catch (DocumentFilePublicationException error)
        {
            string? backup = error.PossibleBackupPath;
            ShowError(backup is not null && File.Exists(backup)
                ? "저장 결과를 확인할 수 없습니다. 교체 전 파일을 보존했습니다. " + backup
                : "저장 결과를 확인할 수 없습니다. 편집본을 다른 이름으로 저장하세요.");
            return "failed";
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
            ShowError(error.Message);
            return "failed";
        }
        finally
        {
            busy = false;
        }
    }

    private async Task<string> PickAndOpenAsync()
    {
        var picker = new FileOpenPicker(AppWindow.Id);
        foreach (string extension in DocumentExtensions)
            picker.FileTypeFilter.Add(extension);

        var picked = await picker.PickSingleFileAsync();
        if (picked is null)
            return "cancelled";

        StudioState? state = await ConfirmDocumentReplacementAsync(discardRecovery: false);
        return state is null ? "cancelled" : await LoadFileAsync(picked.Path, state);
    }

    private async Task<string> LoadFileAsync(string path, StudioState prior)
    {
        StudioOpenedFile opened = await StudioFileStorage.OpenAsync(path);
        StudioWebBridge target = bridge!;
        try
        {
            await target.RequestAsync("load.begin", new
            {
                fileName = Path.GetFileName(opened.Location.Path),
                byteLength = opened.Bytes.Length,
                prior.DocumentEpoch,
                prior.ChangeSeq,
            });

            for (int offset = 0; offset < opened.Bytes.Length;)
            {
                int count = Math.Min(StudioWebBridge.ChunkBytes, opened.Bytes.Length - offset);
                await target.RequestAsync("load.chunk", new
                {
                    offset,
                    bytes = Convert.ToBase64String(opened.Bytes, offset, count),
                });
                offset += count;
            }

            StudioState loaded = await target.RequestAsync<StudioState>("load.commit", new { });
            if (!loaded.HasDocument || loaded.DocumentEpoch <= prior.DocumentEpoch)
                throw new InvalidDataException("편집기에서 열린 새 문서를 확인할 수 없습니다.");

            currentFile = (loaded.DocumentEpoch, opened.Location);
            UpdateStudioState(loaded);
            return "completed";
        }
        finally
        {
            await ReleaseTransferAsync(target);
        }
    }

    private async Task<string> SaveStudioDocumentAsync(bool saveAs, string? requestedFormat, string? suggestedName = null)
    {
        StudioState state = await GetStudioStateAsync();
        if (!state.HasDocument)
            return "cancelled";
        if (!state.CanNativeSave)
        {
            ShowError("이 문서는 Studio의 파일 메뉴에서 저장하세요.");
            return "failed";
        }

        DocumentFileLocation? original = currentFile is { } binding && binding.Epoch == state.DocumentEpoch
            ? binding.Location : null;
        DocumentFormat preferred = FormatFromName(requestedFormat ?? state.Format);
        bool replace = !saveAs && original?.Format == preferred;
        // A clean restored or browser-opened document still needs a path on disk.
        if (replace && !state.Dirty)
            return "saved";

        string? destination = replace ? original!.Path : await PickDestinationAsync(state.FileName, preferred, suggestedName);
        if (destination is null)
            return "cancelled";

        DocumentFormat actualFormat = DocumentFileLocation.FormatForPath(destination)
            ?? throw new InvalidDataException("파일 이름에 .hwp 또는 .hwpx 확장자를 지정하세요.");
        StudioWebBridge target = bridge!;
        string format = actualFormat == DocumentFormat.Hwp ? "hwp" : "hwpx";
        try
        {
            StudioExport exported = await target.RequestAsync<StudioExport>("export", new { format });
            if (exported.ByteLength is <= 0 or > StudioFileStorage.MaxDocumentBytes ||
                exported.Format != format || exported.DocumentEpoch != state.DocumentEpoch)
            {
                throw new InvalidDataException("저장할 문서의 형식이나 버전이 바뀌었습니다.");
            }

            var bytes = new byte[exported.ByteLength];
            for (int offset = 0; offset < bytes.Length;)
            {
                int count = Math.Min(StudioWebBridge.ChunkBytes, bytes.Length - offset);
                StudioChunk part = await target.RequestAsync<StudioChunk>("export.chunk", new { offset, count });
                Span<byte> output = bytes.AsSpan(offset, count);
                if (!Convert.TryFromBase64String(part.Bytes, output, out int written) || written != count)
                    throw new InvalidDataException("편집기에서 내보낸 문서의 길이가 일치하지 않습니다.");
                offset += count;
            }

            DocumentFileReceipt file = await StudioFileStorage.SaveAsync(destination, actualFormat, bytes,
                replace ? original : null);
            currentFile = (exported.DocumentEpoch, new DocumentFileLocation(file.Path, file.Fingerprint, actualFormat));
            try
            {
                StudioState saved = await target.RequestAsync<StudioState>("saved", new
                {
                    fileName = Path.GetFileName(file.Path),
                    exported.DocumentEpoch,
                    exported.ChangeSeq,
                });
                UpdateStudioState(saved);
                return !saved.Dirty ? "saved" : "failed";
            }
            catch (Exception error)
            {
                Debug.WriteLine(error);
                ShowError("파일은 저장했습니다. 편집기의 미저장 상태를 확인해 주세요.");
                return "failed";
            }
        }
        finally
        {
            await ReleaseTransferAsync(target);
        }
    }

    private async Task<string?> PickDestinationAsync(string name, DocumentFormat preferred, string? suggestedName)
    {
        string title = Path.GetFileNameWithoutExtension(suggestedName ?? name);
        if (string.IsNullOrWhiteSpace(title) || title.IndexOfAny(Path.GetInvalidFileNameChars()) >= 0)
            title = "새 문서";
        if (string.IsNullOrWhiteSpace(suggestedName))
            title += " - 사본";
        var picker = new FileSavePicker(AppWindow.Id)
        {
            SuggestedFileName = title,
            DefaultFileExtension = preferred == DocumentFormat.Hwp ? ".hwp" : ".hwpx",
            ShowOverwritePrompt = false,
        };
        foreach (DocumentFormat format in new[] { preferred, preferred == DocumentFormat.Hwp ? DocumentFormat.Hwpx : DocumentFormat.Hwp })
        {
            if (format == DocumentFormat.Hwp)
                picker.FileTypeChoices.Add("한글 문서 (HWP)", new List<string> { ".hwp" });
            else
                picker.FileTypeChoices.Add("한글 문서 (HWPX)", new List<string> { ".hwpx" });
        }

        return (await picker.PickSaveFileAsync())?.Path;
    }

    private static DocumentFormat FormatFromName(string? name) =>
        name == "hwp" ? DocumentFormat.Hwp : DocumentFormat.Hwpx;

    private Task<StudioState> GetStudioStateAsync() => bridge!.RequestAsync<StudioState>("state", new { });

    private void UpdateStudioState(StudioState state)
    {
        if (lastState.CompareTo((state.DocumentEpoch, state.ChangeSeq)) > 0)
            return;
        lastState = (state.DocumentEpoch, state.ChangeSeq);
        if (currentFile is { } file && file.Epoch != state.DocumentEpoch)
            currentFile = null;
        Title = state.HasDocument && !string.IsNullOrWhiteSpace(state.FileName)
            ? state.FileName + (state.Dirty ? " * — RHWP" : " — RHWP")
            : "RHWP";
    }

    private async Task<StudioState?> ConfirmDocumentReplacementAsync(bool discardRecovery)
    {
        StudioState current = await GetStudioStateAsync();
        if (!current.Dirty)
            return current;

        var dialog = new ContentDialog
        {
            XamlRoot = StudioRoot.XamlRoot,
            Title = "변경 사항을 저장할까요?",
            Content = current.CanNativeSave
                ? "현재 문서에 저장하지 않은 변경 사항이 있습니다."
                : "변경 사항을 유지하려면 Studio의 파일 메뉴에서 저장하세요.",
            PrimaryButtonText = current.CanNativeSave ? "저장" : "",
            SecondaryButtonText = "폐기",
            CloseButtonText = "취소",
            DefaultButton = ContentDialogButton.Close,
        };

        ContentDialogResult result = await dialog.ShowAsync();
        if (result == ContentDialogResult.Primary)
            return await SaveStudioDocumentAsync(false, null) == "saved" ? await GetStudioStateAsync() : null;
        if (result != ContentDialogResult.Secondary)
            return null;
        if (!discardRecovery)
            return current;

        return await bridge!.RequestAsync<StudioState>("discard", new { current.DocumentEpoch, current.ChangeSeq });
    }

    private async Task ReleaseTransferAsync(StudioWebBridge target)
    {
        try
        {
            await target.RequestAsync("release", new { }, TimeSpan.FromSeconds(10));
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
        }
    }

    private async void OnClosing(AppWindow sender, AppWindowClosingEventArgs args)
    {
        if (closing)
            return;
        args.Cancel = true;
        if (busy)
            return;
        if (initialized.Task.IsCompletedSuccessfully &&
            await RunDocumentActionAsync(async () =>
                await ConfirmDocumentReplacementAsync(discardRecovery: true) is not null ? "completed" : "cancelled")
                != "completed")
        {
            return;
        }

        closing = true;
        Close();
    }
}
