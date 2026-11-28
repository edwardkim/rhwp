using System.Text;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Automation.Peers;
using Microsoft.UI.Xaml.Controls;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private NativeEquationLimits? equationLimits;

    private async void OnInsertEquationClicked(object sender, RoutedEventArgs args)
    {
        await SettleDirectInputAsync();
        if (!DocumentActionsAvailable || equationLimits is not { } limits ||
            !TryGetBodyInsertionCaret(out PageInfo? requestedPage, out TextPosition? requestedCaret))
            return;

        if (await ShowEquationDialogAsync(limits) is not { } equation || !DocumentActionsAvailable)
            return;

        await EnqueueDirectEdit(async () =>
        {
            if (!TryGetBodyInsertionCaret(out PageInfo? current, out TextPosition? caret) ||
                current.Target != requestedPage.Target || current.FontGeneration != requestedPage.FontGeneration ||
                !SamePosition(requestedCaret, caret))
                return;

            EditResult edit = await worker.InsertEquationAsync(current.Target, equation.Script, equation.FontSize,
                CancellationToken.None);
            await RefreshEditedPageAsync(current, edit);
            await UpdateDirectGeometryAsync(followCaret: true, focus: true);
        });
    }

    private async Task<(string Script, int FontSize)?> ShowEquationDialogAsync(NativeEquationLimits limits)
    {
        const int unitsPerPoint = 100;
        var script = new TextBox
        {
            Header = "한글 수식 스크립트", PlaceholderText = "예  {a + b} over {c}",
            AcceptsReturn = true, TextWrapping = TextWrapping.Wrap, Height = 96,
            MaxLength = limits.ScriptBytes,
        };
        AutomationProperties.SetAutomationId(script, "EquationScript");
        var size = new NumberBox
        {
            Header = "글자 크기 (pt)", Minimum = (double)limits.MinimumSize / unitsPerPoint,
            Maximum = (double)limits.MaximumSize / unitsPerPoint,
            Value = (double)Math.Clamp(10 * unitsPerPoint, limits.MinimumSize, limits.MaximumSize) / unitsPerPoint,
            SmallChange = 1, SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact,
            HorizontalAlignment = HorizontalAlignment.Left, Width = 128,
        };
        AutomationProperties.SetAutomationId(size, "EquationFontSize");
        var validation = new TextBlock { TextWrapping = TextWrapping.Wrap, Visibility = Visibility.Collapsed };
        AutomationProperties.SetLiveSetting(validation, AutomationLiveSetting.Polite);
        var content = new StackPanel { Spacing = 12, Children = { script, size, validation } };
        var dialog = new ContentDialog
        {
            XamlRoot = Root.XamlRoot, Title = "수식 삽입", Content = content,
            PrimaryButtonText = "삽입", CloseButtonText = "취소",
        };
        (string Script, int FontSize)? equation = null;
        dialog.PrimaryButtonClick += (_, args) =>
        {
            double fontSize = Math.Round(size.Value * unitsPerPoint);
            string value = script.Text;
            if (!string.IsNullOrWhiteSpace(value) && Encoding.UTF8.GetByteCount(value) <= limits.ScriptBytes &&
                double.IsFinite(fontSize) && fontSize >= limits.MinimumSize && fontSize <= limits.MaximumSize)
            {
                equation = (value, (int)fontSize);
                return;
            }

            args.Cancel = true;
            validation.Text = $"수식 스크립트와 {size.Minimum:g}–{size.Maximum:g}pt 범위의 글자 크기를 입력하세요.";
            validation.Visibility = Visibility.Visible;
        };

        editorBusy = true;
        RefreshButtons();
        try
        {
            return await dialog.ShowAsync() == ContentDialogResult.Primary ? equation : null;
        }
        catch (Exception error)
        {
            StatusText.Text = "수식 입력 창을 열 수 없습니다. " + error.Message;
            return null;
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
        }
    }
}
