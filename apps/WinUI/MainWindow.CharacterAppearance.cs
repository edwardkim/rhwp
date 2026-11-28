using System.Globalization;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Windows.UI;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private const string NoHighlightColor = "#FFFFFF";

    private enum CharacterAppearance
    {
        Font,
        TextColor,
        Highlight,
    }

    private async void OnFormattingFontClicked(object sender, RoutedEventArgs args) =>
        await ChooseCharacterAppearanceAsync(CharacterAppearance.Font);

    private async void OnFormattingTextColorClicked(object sender, RoutedEventArgs args) =>
        await ChooseCharacterAppearanceAsync(CharacterAppearance.TextColor);

    private async void OnFormattingHighlightClicked(object sender, RoutedEventArgs args) =>
        await ChooseCharacterAppearanceAsync(CharacterAppearance.Highlight);

    private async Task ChooseCharacterAppearanceAsync(CharacterAppearance appearance)
    {
        await SettleDirectInputAsync();
        bool supported = appearance switch
        {
            CharacterAppearance.Font => appearanceFormattingAvailable && systemFontChooserAvailable,
            CharacterAppearance.TextColor => appearanceFormattingAvailable,
            CharacterAppearance.Highlight => highlightFormattingAvailable,
            _ => false,
        };
        if (!supported || !DocumentActionsAvailable || dragPointer is not null ||
            !TryGetFormattingSelection(true, out PageInfo? requested, out TextSelection? original))
            return;

        string? value;
        editorBusy = true;
        RefreshButtons();
        try
        {
            NativeSelectionFormatting? current = await worker.GetSelectionFormattingAsync(requested.Target,
                requested.FontGeneration, original, CancellationToken.None);
            value = appearance switch
            {
                CharacterAppearance.Font => await ShowFontChooserAsync(requested.Target, current),
                CharacterAppearance.TextColor => await ShowCharacterColorAsync(current?.TextColor, highlight: false),
                CharacterAppearance.Highlight => await ShowCharacterColorAsync(current?.ShadeColor, highlight: true),
                _ => throw new ArgumentOutOfRangeException(nameof(appearance)),
            };
        }
        catch (Exception error)
        {
            StatusText.Text = "글자 모양을 변경할 수 없습니다. " + error.Message;
            return;
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
        }

        if (value is null)
            return;

        await QueueFormattingAsync(true, (current, _) => appearance switch
            {
                CharacterAppearance.Font => worker.FormatCharacterAppearanceAsync(current.Target, value,
                    textColor: null, CancellationToken.None),
                CharacterAppearance.TextColor => worker.FormatCharacterAppearanceAsync(current.Target, fontName: null,
                    value, CancellationToken.None),
                CharacterAppearance.Highlight => worker.SetCharacterHighlightAsync(current.Target, value,
                    CancellationToken.None),
                _ => throw new ArgumentOutOfRangeException(nameof(appearance)),
            },
            new FormattingContext(requested.Target, requested.FontGeneration, original));
    }

    private async Task<string?> ShowFontChooserAsync(DocumentTarget document, NativeSelectionFormatting? formatting)
    {
        string[] installed = await worker.GetSystemFontFamiliesAsync(document, CancellationToken.None);
        if (installed.Length == 0)
            throw new InvalidOperationException("사용 가능한 시스템 글꼴이 없습니다.");

        string? current = formatting?.FontNames is { Length: > 0 } names && names[0].Kind == "uniform"
            ? names[0].Value : null;
        var search = new TextBox { PlaceholderText = "글꼴 검색" };
        AutomationProperties.SetName(search, "글꼴 검색");
        AutomationProperties.SetAutomationId(search, "FontFamilySearch");
        var families = new ListView { ItemsSource = installed, Height = 224, SelectionMode = ListViewSelectionMode.Single };
        AutomationProperties.SetName(families, "설치된 글꼴");
        AutomationProperties.SetAutomationId(families, "SystemFontFamilies");
        var preview = new TextBlock { FontSize = 20, Text = "한글 Aa 123" };
        var panel = new StackPanel { Spacing = 10, Children = { search, families, preview } };
        var dialog = new ContentDialog
        {
            Title = "글꼴", XamlRoot = Root.XamlRoot, Content = panel,
            PrimaryButtonText = "적용", CloseButtonText = "취소", IsPrimaryButtonEnabled = false,
        };
        families.SelectionChanged += (_, _) =>
        {
            if (families.SelectedItem is string family)
                preview.FontFamily = new FontFamily(family);
            else
                preview.ClearValue(TextBlock.FontFamilyProperty);
            dialog.IsPrimaryButtonEnabled = families.SelectedItem is string;
        };
        search.TextChanged += (_, _) =>
        {
            string? selected = families.SelectedItem as string;
            string query = search.Text.Trim();
            string[] visible = query.Length == 0 ? installed : installed
                .Where(family => family.Contains(query, StringComparison.CurrentCultureIgnoreCase)).ToArray();
            families.ItemsSource = visible;
            if (selected is not null && visible.Contains(selected))
                families.SelectedItem = selected;
        };
        families.SelectedItem = installed.FirstOrDefault(name => string.Equals(name, current,
            StringComparison.OrdinalIgnoreCase));
        families.Loaded += (_, _) =>
        {
            if (families.SelectedItem is string selected)
                families.ScrollIntoView(selected);
            search.Focus(FocusState.Programmatic);
        };

        return await dialog.ShowAsync() == ContentDialogResult.Primary ? families.SelectedItem as string : null;
    }

    private async Task<string?> ShowCharacterColorAsync(NativeFormattingValue<string?>? formatting, bool highlight)
    {
        string? current = formatting is { Kind: "uniform", Value: string value } ? value : null;
        uint rgb = highlight ? 0xFFFF00u : 0;
        if (current is { Length: 7 } && current[0] == '#')
            uint.TryParse(current.AsSpan(1), NumberStyles.HexNumber, CultureInfo.InvariantCulture, out rgb);
        var picker = new ColorPicker
        {
            Color = Color.FromArgb(255, (byte)(rgb >> 16), (byte)(rgb >> 8), (byte)rgb),
            IsAlphaEnabled = false, IsHexInputVisible = true,
        };
        AutomationProperties.SetAutomationId(picker, highlight ? "HighlightColorPicker" : "TextColorPicker");
        var dialog = new ContentDialog
        {
            Title = highlight ? "형광펜" : "글자색", XamlRoot = Root.XamlRoot, Content = picker,
            PrimaryButtonText = "적용", SecondaryButtonText = highlight ? "색 없음" : "", CloseButtonText = "취소",
        };
        ContentDialogResult result = await dialog.ShowAsync();
        if (highlight && result == ContentDialogResult.Secondary)
            return NoHighlightColor;
        if (result != ContentDialogResult.Primary)
            return null;

        Color color = picker.Color;
        string selected = $"#{color.R:X2}{color.G:X2}{color.B:X2}";
        return highlight && string.Equals(current, selected, StringComparison.OrdinalIgnoreCase)
            ? NoHighlightColor : selected;
    }
}
