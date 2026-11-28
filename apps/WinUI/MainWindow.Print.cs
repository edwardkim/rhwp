using Microsoft.UI;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Input;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private CancellationTokenSource? printCancellation;

    private void RefreshPrintAction(bool available)
    {
        PrintMenuItem.Text = printCancellation is null ? "인쇄…" : "인쇄 작업 취소";
        PrintMenuItem.IsEnabled = printCancellation is { } pending
            ? !pending.IsCancellationRequested
            : available && page is { PageCount: > 0 };
    }

    private async void OnPrintClicked(object sender, RoutedEventArgs args) => await PrintDocumentAsync();

    private async void OnPrintAcceleratorInvoked(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args)
    {
        args.Handled = true;
        await PrintDocumentAsync();
    }

    private async Task PrintDocumentAsync()
    {
        if (printCancellation is { } pending)
        {
            pending.Cancel();
            StatusText.Text = "인쇄 작업을 취소하는 중…";
            RefreshButtons();
            return;
        }

        await SettleDirectInputAsync();
        if (!DocumentActionsAvailable || page is not { PageCount: > 0 } current)
            return;

        using var operation = new CancellationTokenSource();
        printCancellation = operation;
        editorBusy = true;
        RefreshButtons();
        try
        {
            // Let the File menu dismiss before entering the Windows modal loop.
            await Task.Yield();
            operation.Token.ThrowIfCancellationRequested();
            nint owner = Win32Interop.GetWindowFromWindowId(AppWindow.Id);
            using WindowsPrintJob? job = WindowsPrintJob.ShowDialog(owner, current);
            if (job is null)
                return;

            if (page is not { } active || active.Target != current.Target || active.FontGeneration != current.FontGeneration)
                throw new InvalidOperationException("인쇄할 문서가 바뀌었습니다.");

            string title = fileName ?? "RHWP";
            StatusText.Text = "인쇄 중…";
            await Task.Run(() => job.Print(worker, current, title, operation.Token));
            StatusText.Text = "인쇄 작업을 프린터로 보냈습니다.";
        }
        catch (OperationCanceledException)
        {
            StatusText.Text = "인쇄를 취소했습니다.";
        }
        catch (Exception error)
        {
            StatusText.Text = "인쇄할 수 없습니다. " + error.Message;
        }
        finally
        {
            printCancellation = null;
            editorBusy = false;
            RefreshButtons();
        }
    }
}
