using Microsoft.UI.Text;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Automation.Peers;
using Microsoft.UI.Xaml.Controls;
using Windows.Globalization.NumberFormatting;

namespace Rhwp.WinUI;

internal sealed class TableObjectDialog : UserControl
{
    private const double UnitsPerMillimeter = 7200.0 / 25.4;
    private readonly TextBlock validation = new() { Visibility = Visibility.Collapsed, TextWrapping = TextWrapping.Wrap };
    private readonly Func<bool> apply;

    public bool CanApply { get; }
    public (uint Width, uint Height)? SizeChange { get; private set; }
    public TableObjectPropertyChanges? PropertyChanges { get; private set; }

    public TableObjectDialog(NativeTableObject table, bool resize)
    {
        bool editable = CanApply = resize ? table.CanResize : table.Editable;
        AutomationProperties.SetAutomationId(this, resize ? "TableObjectSizeDialog" : "TableObjectPropertiesDialog");
        AutomationProperties.SetLiveSetting(validation, AutomationLiveSetting.Polite);

        var panel = new StackPanel { Spacing = 12, MinWidth = 300 };
        panel.Children.Add(new TextBlock { Text = $"{table.RowCount}행 × {table.ColumnCount}열" });
        NativeTableObjectProperties original = table.Properties;
        if (resize)
        {
            NumberBox width = Measurement("너비 (mm)", "TableObjectWidth", original.TableWidth, int.MaxValue);
            NumberBox height = Measurement("높이 (mm)", "TableObjectHeight", original.TableHeight, int.MaxValue);
            panel.Children.Add(Pair(width, height));
            apply = () =>
            {
                if (!TryRead(width, original.TableWidth, int.MaxValue, out long widthUnits) || widthUnits == 0 ||
                    !TryRead(height, original.TableHeight, int.MaxValue, out long heightUnits) || heightUnits == 0)
                {
                    return Reject("너비와 높이를 0보다 큰 값으로 입력하세요.");
                }
                SizeChange = ((uint)widthUnits, (uint)heightUnits);
                return true;
            };
        }
        else
        {
            var pageBreak = new ComboBox
            {
                Header = "쪽 경계에서", HorizontalAlignment = HorizontalAlignment.Stretch, IsEnabled = editable,
            };
            pageBreak.Items.Add("나누지 않음");
            pageBreak.Items.Add("셀 단위로 나눔");
            pageBreak.Items.Add("글줄 단위로 나눔");
            pageBreak.SelectedIndex = original.PageBreak;
            AutomationProperties.SetAutomationId(pageBreak, "TableObjectPageBreak");
            var repeat = new CheckBox { Content = "제목 줄 반복", IsChecked = original.RepeatHeader, IsEnabled = editable };
            AutomationProperties.SetAutomationId(repeat, "TableObjectRepeatHeader");
            panel.Children.Add(Pair(pageBreak, repeat));

            NumberBox spacing = Margin("셀 간격 (mm)", "CellSpacing", original.CellSpacing);
            panel.Children.Add(spacing);

            (string Label, string Id, short Value)[] fields =
            [
                ("왼쪽", "PaddingLeft", original.PaddingLeft), ("오른쪽", "PaddingRight", original.PaddingRight),
                ("위쪽", "PaddingTop", original.PaddingTop), ("아래쪽", "PaddingBottom", original.PaddingBottom),
                ("왼쪽", "OuterLeft", original.OuterLeft), ("오른쪽", "OuterRight", original.OuterRight),
                ("위쪽", "OuterTop", original.OuterTop), ("아래쪽", "OuterBottom", original.OuterBottom),
            ];
            NumberBox[] boxes = fields.Select(field => Margin(field.Label, field.Id, field.Value)).ToArray();
            panel.Children.Add(Group("셀 기본 여백 (mm)", boxes[0], boxes[1], boxes[2], boxes[3]));
            panel.Children.Add(Group("표 바깥 여백 (mm)", boxes[4], boxes[5], boxes[6], boxes[7]));
            apply = () =>
            {
                if (pageBreak.SelectedIndex < 0 || !TryRead(spacing, original.CellSpacing, short.MaxValue, out long gap))
                {
                    return Reject("쪽 나눔과 셀 간격을 확인하세요.");
                }
                var values = new short[boxes.Length];
                for (int index = 0; index < boxes.Length; index++)
                {
                    if (!TryRead(boxes[index], fields[index].Value, short.MaxValue, out long units))
                    {
                        return Reject("여백은 0 이상으로 입력하세요.");
                    }
                    values[index] = (short)units;
                }
                PropertyChanges = new((short)gap, values[0], values[1], values[2], values[3],
                    values[4], values[5], values[6], values[7], (byte)pageBreak.SelectedIndex, repeat.IsChecked == true);
                return true;
            };
        }

        if (!editable)
        {
            var readOnly = new TextBlock { Text = "이 표는 속성을 읽을 수 있습니다. 표나 셀이 보호되어 있어 변경할 수 없습니다.",
                TextWrapping = TextWrapping.Wrap };
            var content = new StackPanel { Spacing = 8 };
            content.Children.Add(readOnly);
            content.Children.Add(panel);
            Content = new ScrollViewer { Content = content, MaxHeight = 370 };
        }
        else
        {
            panel.Children.Add(validation);
            Content = new ScrollViewer { Content = panel, MaxHeight = resize ? 160 : 370 };
        }

        NumberBox Margin(string label, string id, short value) =>
            Measurement(label, "TableObject" + id, value, short.MaxValue);

        NumberBox Measurement(string label, string id, long original, long maximum)
        {
            var box = new NumberBox
            {
                Header = label,
                HorizontalAlignment = HorizontalAlignment.Stretch,
                Minimum = Math.Min(0, ToMillimeters(original)),
                Maximum = ToMillimeters(maximum),
                Value = ToMillimeters(original),
                NumberFormatter = new DecimalFormatter { FractionDigits = 2 },
                SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact,
                SmallChange = 0.1,
                LargeChange = 1,
                IsEnabled = editable,
            };
            AutomationProperties.SetName(box, label);
            AutomationProperties.SetAutomationId(box, id);
            return box;
        }
    }

