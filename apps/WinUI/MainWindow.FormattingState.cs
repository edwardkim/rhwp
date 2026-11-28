using System.Diagnostics;
using System.Globalization;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private static readonly TimeSpan FormattingReadDelay = TimeSpan.FromMilliseconds(120);
    private bool formattingInspectionAvailable;
    private bool showingFormatting;
    private FormattingContext? lastFormattingContext;
    private CancellationTokenSource? formattingRead;

    private sealed record FormattingContext(DocumentTarget Target, ulong FontGeneration, TextSelection Selection)
    {
        public bool Matches(PageInfo page, TextSelection selection) => Target == page.Target &&
            FontGeneration == page.FontGeneration && SamePosition(Selection.Anchor, selection.Anchor) &&
            SamePosition(Selection.Focus, selection.Focus);
    }

    private void RefreshSelectionFormatting(PageInfo? current, TextSelection? selection)
    {
        if (!formattingInspectionAvailable || HasActiveComposition)
            return;

        if (current is null || selection is null)
        {
            formattingRead?.Cancel();
            lastFormattingContext = null;
            if (showingFormatting)
                DisplaySelectionFormatting(null);
            return;
        }

        if (lastFormattingContext?.Matches(current, selection) == true)
            return;

        formattingRead?.Cancel();
        var context = new FormattingContext(current.Target, current.FontGeneration, selection);
        lastFormattingContext = context;
        _ = ReadSelectionFormattingAsync(context);
    }

    private async Task ReadSelectionFormattingAsync(FormattingContext context)
    {
        using var request = new CancellationTokenSource();
        formattingRead = request;
        bool displayed = false;
        try
        {
            // Let sustained typing and key repeats use the document worker first.
            await Task.Delay(FormattingReadDelay, request.Token);
            if (directQueueRunning || !TryGetFormattingSelection(false, out PageInfo? current, out TextSelection? selection) ||
                !context.Matches(current, selection))
                return;

            NativeSelectionFormatting? value = await worker.GetSelectionFormattingAsync(context.Target,
                context.FontGeneration, context.Selection, request.Token);
            if (!request.IsCancellationRequested && ReferenceEquals(lastFormattingContext, context) &&
                TryGetFormattingSelection(false, out current, out selection) && context.Matches(current, selection))
            {
                DisplaySelectionFormatting(value);
                displayed = true;
            }
        }
        catch (OperationCanceledException) when (request.IsCancellationRequested)
        {
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
        }
        finally
        {
            if (ReferenceEquals(formattingRead, request))
                formattingRead = null;
            if (!displayed && ReferenceEquals(lastFormattingContext, context))
                lastFormattingContext = null;
        }
    }

    private void DisplaySelectionFormatting(NativeSelectionFormatting? value)
    {
        showingFormatting = value is not null;
        bool bold = value?.Bold is { Kind: "uniform", Value: true };
        string boldStatus = value?.Bold.Kind == "mixed" ? "굵기가 혼합되어 있음" : bold ? "굵게 적용됨" : "";
        SetFormattingIndicator(ApplyBoldButton, bold, boldStatus);

        bool mixedSize = value?.FontSize.Kind == "mixed";
        FormattingFontSize.PlaceholderText = mixedSize ? "혼합" : "pt";
        AutomationProperties.SetItemStatus(FormattingFontSize, mixedSize ? "글자 크기가 혼합되어 있음" : "");
        if (FormattingFontSize.FocusState == FocusState.Unfocused)
        {
            FormattingFontSize.Text = value?.FontSize is { Kind: "uniform", Value: var size }
                ? (size / 100m).ToString(CultureInfo.CurrentCulture)
                : "";
        }

        string? alignment = value?.Alignment;
        ShowAlignment(AlignLeftButton, alignment == "left");
        ShowAlignment(AlignCenterButton, alignment == "center");
        ShowAlignment(AlignRightButton, alignment == "right");
        ShowAlignment(AlignJustifyButton, alignment == "justify");
        ShowAlignment(MoreAlignmentButton, alignment is "distribute" or "split");
    }

    private void ShowAlignment(Control control, bool active) =>
        SetFormattingIndicator(control, active, active ? "현재 문단의 정렬" : "");

    private void SetFormattingIndicator(Control control, bool active, string status)
    {
        if (active)
            control.BorderBrush = (Brush)Root.Resources["ActiveFormattingBrush"];
        else
            control.ClearValue(Control.BorderBrushProperty);
        AutomationProperties.SetItemStatus(control, status);
    }
}
