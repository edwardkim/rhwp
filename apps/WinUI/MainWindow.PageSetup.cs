using System.Diagnostics;
using Microsoft.UI.Xaml;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private bool pageSetupAvailable;

    private async Task LoadPageSetupCapabilityAsync()
    {
        try
        {
            pageSetupAvailable = await worker.SupportsPageSetupAsync(CancellationToken.None);
            if (!closing && pageSetupAvailable)
            {
                PageMenu.Visibility = Visibility.Visible;
                RefreshPageSetupAction();
            }
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
        }
    }

    private void RefreshPageSetupAction()
    {
        if (PageSetupMenuItem is not null)
            PageSetupMenuItem.IsEnabled = pageSetupAvailable && DocumentActionsAvailable && page is not null;
    }

    private async void OnPageSetupClicked(object sender, RoutedEventArgs args)
    {
        await SettleDirectInputAsync();
        if (!pageSetupAvailable || !DocumentActionsAvailable || page is not { } current)
            return;

        editorBusy = true;
        RefreshButtons();
        try
        {
            CancellationToken token = CancellationToken.None;
            NativePageSetup setup = await worker.GetPageSetupAsync(current, token);
            var dialog = new PageSetupDialog(setup.Settings);
            if (!await PropertyWindow.ShowAsync(this, "쪽 설정", dialog, dialog.TryApply, height: 560) ||
                dialog.Result is not { } settings || settings == setup.Settings || closing ||
                page is not { } active || active.Target != current.Target || active.FontGeneration != current.FontGeneration)
                return;

            if (editorInfo is not { Editing: true } info || info.Target != current.Target)
            {
                if (setup.StartPosition is not { } start)
                {
                    StatusText.Text = "현재 구역에서 편집할 수 있는 본문 위치를 찾지 못했습니다.";
                    return;
                }
                editorInfo = await worker.BeginEditAsync(current.Target, new TextSelection(start, start), token);
            }

            EditResult edit = await worker.SetPageSetupAsync(current.Target, setup.Section, settings, token);
            await RefreshEditedPageAsync(active, edit);
            await UpdateDirectGeometryAsync(followCaret: false, focus: directEditing);
        }
        catch (Exception error)
        {
            StatusText.Text = "쪽 설정을 적용할 수 없습니다. " + error.Message;
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
        }
    }
}
