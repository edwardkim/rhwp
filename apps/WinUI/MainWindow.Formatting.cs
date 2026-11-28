using System.Diagnostics;
using System.Diagnostics.CodeAnalysis;
using System.Globalization;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Windows.System;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private bool formattingCapabilityChecked;
    private bool basicFormattingAvailable;

    private async void OnFormattingToolbarLoaded(object sender, RoutedEventArgs args)
    {
        if (formattingCapabilityChecked || closing)
            return;

        formattingCapabilityChecked = true;
        try
        {
            basicFormattingAvailable = await worker.SupportsBasicTextFormattingAsync(CancellationToken.None);
            if (!closing)
            {
                FormattingToolbar.Visibility = basicFormattingAvailable ? Visibility.Visible : Visibility.Collapsed;
                RefreshFormattingActions();
            }
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
        }
    }

    private void RefreshFormattingActions()
    {
        // XAML can raise text notifications while InitializeComponent builds the row.
        if (ApplyFontSizeButton is null)
            return;

        bool canAlign = TryGetFormattingSelection(requireSelectedText: false, out _, out TextSelection? selection) &&
            DocumentActionsAvailable;
        bool canFormatText = canAlign && selection!.Anchor.ScalarOffset != selection.Focus.ScalarOffset;
        ApplyBoldButton.IsEnabled = canFormatText;
        FormattingFontSize.IsEnabled = canFormatText;
        ApplyFontSizeButton.IsEnabled = canFormatText && TryReadFontSize(out _);
        AlignLeftButton.IsEnabled = canAlign;
        AlignCenterButton.IsEnabled = canAlign;
        AlignRightButton.IsEnabled = canAlign;
        AlignJustifyButton.IsEnabled = canAlign;
        MoreAlignmentButton.IsEnabled = canAlign;
    }

    private async void OnApplyBoldClicked(SplitButton sender, SplitButtonClickEventArgs args) =>
        await QueueFormattingAsync(requireSelectedText: true,
            target => worker.FormatCharactersAsync(target, bold: true, fontSize: null, CancellationToken.None));

    private async void OnRemoveBoldClicked(object sender, RoutedEventArgs args) =>
        await QueueFormattingAsync(requireSelectedText: true,
            target => worker.FormatCharactersAsync(target, bold: false, fontSize: null, CancellationToken.None));

    private void OnFormattingSizeTextChanged(object sender, TextChangedEventArgs args) =>
        RefreshFormattingActions();

    private async void OnFormattingSizeKeyDown(object sender, KeyRoutedEventArgs args)
    {
        if (args.Key != VirtualKey.Enter || !ApplyFontSizeButton.IsEnabled)
            return;

        args.Handled = true;
        await ApplyFormattingFontSizeAsync();
    }

    private async void OnApplyFontSizeClicked(object sender, RoutedEventArgs args) =>
        await ApplyFormattingFontSizeAsync();

    private async Task ApplyFormattingFontSizeAsync()
    {
        if (TryReadFontSize(out int size))
        {
            await QueueFormattingAsync(requireSelectedText: true,
                target => worker.FormatCharactersAsync(target, bold: null, fontSize: size, CancellationToken.None));
        }
    }

    private bool TryReadFontSize(out int result)
    {
        result = 0;
        if (!decimal.TryParse(FormattingFontSize.Text.Trim(), NumberStyles.AllowDecimalPoint,
            CultureInfo.CurrentCulture, out decimal points) || points is < 0.01m or > 4096m)
            return false;

        decimal units = points * 100;
        if (decimal.Truncate(units) != units)
            return false;

        result = (int)units;
        return true;
    }

    private async void OnAlignmentClicked(object sender, RoutedEventArgs args)
    {
        if (sender is FrameworkElement { Tag: string value } &&
            Enum.TryParse(value, out NativeParagraphAlignment alignment))
        {
            await QueueFormattingAsync(requireSelectedText: false,
                target => worker.SetParagraphAlignmentAsync(target, alignment, CancellationToken.None));
        }
    }

    private async Task QueueFormattingAsync(bool requireSelectedText, Func<DocumentTarget, Task<EditResult>> execute)
    {
        if (!DocumentActionsAvailable ||
            !TryGetFormattingSelection(requireSelectedText, out PageInfo? initialPage, out TextSelection? requested))
            return;

        ulong documentId = initialPage.Target.DocumentId;
        await EnqueueDirectEdit(async () =>
        {
            if (!TryGetFormattingSelection(requireSelectedText, out PageInfo? currentPage, out TextSelection? current) ||
                currentPage.Target.DocumentId != documentId || !SamePosition(requested.Anchor, current.Anchor) ||
                !SamePosition(requested.Focus, current.Focus))
                return;

            EditResult edit = await execute(currentPage.Target);
            if (edit.Outcome.DocumentChanged)
            {
                await RefreshEditedPageAsync(currentPage, edit);
                await UpdateDirectGeometryAsync(followCaret: true, focus: true);
            }
            RefreshFormattingActions();
        });
    }

    private bool TryGetFormattingSelection(bool requireSelectedText, [NotNullWhen(true)] out PageInfo? currentPage,
        [NotNullWhen(true)] out TextSelection? selection)
    {
        currentPage = page;
        selection = editorInfo?.Selection;
        return !closing && basicFormattingAvailable && directEditing && !HasActiveComposition &&
            currentPage is not null && selection is not null &&
            editorInfo is { Editing: true, CompositionId: null } info &&
            info.Target == currentPage.Target && info.FontGeneration == currentPage.FontGeneration &&
            SharesTextLeaf(selection.Anchor, selection.Focus) &&
            (!requireSelectedText || selection.Anchor.ScalarOffset != selection.Focus.ScalarOffset);
    }

}