    private static Grid Pair(FrameworkElement left, FrameworkElement right)
    {
        var grid = new Grid { ColumnSpacing = 12 };
        grid.ColumnDefinitions.Add(new ColumnDefinition());
        grid.ColumnDefinitions.Add(new ColumnDefinition());
        grid.Children.Add(left);
        grid.Children.Add(right);
        Grid.SetColumn(right, 1);
        return grid;
    }

    private static StackPanel Group(string title, NumberBox left, NumberBox right, NumberBox top, NumberBox bottom)
    {
        var group = new StackPanel { Spacing = 8 };
        group.Children.Add(new TextBlock { Text = title, FontWeight = FontWeights.SemiBold });
        foreach (NumberBox field in new[] { left, right, top, bottom })
            AutomationProperties.SetName(field, $"{title} {field.Header}");
        group.Children.Add(Pair(left, right));
        group.Children.Add(Pair(top, bottom));
        return group;
    }

    private static double ToMillimeters(long units) => Math.Round(units / UnitsPerMillimeter, 2);

    private static bool TryRead(NumberBox box, long previous, long maximum, out long units)
    {
        units = previous;
        if (box.Value == ToMillimeters(previous))
            return true;
        double rounded = Math.Round(box.Value * UnitsPerMillimeter, MidpointRounding.AwayFromZero);
        if (!(rounded >= 0 && rounded <= maximum))
            return false;
        units = (long)rounded;
        return true;
    }

    public bool TryApply() => CanApply && apply();

    private bool Reject(string message)
    {
        validation.Text = message;
        validation.Visibility = Visibility.Visible;
        validation.StartBringIntoView();
        return false;
    }
}
