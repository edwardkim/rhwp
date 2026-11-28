using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Automation.Peers;
using Microsoft.UI.Xaml.Controls;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private NativeShapeLimits? shapeLimits;

    private async void OnInsertShapeClicked(object sender, RoutedEventArgs args)
    {
        await SettleDirectInputAsync();
        if (!DocumentActionsAvailable || shapeLimits is not { } limits ||
            !TryGetBodyInsertionCaret(out PageInfo? requested, out TextPosition? requestedCaret))
            return;

        if (await ShowDrawingShapeDialogAsync(limits) is not { } shape || !DocumentActionsAvailable)
            return;

        await EnqueueDirectEdit(async () =>
        {
            if (!TryGetBodyInsertionCaret(out PageInfo? current, out TextPosition? caret) ||
                current.Target != requested.Target || current.FontGeneration != requested.FontGeneration ||
                !SamePosition(requestedCaret, caret))
                return;

            EditResult edit = await worker.InsertDrawingShapeAsync(current.Target, shape.Kind, shape.Width, shape.Height,
                CancellationToken.None);
            await RefreshEditedPageAsync(current, edit);
            await UpdateDirectGeometryAsync(followCaret: true, focus: true);
        });
    }

    private async Task<(NativeDrawingShape Kind, int Width, int Height)?> ShowDrawingShapeDialogAsync(NativeShapeLimits limits)
    {
        const double unitsPerMillimeter = 7200.0 / 25.4;
        double minimum = limits.MinimumSize / unitsPerMillimeter;
        double maximum = limits.MaximumSize / unitsPerMillimeter;
        var shapes = new ComboBox
        {
            Header = "도형", HorizontalAlignment = HorizontalAlignment.Stretch,
            Items =
            {
                new ComboBoxItem { Content = "직사각형", Tag = NativeDrawingShape.Rectangle },
                new ComboBoxItem { Content = "타원", Tag = NativeDrawingShape.Ellipse },
                new ComboBoxItem { Content = "선", Tag = NativeDrawingShape.Line },
            },
            SelectedIndex = 0,
        };
        AutomationProperties.SetAutomationId(shapes, "ShapeKind");
        NumberBox width = Dimension("너비 (mm)", "ShapeWidth", 30);
        NumberBox height = Dimension("높이 (mm)", "ShapeHeight", 20);
        var sizes = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 12, Children = { width, height } };
        var validation = new TextBlock { TextWrapping = TextWrapping.Wrap, Visibility = Visibility.Collapsed };
        AutomationProperties.SetLiveSetting(validation, AutomationLiveSetting.Polite);
        var content = new StackPanel { Spacing = 12, Children = { shapes, sizes, validation } };
        var dialog = new ContentDialog
        {
            Title = "도형 삽입", XamlRoot = Root.XamlRoot, Content = content, PrimaryButtonText = "삽입",
            CloseButtonText = "취소", DefaultButton = ContentDialogButton.Primary,
        };
        (NativeDrawingShape Kind, int Width, int Height)? choice = null;
        dialog.PrimaryButtonClick += (_, args) =>
        {
            double w = Math.Round(width.Value * unitsPerMillimeter);
            double h = Math.Round(height.Value * unitsPerMillimeter);
            if (shapes.SelectedItem is ComboBoxItem { Tag: NativeDrawingShape kind } &&
                w >= limits.MinimumSize && w <= limits.MaximumSize && h >= limits.MinimumSize && h <= limits.MaximumSize)
            {
                choice = (kind, (int)w, (int)h);
                return;
            }
            args.Cancel = true;
            validation.Text = $"너비와 높이를 {minimum:g3}–{maximum:g3}mm 범위로 입력하세요.";
            validation.Visibility = Visibility.Visible;
        };

        editorBusy = true;
        RefreshButtons();
        try
        {
            return await dialog.ShowAsync() == ContentDialogResult.Primary ? choice : null;
        }
        catch (Exception error)
        {
            StatusText.Text = "도형 삽입 창을 열 수 없습니다. " + error.Message;
            return null;
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
        }

        NumberBox Dimension(string label, string automationId, double value)
        {
            var dimension = new NumberBox
            {
                Header = label, Minimum = minimum, Maximum = maximum, Value = Math.Clamp(value, minimum, maximum),
                SmallChange = 1, Width = 120, SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact,
            };
            AutomationProperties.SetAutomationId(dimension, automationId);
            return dimension;
        }
    }
}
