using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Automation.Peers;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Shapes;
using Windows.Foundation;
using Windows.Globalization.NumberFormatting;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private const double PictureUnitsPerMillimeter = 7200.0 / 25.4;
    private const string PictureSelectedStatus = "그림 선택됨";
    private sealed record PictureContext(PageInfo Page, NativePictureObject Picture);
    private PictureContext? selectedPicture;

    private async Task<PictureContext?> FindPictureContextAsync(PageInfo sheet, Point location)
    {
        if (pictureLimits is not { MaxDimension: > 0 })
            return null;
        NativePictureObject? picture = await worker.GetPictureAtAsync(sheet, location.X, location.Y,
            CancellationToken.None);
        if (picture is null)
            return null;
        var context = new PictureContext(sheet, picture);
        return IsCurrentPicture(context) ? context : null;
    }

    private bool IsCurrentPicture(PictureContext context) => !closing && page is { } active &&
        context.Page.Target == active.Target && context.Page.FontGeneration == active.FontGeneration;

    private void SelectPictureObject(PictureContext context)
    {
        if (!IsCurrentPicture(context) || HasActiveComposition)
            return;

        ClearTableObjectSelection();
        DismissDirectEditor();
        selectedPicture = context;
        DrawPictureObjectSelection();
        StatusText.Text = PictureSelectedStatus;
        RestorePageEditFocus();
    }

    private void DrawPictureObjectSelection()
    {
        if (selectedPicture is not { } current)
            return;
        if (!IsCurrentPicture(current) || continuousPages is not { } layout ||
            !layout.TryGet(current.Page.Page, out PagePlacement sheet))
        {
            ClearPictureObjectSelection();
            return;
        }

        CaretBounds bounds = current.Picture.Bounds;
        PictureObjectOverlay.Children.Clear();
        var outline = new Rectangle
        {
            Stroke = (Brush)DirectEditOverlay.Resources["DirectSelectionBrush"], StrokeThickness = 2,
            Width = bounds.Width * zoom, Height = bounds.Height * zoom,
        };
        Canvas.SetLeft(outline, (sheet.Left + bounds.X) * zoom);
        Canvas.SetTop(outline, (sheet.Top + bounds.Y) * zoom);
        PictureObjectOverlay.Children.Add(outline);
    }

    private void ClearPictureObjectSelection()
    {
        if (selectedPicture is null)
            return;
        selectedPicture = null;
        PictureObjectOverlay.Children.Clear();
        if (StatusText.Text == PictureSelectedStatus)
            StatusText.Text = "";
    }

    private async Task ShowPicturePropertiesAsync(PictureContext context)
    {
        await SettleDirectInputAsync();
        if (!DocumentActionsAvailable || !ReferenceEquals(selectedPicture, context) || !IsCurrentPicture(context) ||
            pictureLimits is not { MaxDimension: > 0 } limits)
            return;

        editorBusy = true;
        RefreshButtons();
        try
        {
            if (await ReadPictureSizeAsync(context.Picture, (uint)limits.MaxDimension) is not { } size ||
                !ReferenceEquals(selectedPicture, context) || !IsCurrentPicture(context) || page is not { } current)
                return;

            CancellationToken token = CancellationToken.None;
            if (editorInfo is not { Editing: true } info || info.Target != current.Target)
            {
                NativePageSetup setup = await worker.GetPageSetupAsync(context.Page, token);
                if (setup.StartPosition is not { } start)
                {
                    StatusText.Text = "그림 속성을 적용할 수 있는 본문 위치를 찾지 못했습니다.";
                    return;
                }
                editorInfo = await worker.BeginEditAsync(current.Target, new TextSelection(start, start), token);
            }

            EditResult edit = await worker.ResizePictureObjectAsync(current.Target, context.Picture.Address,
                size.Width, size.Height, token);
            await RefreshEditedPageAsync(current, edit);
            if (edit.Outcome.DocumentChanged && page is { PageCount: > 0 } updated && updated.Target == edit.Target)
            {
                var sheet = updated with { Page = Math.Min(context.Page.Page, updated.PageCount - 1) };
                if (await worker.GetPictureObjectAsync(sheet, context.Picture.Address, token) is { } resized)
                    SelectPictureObject(new PictureContext(sheet, resized));
            }
            StatusText.Text = edit.Outcome.DocumentChanged ? "그림 크기를 변경했습니다." : "그림 크기가 변경되지 않았습니다.";
        }
        catch (Exception error)
        {
            StatusText.Text = "그림 속성을 적용할 수 없습니다. " + error.Message;
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
            RestorePageEditFocus();
        }
    }

    private async Task<(uint Width, uint Height)?> ReadPictureSizeAsync(NativePictureObject picture, uint limit)
    {
        NativePictureProperties properties = picture.Properties;
        double maximum = limit / PictureUnitsPerMillimeter;
        var formatter = new DecimalFormatter { FractionDigits = 2 };
        NumberBox width = Field("너비 (mm)", "PictureWidth", properties.Width);
        NumberBox height = Field("높이 (mm)", "PictureHeight", properties.Height);
        var dimensions = new Grid { ColumnSpacing = 12 };
        dimensions.ColumnDefinitions.Add(new ColumnDefinition());
        dimensions.ColumnDefinitions.Add(new ColumnDefinition());
        dimensions.Children.Add(width);
        dimensions.Children.Add(height);
        Grid.SetColumn(height, 1);

        var keepAspect = new CheckBox
        {
            Content = "가로세로 비율 유지", IsChecked = true, IsEnabled = picture.CanResize,
        };
        AutomationProperties.SetAutomationId(keepAspect, "PictureKeepAspect");
        bool adjusting = false;
        if (properties.Width > 0 && properties.Height > 0)
        {
            double aspect = (double)properties.Width / properties.Height;
            width.ValueChanged += (_, _) => MatchDimension(height, width.Value / aspect);
            height.ValueChanged += (_, _) => MatchDimension(width, height.Value * aspect);
        }

        var validation = new TextBlock { TextWrapping = TextWrapping.Wrap, Visibility = Visibility.Collapsed };
        AutomationProperties.SetLiveSetting(validation, AutomationLiveSetting.Polite);
        var fields = new StackPanel { Spacing = 12, Children = { dimensions, keepAspect, validation } };
        AutomationProperties.SetAutomationId(fields, "PictureProperties");
        if (!picture.CanResize)
        {
            validation.Text = "이 그림의 크기는 읽을 수 있습니다.";
            validation.Visibility = Visibility.Visible;
        }

        (uint Width, uint Height)? result = null;
        if (!await PropertyWindow.ShowAsync(this, "그림 속성", fields, picture.CanResize ? Apply : null, height: 210))
            return null;
        return result;

        bool Apply()
        {
            double w = ReadUnits(width, properties.Width);
            double h = ReadUnits(height, properties.Height);
            if (w >= 1 && w <= limit && h >= 1 && h <= limit)
            {
                result = ((uint)w, (uint)h);
                return true;
            }
            validation.Text = "너비와 높이를 0보다 큰 값으로 입력하세요.";
            validation.Visibility = Visibility.Visible;
            return false;
        }

        NumberBox Field(string label, string id, uint value)
        {
            var box = new NumberBox
            {
                Header = label, Value = ToMillimeters(value), MinWidth = 0, NumberFormatter = formatter,
                Minimum = 0, Maximum = Math.Max(maximum, ToMillimeters(value)), SmallChange = 1,
                IsEnabled = picture.CanResize, SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact,
            };
            AutomationProperties.SetAutomationId(box, id);
            return box;
        }

        void MatchDimension(NumberBox other, double value)
        {
            if (adjusting || keepAspect.IsChecked != true || !double.IsFinite(value))
                return;
            adjusting = true;
            other.Value = Math.Min(maximum, Math.Round(value, 2));
            adjusting = false;
        }
    }

    private static double ToMillimeters(uint value) => Math.Round(value / PictureUnitsPerMillimeter, 2);
    private static double ReadUnits(NumberBox box, uint previous) => box.Value == ToMillimeters(previous)
        ? previous : Math.Round(box.Value * PictureUnitsPerMillimeter, MidpointRounding.AwayFromZero);
}
