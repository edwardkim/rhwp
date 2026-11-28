using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private async void OnParagraphStyleClicked(object sender, RoutedEventArgs args)
    {
        await SettleDirectInputAsync();
        if (ParagraphStyleButton.Visibility != Visibility.Visible || !DocumentActionsAvailable ||
            dragPointer is not null ||
            !TryGetFormattingSelection(false, out PageInfo? requested, out TextSelection? selection) ||
            selection.Focus.CellPath.Length > 1)
            return;

        byte? styleId;
        editorBusy = true;
        RefreshButtons();
        try
        {
            NativeParagraphStyleCatalog catalog = await worker.GetParagraphStylesAsync(requested.Target, selection,
                CancellationToken.None);
            if (catalog.Styles.Length == 0)
            {
                StatusText.Text = "이 문서에서 적용할 수 있는 문단 양식이 없습니다.";
                return;
            }
            styleId = await ShowParagraphStyleDialogAsync(catalog);
        }
        catch (Exception error)
        {
            StatusText.Text = "문단 양식을 불러올 수 없습니다. " + error.Message;
            return;
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
        }

        if (styleId is byte chosen)
        {
            await QueueFormattingAsync(false, (current, _) => worker.ApplyParagraphStyleAsync(current.Target,
                chosen, CancellationToken.None), new FormattingContext(requested.Target, requested.FontGeneration, selection));
        }
    }

    private async Task<byte?> ShowParagraphStyleDialogAsync(NativeParagraphStyleCatalog catalog)
    {
        var styles = new ListView
        {
            ItemsSource = catalog.Styles.Select(style => style.DisplayName).ToArray(),
            SelectionMode = ListViewSelectionMode.Single,
            Height = 240,
        };
        AutomationProperties.SetName(styles, "문단 양식");
        AutomationProperties.SetAutomationId(styles, "ParagraphStyleList");
        var dialog = new ContentDialog
        {
            Title = "문단 양식",
            XamlRoot = Root.XamlRoot,
            Content = styles,
            PrimaryButtonText = "적용",
            CloseButtonText = "취소",
            IsPrimaryButtonEnabled = false,
        };
        styles.SelectionChanged += (_, _) => dialog.IsPrimaryButtonEnabled = styles.SelectedIndex >= 0;
        styles.SelectedIndex = Array.FindIndex(catalog.Styles, style => style.Id == catalog.CurrentStyleId);
        styles.Loaded += (_, _) =>
        {
            if (styles.SelectedItem is string selected)
                styles.ScrollIntoView(selected);
            styles.Focus(FocusState.Programmatic);
        };
        return await dialog.ShowAsync() == ContentDialogResult.Primary && styles.SelectedIndex >= 0
            ? catalog.Styles[styles.SelectedIndex].Id : null;
    }
}
