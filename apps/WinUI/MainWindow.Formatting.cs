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
    private bool appearanceFormattingAvailable;
    private bool systemFontChooserAvailable;

    private async void OnFormattingToolbarLoaded(object sender, RoutedEventArgs args)
    {
        if (formattingCapabilityChecked || closing)
            return;

        formattingCapabilityChecked = true;
        try
        {
            NativeFormattingCapabilities capabilities = await worker.GetFormattingCapabilitiesAsync(CancellationToken.None);
            basicFormattingAvailable = capabilities.Edit;
            formattingInspectionAvailable = capabilities.Inspect;
            appearanceFormattingAvailable = capabilities.FontAndColor;
            systemFontChooserAvailable = capabilities.SystemFonts;
            if (!closing)
            {
                FormattingToolbar.Visibility = basicFormattingAvailable ? Visibility.Visible : Visibility.Collapsed;
                FormattingFontButton.Visibility = appearanceFormattingAvailable && systemFontChooserAvailable
                    ? Visibility.Visible : Visibility.Collapsed;
                FormattingTextColorButton.Visibility = appearanceFormattingAvailable ? Visibility.Visible : Visibility.Collapsed;
                ParagraphStyleButton.Visibility = basicFormattingAvailable && capabilities.ParagraphStyles
                    ? Visibility.Visible : Visibility.Collapsed;
                FormattingMenu.Visibility = ParagraphStyleButton.Visibility;
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
        // A queued cursor move or edit can briefly leave the page and selection
        // on different revisions. Keep the last settled appearance until it completes.
        if (AlignJustifyButton is null || ApplyFontSizeButton is null || directQueueRunning || dragPointer is not null)
            return;

        bool canAlign = TryGetFormattingSelection(requireSelectedText: false, out _, out TextSelection? selection) &&
            DocumentActionsAvailable;
        bool canFormatText = canAlign && selection!.Anchor.ScalarOffset != selection.Focus.ScalarOffset;
        ParagraphStyleButton.IsEnabled = canAlign && selection!.Focus.CellPath.Length <= 1;
        FormattingFontButton.IsEnabled = canFormatText;
        FormattingTextColorButton.IsEnabled = canFormatText;
        ApplyBoldButton.IsEnabled = canFormatText;
        FormattingFontSize.IsEnabled = canAlign;
        FormattingFontSize.IsReadOnly = !canFormatText;
        ApplyFontSizeButton.IsEnabled = canFormatText && TryReadFontSize(out _);
        AlignLeftButton.IsEnabled = canAlign;
        AlignCenterButton.IsEnabled = canAlign;
        AlignRightButton.IsEnabled = canAlign;
        AlignJustifyButton.IsEnabled = canAlign;
        MoreAlignmentButton.IsEnabled = canAlign;
        RefreshSelectionFormatting(canAlign ? page : null, canAlign ? selection : null);
    }

    private async void OnApplyBoldClicked(SplitButton sender, SplitButtonClickEventArgs args) => await ToggleBoldAsync();

    private async void OnBoldAcceleratorInvoked(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args)
    {
        if (!ApplyBoldButton.IsEnabled || !DocumentActionsAvailable || dragPointer is not null)
            return;
        if (FocusManager.GetFocusedElement(Root.XamlRoot) is TextBox focused && focused != DirectInput)
            return;
        args.Handled = true;
        await ToggleBoldAsync();
    }

    private Task ToggleBoldAsync() => QueueFormattingAsync(requireSelectedText: true,
            async (current, selection) =>
            {
                NativeSelectionFormatting? formatting = await worker.GetSelectionFormattingAsync(current.Target,
                    current.FontGeneration, selection, CancellationToken.None);
                bool bold = formatting?.Bold is not { Kind: "uniform", Value: true };
                return await worker.FormatCharactersAsync(current.Target, bold, fontSize: null, CancellationToken.None);
            });

    private async void OnSetBoldClicked(object sender, RoutedEventArgs args) =>
        await ChangeBoldAsync(true);

    private async void OnRemoveBoldClicked(object sender, RoutedEventArgs args) =>
        await ChangeBoldAsync(false);

    private async Task ChangeBoldAsync(bool bold) =>
        await QueueFormattingAsync(requireSelectedText: true,
            (current, _) => worker.FormatCharactersAsync(current.Target, bold, fontSize: null, CancellationToken.None));

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
                (current, _) => worker.FormatCharactersAsync(current.Target, bold: null, fontSize: size, CancellationToken.None));
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
                (current, _) => worker.SetParagraphAlignmentAsync(current.Target, alignment, CancellationToken.None));
        }
    }

    private async Task QueueFormattingAsync(bool requireSelectedText, Func<PageInfo, TextSelection, Task<EditResult>> execute,
        FormattingContext? expected = null)
    {
        if (!DocumentActionsAvailable || dragPointer is not null ||
            !TryGetFormattingSelection(requireSelectedText, out PageInfo? initialPage, out TextSelection? requested) ||
            expected?.Matches(initialPage, requested) == false)
            return;

        ulong documentId = initialPage.Target.DocumentId;
        await EnqueueDirectEdit(async () =>
        {
            if (!TryGetFormattingSelection(requireSelectedText, out PageInfo? currentPage, out TextSelection? current) ||
                currentPage.Target.DocumentId != documentId || !SamePosition(requested.Anchor, current.Anchor) ||
                !SamePosition(requested.Focus, current.Focus) || expected?.Matches(currentPage, current) == false)
                return;

            EditResult edit = await execute(currentPage, current);
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
