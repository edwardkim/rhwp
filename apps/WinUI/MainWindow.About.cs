using System.Reflection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private async void OnAboutClicked(object sender, RoutedEventArgs args)
    {
        if (!DocumentActionsAvailable || Root.XamlRoot is null)
            return;

        Assembly assembly = typeof(MainWindow).Assembly;
        string? version = assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion;
        var content = new StackPanel { Spacing = 8 };
        content.Children.Add(new TextBlock { Text = "HWP · HWPX 문서 편집기", TextWrapping = TextWrapping.Wrap });
        if (!string.IsNullOrEmpty(version))
        {
            string[] parts = version.Split('+', 2);
            content.Children.Add(new TextBlock { Text = "버전 " + parts[0] });
            if (parts.Length == 2)
                content.Children.Add(new TextBlock { Text = "빌드 " + parts[1][..Math.Min(parts[1].Length, 8)] });
        }
        content.Children.Add(new HyperlinkButton
        {
            Content = "rhwp 프로젝트와 라이선스",
            NavigateUri = new Uri("https://github.com/edwardkim/rhwp"),
        });

        var dialog = new ContentDialog
        {
            XamlRoot = Root.XamlRoot,
            Title = "RHWP",
            Content = content,
            CloseButtonText = "닫기",
            DefaultButton = ContentDialogButton.Close,
        };
        AutomationProperties.SetAutomationId(dialog, "AboutAppDialog");
        editorBusy = true;
        RefreshButtons();
        try
        {
            await dialog.ShowAsync();
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
        }
    }
}
