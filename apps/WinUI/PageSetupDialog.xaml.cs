using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Windows.Globalization.NumberFormatting;

namespace Rhwp.WinUI;

public sealed partial class PageSetupDialog : ContentDialog
{
    private const double UnitsPerMillimeter = 7200.0 / 25.4;
    private const double MaxMillimeters = 1016;
    private static readonly (double Width, double Height)[] paperSizes =
        [(210, 297), (148, 210), (182, 257), (215.9, 279.4), (215.9, 355.6)];

    private readonly NativePageSettings original;
    private bool updatingFields = true;
    private bool landscape;

    internal NativePageSettings? Result { get; private set; }

    internal PageSetupDialog(NativePageSettings original)
    {
        this.original = original;
        InitializeComponent();

        landscape = original.Landscape;
        Orientation.SelectedIndex = landscape ? 1 : 0;
        Binding.SelectedIndex = (int)original.Binding;
        var formatter = new DecimalFormatter { FractionDigits = 2 };
        SetNumber(PaperWidth, landscape ? original.Height : original.Width, formatter);
        SetNumber(PaperHeight, landscape ? original.Width : original.Height, formatter);
        SetNumber(MarginTop, original.MarginTop, formatter);
        SetNumber(MarginBottom, original.MarginBottom, formatter);
        SetNumber(MarginLeft, original.MarginLeft, formatter);
        SetNumber(MarginRight, original.MarginRight, formatter);
        SetNumber(MarginHeader, original.MarginHeader, formatter);
        SetNumber(MarginFooter, original.MarginFooter, formatter);
        SetNumber(MarginGutter, original.MarginGutter, formatter);
        Preset.SelectedIndex = Array.IndexOf(paperSizes, (ToMillimeters(original.Width), ToMillimeters(original.Height))) + 1;
        updatingFields = false;
        RefreshPreview();
    }

    private static void SetNumber(NumberBox box, uint units, DecimalFormatter formatter)
    {
        box.NumberFormatter = formatter;
        box.Value = ToMillimeters(units);
    }

    private static double ToMillimeters(uint units) => Math.Round(units / UnitsPerMillimeter, 2);

    private static uint ReadUnits(NumberBox box, uint previous) => box.Value == ToMillimeters(previous)
        ? previous : checked((uint)Math.Round(box.Value * UnitsPerMillimeter, MidpointRounding.AwayFromZero));

    private void OnPaperChanged(object sender, SelectionChangedEventArgs args)
    {
        if (updatingFields || Preset.SelectedIndex <= 0)
            return;

        var (width, height) = paperSizes[Preset.SelectedIndex - 1];
        SetDimensions(landscape ? height : width, landscape ? width : height);
    }

    private void OnOrientationChanged(object sender, SelectionChangedEventArgs args)
    {
        if (updatingFields || Orientation.SelectedIndex < 0 || landscape == (Orientation.SelectedIndex == 1))
            return;

        landscape = Orientation.SelectedIndex == 1;
        SetDimensions(PaperHeight.Value, PaperWidth.Value);
    }

    private void SetDimensions(double width, double height)
    {
        updatingFields = true;
        PaperWidth.Value = width;
        PaperHeight.Value = height;
        updatingFields = false;
        RefreshPreview();
    }

    private void OnDimensionChanged(NumberBox sender, NumberBoxValueChangedEventArgs args)
    {
        if (updatingFields)
            return;

        Preset.SelectedIndex = 0;
        RefreshPreview();
    }

    private void OnMarginChanged(NumberBox sender, NumberBoxValueChangedEventArgs args)
    {
        if (!updatingFields)
            RefreshPreview();
    }

    private void RefreshPreview()
    {
        bool valid = IsValid();
        BodyPreview.Visibility = valid ? Visibility.Visible : Visibility.Collapsed;
        if (PaperWidth.Value is not (>= 2.54 and <= MaxMillimeters) || PaperHeight.Value is not (>= 2.54 and <= MaxMillimeters))
            return;

        double scale = 80 / Math.Max(PaperWidth.Value, PaperHeight.Value);
        PaperPreview.Width = PaperWidth.Value * scale;
        PaperPreview.Height = PaperHeight.Value * scale;
        if (valid)
        {
            BodyPreview.Margin = new Thickness((MarginLeft.Value + MarginGutter.Value) * scale,
                (MarginTop.Value + MarginHeader.Value) * scale, MarginRight.Value * scale,
                (MarginBottom.Value + MarginFooter.Value) * scale);
        }
    }

    private bool IsValid() => PaperWidth.Value is >= 2.54 and <= MaxMillimeters &&
        PaperHeight.Value is >= 2.54 and <= MaxMillimeters &&
        MarginLeft.Value >= 0 && MarginRight.Value >= 0 && MarginGutter.Value >= 0 &&
        MarginTop.Value >= 0 && MarginBottom.Value >= 0 && MarginHeader.Value >= 0 && MarginFooter.Value >= 0 &&
        MarginLeft.Value + MarginRight.Value + MarginGutter.Value < PaperWidth.Value &&
        MarginTop.Value + MarginBottom.Value + MarginHeader.Value + MarginFooter.Value < PaperHeight.Value;

    private void OnApply(ContentDialog sender, ContentDialogButtonClickEventArgs args)
    {
        if (!IsValid() || Binding.SelectedIndex < 0)
        {
            args.Cancel = true;
            Validation.Text = "용지 크기와 여백을 확인하세요. 여백을 빼고 본문이 들어갈 공간이 있어야 합니다.";
            Validation.Visibility = Visibility.Visible;
            return;
        }

        Result = new NativePageSettings(
            ReadUnits(landscape ? PaperHeight : PaperWidth, original.Width),
            ReadUnits(landscape ? PaperWidth : PaperHeight, original.Height),
            ReadUnits(MarginLeft, original.MarginLeft), ReadUnits(MarginRight, original.MarginRight),
            ReadUnits(MarginTop, original.MarginTop), ReadUnits(MarginBottom, original.MarginBottom),
            ReadUnits(MarginHeader, original.MarginHeader), ReadUnits(MarginFooter, original.MarginFooter),
            ReadUnits(MarginGutter, original.MarginGutter), landscape, (uint)Binding.SelectedIndex);
    }
}
