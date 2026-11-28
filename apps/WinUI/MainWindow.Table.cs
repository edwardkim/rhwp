using System.Diagnostics;
using System.Diagnostics.CodeAnalysis;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Automation.Peers;
using Microsoft.UI.Xaml.Controls;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private NativeTableLimits? tableLimits;

    private async Task LoadTableCapabilityAsync()
    {
        if (closing)
            return;

        try
        {
            tableLimits = await worker.GetTableLimitsAsync(CancellationToken.None);
            if (!closing && tableLimits is not null)
            {
                InsertMenu.Visibility = Visibility.Visible;
                RefreshTableActions();
            }
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
        }
    }

    private void RefreshTableActions()
    {
        if (!directQueueRunning && InsertTableMenuItem is not null)
            InsertTableMenuItem.IsEnabled = DocumentActionsAvailable && TryGetTableCaret(out _, out _);
    }

    private bool TryGetTableCaret([NotNullWhen(true)] out PageInfo? current, [NotNullWhen(true)] out TextPosition? caret)
    {
        current = page;
        caret = null;
        if (tableLimits is null || closing || !directEditing || HasActiveComposition || current is null ||
            editorInfo is not { Editing: true, CompositionId: null, Selection: { } selection } info ||
            info.Target != current.Target || info.FontGeneration != current.FontGeneration ||
            selection.Anchor.CellPath.Length != 0 || !SamePosition(selection.Anchor, selection.Focus))
            return false;

        caret = selection.Anchor;
        return true;
    }

    private async void OnInsertTableClicked(object sender, RoutedEventArgs args)
    {
        await SettleDirectInputAsync();
        if (!DocumentActionsAvailable || tableLimits is not { } limits ||
            !TryGetTableCaret(out PageInfo? requestedPage, out TextPosition? requestedCaret))
            return;

        (int Rows, int Columns)? size = await ShowTableSizeDialogAsync(limits);
        if (size is not { } dimensions || !DocumentActionsAvailable)
            return;

        await EnqueueDirectEdit(async () =>
        {
            if (!TryGetTableCaret(out PageInfo? current, out TextPosition? caret) ||
                current.Target != requestedPage.Target || !SamePosition(requestedCaret, caret))
                return;

            EditResult edit = await worker.CreateTableAsync(current.Target, dimensions.Rows, dimensions.Columns,
                CancellationToken.None);
            await RefreshEditedPageAsync(current, edit);
            await UpdateDirectGeometryAsync(followCaret: true, focus: true);
        });
    }

    private async Task<(int Rows, int Columns)?> ShowTableSizeDialogAsync(NativeTableLimits limits)
    {
        var rows = CreateTableDimensionBox("줄 수", "TableRows", Math.Min(limits.Rows, limits.Cells));
        var columns = CreateTableDimensionBox("칸 수", "TableColumns", Math.Min(limits.Columns, limits.Cells));
        if (!limits.Contains(rows.Value, columns.Value))
            rows.Value = columns.Value = 1;

        var dimensions = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 12 };
        dimensions.Children.Add(rows);
        dimensions.Children.Add(columns);
        var validation = new TextBlock { TextWrapping = TextWrapping.Wrap, Visibility = Visibility.Collapsed };
        AutomationProperties.SetLiveSetting(validation, AutomationLiveSetting.Polite);
        var content = new StackPanel { Spacing = 8 };
        content.Children.Add(dimensions);
        content.Children.Add(validation);

        var dialog = new ContentDialog
        {
            XamlRoot = Root.XamlRoot,
            Title = "표 삽입",
            Content = content,
            PrimaryButtonText = "삽입",
            CloseButtonText = "취소",
            DefaultButton = ContentDialogButton.Primary,
        };
        dialog.PrimaryButtonClick += (_, args) =>
        {
            if (limits.Contains(rows.Value, columns.Value))
                return;

            args.Cancel = true;
            validation.Text = $"줄과 칸의 수를 정수로 입력하세요. 셀은 최대 {limits.Cells:N0}개입니다.";
            validation.Visibility = Visibility.Visible;
        };

        editorBusy = true;
        RefreshButtons();
        try
        {
            return await dialog.ShowAsync() == ContentDialogResult.Primary ? ((int)rows.Value, (int)columns.Value) : null;
        }
        catch (Exception error)
        {
            StatusText.Text = "표 삽입 창을 열 수 없습니다. " + error.Message;
            return null;
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
        }
    }

    private static NumberBox CreateTableDimensionBox(string label, string automationId, int maximum)
    {
        var box = new NumberBox
        {
            Header = label, Minimum = 1, Maximum = maximum, Value = Math.Min(2, maximum),
            SmallChange = 1, LargeChange = 10, Width = 120, SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact,
        };
        AutomationProperties.SetName(box, label);
        AutomationProperties.SetAutomationId(box, automationId);
        return box;
    }
}
