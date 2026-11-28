using Microsoft.UI.Xaml;
using Windows.ApplicationModel.DataTransfer;
using Windows.System;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private void OnCopyClicked(object sender, RoutedEventArgs args) => EnqueueClipboardAction(VirtualKey.C);
    private void OnCutClicked(object sender, RoutedEventArgs args) => EnqueueClipboardAction(VirtualKey.X);
    private void OnPasteClicked(object sender, RoutedEventArgs args) => EnqueueClipboardAction(VirtualKey.V);

    private void EnqueueClipboardAction(VirtualKey key)
    {
        if (!HasActiveComposition)
            _ = EnqueueDirectEdit(() => ApplyClipboardAsync(key));
    }

    private async Task ApplyClipboardAsync(VirtualKey key)
    {
        if (!directEditing || page is not PageInfo current || editorInfo?.Selection is not { } selection)
            return;

        CancellationToken token = CancellationToken.None;
        EditResult edit;
        if (key == VirtualKey.V)
        {
            DataPackageView source = Clipboard.GetContent();
            if (selection.Focus.CellPath.Length == 0 && source.Contains(StandardDataFormats.Html))
            {
                string html = HtmlFormatHelper.GetStaticFragment(await source.GetHtmlFormatAsync());
                if (html.Length == 0)
                    return;
                edit = await worker.PasteHtmlAsync(current.Target, html, token);
            }
            else if (source.Contains(StandardDataFormats.Text))
            {
                string text = await source.GetTextAsync();
                if (text.Length == 0)
                    return;
                edit = await worker.PasteTextAsync(current.Target, text, token);
            }
            else
            {
                return;
            }
        }
        else
        {
            if (SamePosition(selection.Anchor, selection.Focus))
                return;

            ClipboardSelection copied = await worker.CopySelectionAsync(current.Target, token);
            var data = new DataPackage();
            data.SetText(copied.PlainText);
            if (!string.IsNullOrEmpty(copied.Html))
                data.SetHtmlFormat(HtmlFormatHelper.CreateHtmlFormat(copied.Html));
            Clipboard.SetContent(data);
            Clipboard.Flush();

            if (key == VirtualKey.C)
                return;
            // The system clipboard is written before removing the exact native selection.
            edit = await worker.CutSelectionAsync(current.Target, selection, token);
        }

        preferredX = null;
        await RefreshEditedPageAsync(current, edit);
        await UpdateDirectGeometryAsync(followCaret: true);
        RestorePageEditFocus();
    }

    private void RefreshClipboardActions(bool available)
    {
        bool canPaste = available && directEditing && editorInfo?.Selection is not null;
        bool hasRange = canPaste && editorInfo?.Selection is { } selection &&
            !SamePosition(selection.Anchor, selection.Focus);
        CopyMenuItem.IsEnabled = hasRange;
        CutMenuItem.IsEnabled = hasRange;
        PasteMenuItem.IsEnabled = canPaste;
    }
}
