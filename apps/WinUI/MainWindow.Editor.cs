using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Windows.System;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private NativeDocumentInfo? editorInfo;
    private bool editorBusy;
    private InlineDraft? inlineEditor;
    private bool InlineEditing => inlineEditor is not null;

    private async void OnPageTapped(object sender, TappedRoutedEventArgs args)
    {
        if (closing || editorBusy || picking || InlineEditing || page is not PageInfo current)
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

            await BeginInlineEditorAsync(current, hit.Position, caret);
        });
    }

    private async void OnPageEditKeyDown(object sender, KeyRoutedEventArgs args)
    {
        if (args.Key != VirtualKey.F2 || closing || editorBusy || picking || InlineEditing ||
            page is not PageInfo current)
            return;

        args.Handled = true;
        await RunEditorActionAsync(async () =>
        {
            CancellationToken token = CancellationToken.None;
            if (editorInfo is { Selection.Focus: TextPosition focus } info && info.Target == current.Target &&
                await worker.CaretAsync(current.Target, focus, token) is { } selectedCaret &&
                selectedCaret.Page == current.Page)
            {
                await BeginInlineEditorAsync(current, focus, selectedCaret);
                return;
            }

            // Hit-testing snaps to the nearest text line when the viewport inset
            // falls between lines. Confirm that the result belongs to this page.
            double x = Math.Clamp((PageScroll.HorizontalOffset + 4) / zoom, 0, Math.Max(0, current.Width - 1));
            double y = Math.Clamp((PageScroll.VerticalOffset + 4) / zoom, 0, Math.Max(0, current.Height - 1));
            TextHit? hit = await worker.HitTestAsync(current, x, y, token);
            if (hit?.Caret is not PageCaret caret || caret.Page != current.Page)
                throw new InvalidOperationException("이 쪽에서 편집할 문단을 찾을 수 없습니다.");
            await BeginInlineEditorAsync(current, hit.Position, caret);
        });
    }

    private async Task BeginInlineEditorAsync(PageInfo current, TextPosition position, PageCaret caret)
    {
        if (!IsInlinePageCurrent(current) || caret.Page != current.Page)
            return;

        CancellationToken token = CancellationToken.None;
        PageCaret? paragraphStart = await worker.CaretAsync(current.Target, position with { ScalarOffset = 0 }, token);
        InlineEditorPlacement placement = InlineEditorPlacement.Create(current, caret, paragraphStart, zoom,
            PageScroll.ViewportWidth, PageScroll.ViewportHeight)
            ?? throw new InvalidOperationException("편집할 문단의 위치를 찾을 수 없습니다.");

        TextSelection selection = new(position, position);
        if (editorInfo is { Editing: true } info && info.Target == current.Target)
        {
            EditResult result = await worker.SetSelectionAsync(current.Target, selection, token);
            editorInfo = info with { Selection = result.Outcome.Selection };
        }
        else
        {
            editorInfo = await worker.BeginEditAsync(current.Target, selection, token);
        }

        ImeEditContext context = await worker.ReadImeEditContextAsync(current.Target, token);
        if (!IsInlinePageCurrent(current) || context.Revision != current.Target.Revision ||
            context.FontGeneration != current.FontGeneration)
            return;
        if (context.Context.TextTruncated || context.Context.SelectionTruncated || context.CompositionId is not null)
            throw new InvalidOperationException("현재 문단의 편집 범위를 확보할 수 없습니다.");

        ShowInlineEditor(current, context, placement);
    }

    private void ShowInlineEditor(PageInfo current, ImeEditContext context, InlineEditorPlacement placement)
    {
        var text = new TextBox
        {
            Text = context.Context.Text,
            AcceptsReturn = false,
            TextWrapping = TextWrapping.Wrap,
            FontSize = placement.FontSize,
            MaxHeight = placement.MaxTextHeight,
        };
        AutomationProperties.SetName(text, "문단 텍스트");
        AutomationProperties.SetAutomationId(text, "InlineEditorText");
        AutomationProperties.SetHelpText(text, "Ctrl+Enter로 적용하고 Esc로 편집을 취소합니다.");
        var apply = new Button { Content = "적용" };
        var cancel = new Button { Content = "취소" };
        AutomationProperties.SetAutomationId(apply, "InlineEditorApply");
        AutomationProperties.SetAutomationId(cancel, "InlineEditorCancel");
        var actions = new StackPanel
        {
            Orientation = Orientation.Horizontal,
            HorizontalAlignment = HorizontalAlignment.Right,
            Spacing = 4,
            Children = { apply, cancel },
        };
        var overlay = new StackPanel
        {
            Width = placement.Width,
            Spacing = 4,
            Children = { text, actions },
        };
        var draft = new InlineDraft(current, context, overlay, text, apply, cancel);
        inlineEditor = draft;

        text.TextCompositionStarted += (_, _) =>
        {
            draft.IsComposing = true;
            apply.IsEnabled = false;
        };
        text.TextCompositionEnded += (_, _) =>
        {
            draft.IsComposing = false;
            apply.IsEnabled = !editorBusy;
        };
        apply.Click += async (_, _) => await CommitInlineEditorAsync(draft);
        cancel.Click += (_, _) => CancelInlineEditor(draft);

        var applyShortcut = new KeyboardAccelerator
        {
            Key = VirtualKey.Enter,
            Modifiers = VirtualKeyModifiers.Control,
            ScopeOwner = overlay,
        };
        applyShortcut.Invoked += async (_, args) =>
        {
            if (draft.IsComposing)
                return;
            args.Handled = true;
            await CommitInlineEditorAsync(draft);
        };
        var cancelShortcut = new KeyboardAccelerator { Key = VirtualKey.Escape, ScopeOwner = overlay };
        cancelShortcut.Invoked += (_, args) =>
        {
            // Let the IME handle the first Escape during active composition.
            if (draft.IsComposing)
                return;
            args.Handled = true;
            CancelInlineEditor(draft);
        };
        overlay.KeyboardAccelerators.Add(applyShortcut);
        overlay.KeyboardAccelerators.Add(cancelShortcut);

        int cursor = PageEditText.Utf16Offset(text.Text, context.Context.CursorScalar);
        text.Loaded += (_, _) =>
        {
            if (inlineEditor != draft)
                return;
            text.Focus(FocusState.Programmatic);
            text.Select(cursor, 0);
            overlay.StartBringIntoView(new BringIntoViewOptions { AnimationDesired = false });
        };
        Canvas.SetLeft(overlay, placement.Left);
        Canvas.SetTop(overlay, placement.Top);
        Canvas.SetZIndex(overlay, 1);
        overlay.SizeChanged += (_, args) => Canvas.SetTop(overlay,
            placement.FitTop(current.Height * zoom, args.NewSize.Height));
        PageCanvas.Children.Add(overlay);
    }

    private async Task CommitInlineEditorAsync(InlineDraft draft)
    {
        if (inlineEditor != draft || draft.IsComposing)
            return;

        await RunEditorActionAsync(async () =>
        {
            if (!IsInlinePageCurrent(draft.Page))
                throw new InvalidOperationException("문서가 변경되어 이 문단에 적용할 수 없습니다.");
            string current = draft.Text.Text;
            Utf16Selection selection = PageEditText.Selection(current, draft.Text.SelectionStart,
                draft.Text.SelectionLength) ?? throw new InvalidOperationException("텍스트 선택 범위가 올바르지 않습니다.");
            ImeReplacement replacement = PageEditText.Difference(draft.Context.Context.Text, current)
                ?? new ImeReplacement(new Utf16Range(0, 0), string.Empty);

            draft.Text.IsReadOnly = true;
            draft.Apply.IsEnabled = false;
            draft.Cancel.IsEnabled = false;
            try
            {
                EditResult edited = await worker.CommitImeAsync(draft.Page.Target, draft.Context, replacement,
                    selection, CancellationToken.None);
                DismissInlineEditor();
                if (edited.Outcome.DocumentChanged)
                    await RefreshEditedPageAsync(draft.Page, edited);
                else if (editorInfo is NativeDocumentInfo info)
                    editorInfo = info with { Selection = edited.Outcome.Selection };
                RestorePageEditFocus();
            }
            finally
            {
                if (inlineEditor == draft)
                {
                    draft.Text.IsReadOnly = false;
                    draft.Apply.IsEnabled = !draft.IsComposing;
                    draft.Cancel.IsEnabled = true;
                    draft.Text.Focus(FocusState.Programmatic);
                }
            }
        });
    }

    private void CancelInlineEditor(InlineDraft draft)
    {
        if (inlineEditor != draft || editorBusy)
            return;

        DismissInlineEditor();
        RefreshButtons();
        RestorePageEditFocus();
    }

    private void DismissInlineEditor()
    {
        if (inlineEditor is not InlineDraft draft)
            return;
        inlineEditor = null;
        PageCanvas.Children.Remove(draft.Overlay);
    }

    private void RestorePageEditFocus()
    {
        if (closing || page is null || !PageCanvas.IsLoaded)
            return;
        if (!PageCanvas.Focus(FocusState.Programmatic) && PageScroll.IsLoaded)
            PageScroll.Focus(FocusState.Programmatic);
    }

    private bool IsInlinePageCurrent(PageInfo current) =>
        page is PageInfo active && active.Target == current.Target && active.Page == current.Page &&
        active.FontGeneration == current.FontGeneration;

    private sealed class InlineDraft(PageInfo page, ImeEditContext context, StackPanel overlay,
        TextBox text, Button apply, Button cancel)
    {
        public PageInfo Page { get; } = page;
        public ImeEditContext Context { get; } = context;
        public StackPanel Overlay { get; } = overlay;
        public TextBox Text { get; } = text;
        public Button Apply { get; } = apply;
        public Button Cancel { get; } = cancel;
        public bool IsComposing { get; set; }
    }

    private async void OnUndoClicked(object sender, RoutedEventArgs args)
    {
        if (!InlineEditing && page is PageInfo current && editorInfo?.CanUndo == true)
            await RunEditorActionAsync(async () => await RefreshEditedPageAsync(current,
                await worker.UndoAsync(current.Target, CancellationToken.None)));
    }

    private async void OnRedoClicked(object sender, RoutedEventArgs args)
    {
        if (!InlineEditing && page is PageInfo current && editorInfo?.CanRedo == true)
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
                Selection = edit.Outcome.Selection,
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
