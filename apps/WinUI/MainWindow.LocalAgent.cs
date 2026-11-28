using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private ContentDialog? localAgentDialog;

    private sealed record LocalAgentSelection(DocumentTarget Target, TextSelection Range)
    {
        public bool Matches(NativeDocumentInfo? info) =>
            info is { Editing: true, CompositionId: null, Selection: { } range } && info.Target == Target &&
            SamePosition(Range.Anchor, range.Anchor) && SamePosition(Range.Focus, range.Focus);
    }

    private void RefreshLocalAgentActions(bool available)
    {
        bool canSend = available && directEditing && dragPointer is null && editorInfo?.Selection is { } selection &&
            !SamePosition(selection.Anchor, selection.Focus);
        CodexSelectionMenuItem.IsEnabled = canSend;
        ClaudeSelectionMenuItem.IsEnabled = canSend;
    }

    private async void OnCodexSelectionClicked(object sender, RoutedEventArgs args) =>
        await AskLocalAgentAsync(LocalAgentProvider.Codex);

    private async void OnClaudeSelectionClicked(object sender, RoutedEventArgs args) =>
        await AskLocalAgentAsync(LocalAgentProvider.Claude);

    private async Task AskLocalAgentAsync(LocalAgentProvider provider)
    {
        if (!DocumentActionsAvailable || !directEditing || dragPointer is not null || Root.XamlRoot is null ||
            page is not PageInfo current || editorInfo?.Selection is not { } range ||
            SamePosition(range.Anchor, range.Focus))
            return;

        var selected = new LocalAgentSelection(current.Target, range);
        if (!selected.Matches(editorInfo))
            return;

        string name = LocalAgentCli.DisplayName(provider);
        string? replacement;
        editorBusy = true;
        RefreshButtons();
        try
        {
            ClipboardSelection captured = await worker.CopySelectionAsync(selected.Target, CancellationToken.None);
            if (string.IsNullOrEmpty(captured.PlainText) || captured.PlainText.Length > LocalAgentCli.MaxSelectionLength)
            {
                StatusText.Text = "선택한 텍스트를 보낼 수 없습니다. 더 짧은 텍스트를 선택하세요.";
                return;
            }

            var instruction = new TextBox
            {
                Header = "요청 내용", AcceptsReturn = true, TextWrapping = TextWrapping.Wrap,
                MinHeight = 72, MaxHeight = 144, MaxLength = LocalAgentCli.MaxInstructionLength,
            };
            AutomationProperties.SetAutomationId(instruction, "LocalAgentInstruction");
            var rewrite = new CheckBox { Content = "선택한 텍스트를 바꿀 대체 제안 받기" };
            AutomationProperties.SetAutomationId(rewrite, "LocalAgentRewrite");
            var content = new StackPanel { Spacing = 12 };
            content.Children.Add(new TextBlock
            {
                Text = $"보내기를 선택하면 아래 텍스트와 요청이 설치된 CLI를 통해 {name} 서비스로 전송됩니다.",
                TextWrapping = TextWrapping.Wrap,
            });
            content.Children.Add(LocalAgentTextBox("선택한 텍스트", captured.PlainText, "LocalAgentSelection", 112));
            content.Children.Add(instruction);
            content.Children.Add(rewrite);
            var prompt = new ContentDialog
            {
                XamlRoot = Root.XamlRoot, Title = name, Content = content, PrimaryButtonText = "보내기",
                IsPrimaryButtonEnabled = false, CloseButtonText = "취소", DefaultButton = ContentDialogButton.Close,
            };
            AutomationProperties.SetAutomationId(prompt, "LocalAgentPrompt");
            instruction.TextChanged += (_, _) => prompt.IsPrimaryButtonEnabled = !string.IsNullOrWhiteSpace(instruction.Text);
            localAgentDialog = prompt;
            if (await prompt.ShowAsync() != ContentDialogResult.Primary || page?.Target != selected.Target ||
                !selected.Matches(editorInfo))
                return;

            replacement = await ShowLocalAgentResultAsync(provider, selected, captured.PlainText, instruction.Text.Trim(),
                rewrite.IsChecked == true);
        }
        catch (Exception error)
        {
            StatusText.Text = $"{name} 작업을 완료할 수 없습니다. {error.Message}";
            return;
        }
        finally
        {
            localAgentDialog = null;
            editorBusy = false;
            if (!closing)
            {
                RefreshButtons();
                RestorePageEditFocus();
            }
        }

        if (replacement is not null)
            await EnqueueDirectEdit(() => ApplyLocalAgentTextAsync(selected, replacement));
    }

    private async Task<string?> ShowLocalAgentResultAsync(LocalAgentProvider provider, LocalAgentSelection selected,
        string original, string instruction, bool rewrite)
    {
        using var request = new CancellationTokenSource();
        var progress = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 12 };
        progress.Children.Add(new ProgressRing { IsActive = true, Width = 32, Height = 32 });
        progress.Children.Add(new TextBlock { Text = "응답을 기다리는 중…", VerticalAlignment = VerticalAlignment.Center });
        var dialog = new ContentDialog
        {
            XamlRoot = Root.XamlRoot, Title = LocalAgentCli.DisplayName(provider),
            Content = progress, CloseButtonText = "취소", DefaultButton = ContentDialogButton.Close,
        };
        AutomationProperties.SetAutomationId(dialog, "LocalAgentResult");
        dialog.Closing += (_, _) => request.Cancel();
        localAgentDialog = dialog;
        var review = dialog.ShowAsync();
        string? response = null;
        try
        {
            response = await Task.Run(() => LocalAgentCli.RunAsync(provider, instruction, original, rewrite, request.Token),
                request.Token);
            if (!request.IsCancellationRequested)
            {
                var content = new StackPanel { Spacing = 12 };
                if (rewrite)
                    content.Children.Add(LocalAgentTextBox("선택한 텍스트", original, "LocalAgentOriginal", 112));
                content.Children.Add(LocalAgentTextBox(rewrite ? "대체 제안" : "응답", response, "LocalAgentResponse", 260));
                dialog.Content = content;
                dialog.CloseButtonText = "닫기";
                if (rewrite && selected.Matches(editorInfo) && page?.Target == selected.Target && original != response)
                {
                    string? restriction = ReplacementRestriction(selected.Range, response);
                    if (restriction is null)
                        dialog.PrimaryButtonText = "선택한 텍스트 바꾸기";
                    else
                        content.Children.Add(new TextBlock { Text = restriction, TextWrapping = TextWrapping.Wrap });
                }
            }
        }
        catch (Exception error)
        {
            if (!request.IsCancellationRequested)
            {
                dialog.Content = new TextBlock { Text = error.Message, TextWrapping = TextWrapping.Wrap };
                dialog.CloseButtonText = "닫기";
            }
        }
        return await review == ContentDialogResult.Primary ? response : null;
    }

    private static string? ReplacementRestriction(TextSelection selection, string text)
    {
        if (text.Contains('\t'))
            return "탭 문자가 포함된 제안은 적용할 수 없습니다.";
        if (selection.Anchor.CellPath.Length != 0 && text.AsSpan().IndexOfAny('\r', '\n') >= 0)
            return "표 셀에는 한 문단으로 된 제안만 적용할 수 있습니다.";
        return null;
    }

    private static TextBox LocalAgentTextBox(string header, string text, string automationId, double maxHeight)
    {
        var box = new TextBox
        {
            Header = header, Text = text, IsReadOnly = true, AcceptsReturn = true,
            TextWrapping = TextWrapping.Wrap, MaxHeight = maxHeight,
        };
        AutomationProperties.SetAutomationId(box, automationId);
        ScrollViewer.SetVerticalScrollBarVisibility(box, ScrollBarVisibility.Auto);
        return box;
    }

    private async Task ApplyLocalAgentTextAsync(LocalAgentSelection selected, string response)
    {
        CancellationToken token = CancellationToken.None;
        if (HasActiveComposition || page is not PageInfo current || current.Target != selected.Target ||
            !selected.Matches(editorInfo) || !selected.Matches(await worker.GetDocumentInfoAsync(selected.Target, token)))
        {
            StatusText.Text = "선택한 텍스트 또는 문서가 바뀌었습니다. 대체 제안을 적용하지 않았습니다.";
            return;
        }

        EditResult edit = await worker.PasteTextAsync(selected.Target, response, token);
        preferredX = null;
        await RefreshEditedPageAsync(current, edit);
        await UpdateDirectGeometryAsync(followCaret: true);
        RestorePageEditFocus();
    }
}
