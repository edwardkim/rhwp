using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private NativeDocumentInfo? editorInfo;
    private bool editorBusy;

    private async void OnPageTapped(object sender, TappedRoutedEventArgs args)
    {
        if (closing || editorBusy || picking || page is not PageInfo current)
            return;

        var point = args.GetPosition(PageCanvas);
        double x = point.X / zoom;
        double y = point.Y / zoom;
        if (x < 0 || y < 0 || x >= current.Width || y >= current.Height)
            return;

        args.Handled = true;
        await RunEditorActionAsync(async () =>
        {
            CancellationToken token = CancellationToken.None;
            TextHit? hit = await worker.HitTestAsync(current, x, y, token);
            if (hit?.Caret is not PageCaret caret || caret.Page != current.Page)
                return;

            TextSelection selection = new(hit.Position, hit.Position);
            if (editorInfo?.Editing == true)
                await worker.SetSelectionAsync(current.Target, selection, token);
            else
                editorInfo = await worker.BeginEditAsync(current.Target, selection, token);

            ImeEditContext context = await worker.ReadImeEditContextAsync(current.Target, token);
            if (context.Context.TextTruncated || context.Context.SelectionTruncated)
                throw new InvalidOperationException("한 문단 전체를 가져올 수 없어 이 문단은 편집할 수 없습니다.");

            var text = new TextBox
            {
                Text = context.Context.Text,
                AcceptsReturn = false,
                Header = "문단 텍스트",
                MinWidth = 320,
                TextWrapping = TextWrapping.Wrap,
            };
            int caretOffset = PageEditText.Utf16Offset(text.Text, context.Context.CursorScalar);
            text.Loaded += (_, _) =>
            {
                text.Focus(FocusState.Programmatic);
                text.Select(caretOffset, 0);
            };
            var dialog = new ContentDialog
            {
                XamlRoot = Root.XamlRoot,
                Title = "문단 편집",
                Content = text,
                PrimaryButtonText = "적용",
                CloseButtonText = "취소",
            };
            if (await dialog.ShowAsync() != ContentDialogResult.Primary)
                return;

            IReadOnlyList<ImeReplacement> replacements = PageEditText.Differences(context.Context.Text, text.Text);
            if (replacements.Count == 0)
                return;

            var selected = PageEditText.Selection(text.Text, text.SelectionStart, text.SelectionLength);
            EditResult edited = await worker.CommitParagraphAsync(current.Target, context, replacements, selected, token);
            await RefreshEditedPageAsync(current, edited);
        });
    }

    private async void OnUndoClicked(object sender, RoutedEventArgs args)
    {
        if (page is PageInfo current && editorInfo?.CanUndo == true)
            await RunEditorActionAsync(async () => await RefreshEditedPageAsync(current,
                await worker.UndoAsync(current.Target, CancellationToken.None)));
    }

    private async void OnRedoClicked(object sender, RoutedEventArgs args)
    {
        if (page is PageInfo current && editorInfo?.CanRedo == true)
            await RunEditorActionAsync(async () => await RefreshEditedPageAsync(current,
                await worker.RedoAsync(current.Target, CancellationToken.None)));
    }

    private async Task RunEditorActionAsync(Func<Task> action)
    {
        if (closing || editorBusy || picking)
            return;

        editorBusy = true;
        RefreshButtons();
        try
        {
            await action();
        }
        catch (Exception error)
        {
            StatusText.Text = "문단을 편집할 수 없습니다. " + error.Message;
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
        }
    }

    private async Task RefreshEditedPageAsync(PageInfo previous, EditResult edit)
    {
        renderRequest?.Cancel();
        renderTimer.Stop();
        PageImage.Source = null;
        page = null;
        if (editorInfo is NativeDocumentInfo prior)
        {
            editorInfo = prior with
            {
                Revision = edit.Target.Revision,
                IsDirty = prior.IsDirty || edit.Outcome.DocumentChanged,
            };
            UpdateTitle();
        }

        CancellationToken token = CancellationToken.None;
        NativeDocumentInfo info = await worker.GetDocumentInfoAsync(edit.Target, token);
        editorInfo = info;
        documentError = null;
        UpdateTitle();
        if (info.PageCount == 0)
            return;

        uint index = Math.Min(previous.Page, info.PageCount - 1);
        PageInfo updated = await worker.GetPageAsync(previous with
        {
            Target = info.Target,
            PageCount = info.PageCount,
            FontGeneration = info.FontGeneration,
        }, index, token);
        page = updated;
        SetPageSize(updated);
        QueueRender();
    }

    private void UpdateTitle()
    {
        Title = fileName is null ? "RHWP" : fileName + (editorInfo?.IsDirty == true ? " * — RHWP" : " — RHWP");
    }
}
