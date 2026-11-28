using Microsoft.UI.Input;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Shapes;
using Windows.Foundation;
using Windows.System;
using Windows.UI.Core;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private readonly Queue<Func<Task>> directActions = new();
    private Task pendingDirectEdits = Task.CompletedTask;
    private readonly Microsoft.UI.Dispatching.DispatcherQueueTimer recoveryTimer;
    private ulong? pendingRecoveryDocument;
    private bool directEditing;
    private bool directQueueRunning;
    private bool changingInput;
    private bool platformComposing;
    private bool nativePreviewAvailable;
    private string previewText = "";
    private long previewVersion;
    private uint? dragPointer;
    private TextPosition? dragAnchor;
    private long pointerGesture;
    private long dragVersion;
    private double? preferredX;
    private (DocumentTarget Target, ulong FontGeneration, PageCaret? Caret, SelectionRectangle[] Selection)? directGeometry;

    private bool HasActiveComposition => platformComposing || editorInfo?.CompositionId is not null;

    private void OnPagePointerPressed(object sender, PointerRoutedEventArgs args)
    {
        if (args.Pointer.PointerDeviceType != PointerDeviceType.Mouse ||
            !args.GetCurrentPoint(PageCanvas).Properties.IsLeftButtonPressed ||
            !AcceptPageInput(args.GetPosition(PageCanvas), out PageInfo current, out Point location))
            return;

        args.Handled = true;
        dragPointer = PageImage.CapturePointer(args.Pointer) ? args.Pointer.PointerId : null;
        dragAnchor = null;
        QueuePointerSelection(current, location, extend: false);
    }

    private void OnPagePointerMoved(object sender, PointerRoutedEventArgs args)
    {
        if (dragPointer != args.Pointer.PointerId ||
            !AcceptPageInput(args.GetPosition(PageCanvas), out PageInfo current, out Point location))
            return;

        args.Handled = true;
        QueuePointerSelection(current, location, extend: true);
    }

    private void OnPagePointerReleased(object sender, PointerRoutedEventArgs args)
    {
        if (dragPointer != args.Pointer.PointerId)
            return;

        args.Handled = true;
        if (AcceptPageInput(args.GetPosition(PageCanvas), out PageInfo current, out Point location))
            QueuePointerSelection(current, location, extend: true);
        dragPointer = null;
        PageImage.ReleasePointerCapture(args.Pointer);
    }

    private void OnPagePointerCaptureLost(object sender, PointerRoutedEventArgs args)
    {
        if (dragPointer == args.Pointer.PointerId)
            dragPointer = null;
    }

    private void OnPageTapped(object sender, TappedRoutedEventArgs args)
    {
        if (args.PointerDeviceType == PointerDeviceType.Mouse ||
            !AcceptPageInput(args.GetPosition(PageCanvas), out PageInfo current, out Point location))
            return;

        args.Handled = true;
        QueuePointerSelection(current, location, extend: false);
    }

    private void OnPageEditKeyDown(object sender, KeyRoutedEventArgs args)
    {
        if (args.Key != VirtualKey.F2 || !DocumentActionsAvailable || page is not PageInfo current)
            return;

        args.Handled = true;
        EnqueueDirectEdit(async () =>
        {
            if (editorInfo is { Editing: true, Selection: not null } info && info.Target == current.Target)
            {
                directEditing = true;
                await UpdateDirectGeometryAsync(followCaret: true, focus: true);
                return;
            }

            var location = new Point(
                Math.Clamp((PageScroll.HorizontalOffset + 4) / zoom, 0, Math.Max(0, current.Width - 1)),
                Math.Clamp((PageScroll.VerticalOffset + 4) / zoom, 0, Math.Max(0, current.Height - 1)));
            await MovePointerSelectionAsync(current, location, extend: false);
        });
    }

    private bool AcceptPageInput(Point point, out PageInfo current, out Point location)
    {
        current = page!;
        location = new Point(point.X / zoom, point.Y / zoom);
        return current is not null && !HasActiveComposition && CanQueueDirectEdit &&
            location.X >= 0 && location.Y >= 0 && location.X < current.Width && location.Y < current.Height;
    }

    private void QueuePointerSelection(PageInfo current, Point location, bool extend)
    {
        long gesture = extend ? pointerGesture : ++pointerGesture;
        long version = ++dragVersion;
        EnqueueDirectEdit(() =>
            gesture == pointerGesture && (!extend || version == dragVersion)
                ? MovePointerSelectionAsync(current, location, extend)
                : Task.CompletedTask);
    }

    private async Task MovePointerSelectionAsync(PageInfo original, Point location, bool extend)
    {
        if (page is not PageInfo current || current.Page != original.Page ||
            current.Target.DocumentId != original.Target.DocumentId)
            return;

        CancellationToken token = CancellationToken.None;
        TextHit? hit = await worker.HitTestAsync(current, location.X, location.Y, token);
        if (hit?.Caret is not PageCaret caret || caret.Page != current.Page)
            return;

        if (!extend)
            dragAnchor = hit.Position;
        if (dragAnchor is not TextPosition anchor || anchor.Section != hit.Position.Section ||
            ((anchor.CellPath.Length != 0 || hit.Position.CellPath.Length != 0) &&
             !SharesTextLeaf(anchor, hit.Position)))
            return;

        var selection = new TextSelection(anchor, hit.Position);
        if (editorInfo is { Editing: true } active && active.Target == current.Target)
        {
            if (active.Selection is not { } old ||
                !SamePosition(old.Anchor, selection.Anchor) || !SamePosition(old.Focus, selection.Focus))
            {
                EditResult selected = await worker.SetSelectionAsync(current.Target, selection, token);
                editorInfo = active with { Selection = selected.Outcome.Selection };
            }
        }
        else
        {
            editorInfo = await worker.BeginEditAsync(current.Target, selection, token);
        }

        preferredX = null;
        directEditing = true;
        await UpdateDirectGeometryAsync(followCaret: false, focus: !extend);
    }

    private static bool SharesTextLeaf(TextPosition left, TextPosition right) =>
        left.Section == right.Section && left.Paragraph == right.Paragraph &&
        left.CellPath.SequenceEqual(right.CellPath);

    private static bool SamePosition(TextPosition left, TextPosition right) =>
        SharesTextLeaf(left, right) && left.ScalarOffset == right.ScalarOffset;

    private bool CanQueueDirectEdit => !closing && !picking && documentRequest is null &&
        (!editorBusy || directQueueRunning);

    private Task EnqueueDirectEdit(Func<Task> action)
    {
        if (!CanQueueDirectEdit)
            return Task.CompletedTask;

        directActions.Enqueue(action);
        if (!directQueueRunning)
            pendingDirectEdits = DrainDirectEditsAsync();
        return pendingDirectEdits;
    }

    private async Task SettleDirectInputAsync()
    {
        if (platformComposing)
            PageCanvas.Focus(FocusState.Programmatic);
        await pendingDirectEdits;
    }

    private async Task DrainDirectEditsAsync()
    {
        directQueueRunning = true;
        editorBusy = true;
        RefreshButtons();
        try
        {
            while (!closing && directActions.TryDequeue(out Func<Task>? action))
                await action();
        }
        catch (Exception error)
        {
            directActions.Clear();
            try
            {
                await CancelNativePreviewAsync();
            }
            catch (Exception cleanup)
            {
                System.Diagnostics.Debug.WriteLine(cleanup);
            }
            DismissDirectEditor();
            StatusText.Text = "문서를 편집할 수 없습니다. " + error.Message;
        }
        finally
        {
            directQueueRunning = false;
            editorBusy = false;
            RefreshButtons();
        }
    }

    private void OnDirectTextChanged(object sender, TextChangedEventArgs args)
    {
        if (changingInput || platformComposing || !directEditing || DirectInput.Text.Length == 0)
            return;

        string text = DirectInput.Text;
        EmptyInputControl();
        EnqueueDirectEdit(() => InsertDirectTextAsync(text));
    }

    private void EmptyInputControl()
    {
        if (DirectInput.Text.Length == 0)
            return;
        changingInput = true;
        try
        {
            DirectInput.Text = "";
        }
        finally
        {
            changingInput = false;
        }
    }

    private async Task InsertDirectTextAsync(string text)
    {
        if (!directEditing || page is not PageInfo current || editorInfo?.Editing != true)
            return;

        EditResult edit = await worker.ReplaceAsync(current.Target, text, CancellationToken.None);
        preferredX = null;
        await RefreshEditedPageAsync(current, edit);
        await UpdateDirectGeometryAsync(followCaret: true);
    }

    private void OnDirectCompositionStarted(TextBox sender, TextCompositionStartedEventArgs args)
    {
        if (!directEditing || changingInput || !CanQueueDirectEdit)
            return;
        platformComposing = true;
        previewVersion++;
        EnqueueDirectEdit(BeginNativePreviewAsync);
    }

    private void OnDirectCompositionChanged(TextBox sender, TextCompositionChangedEventArgs args)
    {
        if (!directEditing || !platformComposing)
            return;

        long version = ++previewVersion;
        string text = sender.Text;
        EnqueueDirectEdit(() => version == previewVersion ? SetNativePreviewAsync(text, updateView: true) :
            Task.CompletedTask);
    }

    private void OnDirectCompositionEnded(TextBox sender, TextCompositionEndedEventArgs args)
    {
        if (!platformComposing)
            return;
        string text = sender.Text;
        platformComposing = false;
        previewVersion++;
        EmptyInputControl();
        EnqueueDirectEdit(() => FinishNativePreviewAsync(text));
    }

    private async Task BeginNativePreviewAsync()
    {
        previewText = "";
        nativePreviewAvailable = false;
        if (page is not PageInfo current ||
            editorInfo?.Selection is not { } selection || !SharesTextLeaf(selection.Anchor, selection.Focus))
            return;

        EditResult started = await worker.BeginCompositionAsync(current.Target, CancellationToken.None);
        await RefreshEditedPageAsync(current, started);
        nativePreviewAvailable = true;
    }

    private async Task SetNativePreviewAsync(string text, bool updateView)
    {
        if (!nativePreviewAvailable || previewText == text ||
            page is not PageInfo current || editorInfo?.CompositionId is null)
            return;

        CancellationToken token = CancellationToken.None;
        ImeEditContext context = await worker.ReadImeEditContextAsync(current.Target, token);
        Utf16Range range = context.Preedit ?? throw new InvalidOperationException("입력기 조합 범위가 없습니다.");
        EditResult preview = await worker.ContinueCompositionAsync(current.Target, context,
            new ImeReplacement(range, text), null, token);
        previewText = text;
        await RefreshEditedPageAsync(current, preview);
        if (updateView)
            await UpdateDirectGeometryAsync(followCaret: true);
    }

    private async Task FinishNativePreviewAsync(string text)
    {
        if (!nativePreviewAvailable)
        {
            if (text.Length != 0)
                await InsertDirectTextAsync(text);
            return;
        }

        if (text.Length == 0)
        {
            await CancelNativePreviewAsync();
        }
        else
        {
            await SetNativePreviewAsync(text, updateView: false);
            if (page is PageInfo current && editorInfo?.CompositionId is ulong composition)
            {
                EditResult edit = await worker.CommitCompositionAsync(current.Target, composition, CancellationToken.None);
                await RefreshEditedPageAsync(current, edit);
            }
        }

        nativePreviewAvailable = false;
        previewText = "";
        preferredX = null;
        await UpdateDirectGeometryAsync(followCaret: true);
    }

    private async Task CancelNativePreviewAsync()
    {
        if (page is PageInfo current && editorInfo?.CompositionId is ulong composition)
        {
            EditResult cancelled = await worker.CancelCompositionAsync(current.Target, composition, CancellationToken.None);
            await RefreshEditedPageAsync(current, cancelled);
        }
        nativePreviewAvailable = false;
        previewText = "";
    }

    private void OnDirectPreviewKeyDown(object sender, KeyRoutedEventArgs args)
    {
        if (!directEditing || platformComposing)
            return;

        bool control = KeyDown(VirtualKey.Control);
        bool shift = KeyDown(VirtualKey.Shift);
        if (control && args.Key is VirtualKey.Z or VirtualKey.Y)
        {
            args.Handled = true;
            bool redo = args.Key == VirtualKey.Y || shift;
            EnqueueDirectEdit(() => EditHistoryAsync(redo));
            return;
        }
        if (control && args.Key is VirtualKey.C or VirtualKey.X or VirtualKey.V)
        {
            args.Handled = true;
            EnqueueClipboardAction(args.Key);
            return;
        }

        NativeEditKey? key = control
            ? args.Key == VirtualKey.A ? NativeEditKey.SelectAll : null
            : args.Key switch
            {
                VirtualKey.Left => NativeEditKey.MoveLeft,
                VirtualKey.Right => NativeEditKey.MoveRight,
                VirtualKey.Up => NativeEditKey.MoveUp,
                VirtualKey.Down => NativeEditKey.MoveDown,
                VirtualKey.Home => NativeEditKey.Home,
                VirtualKey.End => NativeEditKey.End,
                VirtualKey.Back => NativeEditKey.DeleteBackward,
                VirtualKey.Delete => NativeEditKey.DeleteForward,
                VirtualKey.Enter => NativeEditKey.SplitParagraph,
                _ => null,
            };
        if (key is NativeEditKey command)
        {
            args.Handled = true;
            EnqueueDirectEdit(() => ApplyDirectKeyAsync(command, shift));
        }
    }

    private static bool KeyDown(VirtualKey key) =>
        (InputKeyboardSource.GetKeyStateForCurrentThread(key) & CoreVirtualKeyStates.Down) != 0;

    private async Task ApplyDirectKeyAsync(NativeEditKey key, bool extendSelection)
    {
        if (page is not PageInfo current || editorInfo?.Selection is not { } selection)
            return;

        if (selection.Focus.CellPath.Length != 0 && key != NativeEditKey.SplitParagraph)
        {
            if (key is NativeEditKey.DeleteBackward or NativeEditKey.DeleteForward &&
                !SamePosition(selection.Anchor, selection.Focus))
            {
                EditResult removed = await worker.ReplaceAsync(current.Target, "", CancellationToken.None);
                await RefreshEditedPageAsync(current, removed);
                await UpdateDirectGeometryAsync(followCaret: true);
                return;
            }
            StatusText.Text = "표 셀의 방향키와 삭제 명령은 아직 지원 범위가 제한됩니다.";
            return;
        }

        CancellationToken token = CancellationToken.None;
        NativeCommandResult result = await worker.ExecuteEditCommandAsync(current.Target, key, token, extendSelection,
            preferredX);
        preferredX = result.PreferredX;
        await RefreshEditedPageAsync(current, result.Edit);
        await UpdateDirectGeometryAsync(followCaret: true);
    }

    private async Task UpdateDirectGeometryAsync(bool followCaret, bool focus = false)
    {
        if (!directEditing || page is not PageInfo current || editorInfo?.Selection is not { } selection)
            return;

        CancellationToken token = CancellationToken.None;
        PageCaret? caret = await worker.CaretAsync(current.Target, selection.Focus, token);
        if (caret is not null && caret.Page != current.Page && followCaret)
        {
            current = await worker.GetPageAsync(current, caret.Page, token);
            PageImage.Source = null;
            page = current;
            SetPageSize(current);
            QueueRender();
        }

        SelectionRectangle[] rectangles = SamePosition(selection.Anchor, selection.Focus) ? [] :
            await worker.GetSelectionRectanglesAsync(current, selection, token);
        directGeometry = (current.Target, current.FontGeneration, caret, rectangles);
        DrawDirectGeometry();
        if (caret?.Page == current.Page)
        {
            if (followCaret)
                BringCaretIntoView(caret);
            if (focus)
            {
                PageCanvas.UpdateLayout();
                DirectInput.Focus(FocusState.Programmatic);
            }
        }
    }

    private void DrawDirectGeometry()
    {
        DirectEditOverlay.Children.Clear();
        if (page is not PageInfo current || directGeometry is not { } geometry ||
            geometry.Target != current.Target || geometry.FontGeneration != current.FontGeneration)
            return;

        foreach (SelectionRectangle rectangle in geometry.Selection.Where(rectangle => rectangle.Page == current.Page))
            if (rectangle.Bounds.Width > 0 && rectangle.Bounds.Height > 0)
                DrawMark(rectangle.Bounds.X, rectangle.Bounds.Y, rectangle.Bounds.Width * zoom,
                    rectangle.Bounds.Height * zoom, (Brush)DirectEditOverlay.Resources["DirectSelectionBrush"], 0.32);

        if (geometry.Caret is not { } caret || caret.Page != current.Page || caret.Height <= 0)
            return;

        if (geometry.Selection.Length == 0)
            DrawMark(caret.X, caret.Y, 1.5, caret.Height * zoom,
                (Brush)DirectEditOverlay.Resources["DirectCaretBrush"], 1);

        // Keep an actual, measured WinUI text input at the Skia caret for the
        // input service's screen coordinates. The document renders the visual text.
        DirectInput.Visibility = Visibility.Visible;
        DirectInput.Height = Math.Max(18, caret.Height * zoom);
        DirectInput.FontSize = Math.Clamp(caret.Height * zoom * 0.8, 9, 48);
        Canvas.SetLeft(DirectInput, caret.X * zoom);
        Canvas.SetTop(DirectInput, caret.Y * zoom);
    }

    private void DrawMark(double x, double y, double width, double height, Brush brush, double opacity)
    {
        if (!double.IsFinite(x) || !double.IsFinite(y) || !double.IsFinite(width) || !double.IsFinite(height))
            return;

        var mark = new Rectangle { Width = width, Height = height, Fill = brush, Opacity = opacity };
        Canvas.SetLeft(mark, x * zoom);
        Canvas.SetTop(mark, y * zoom);
        DirectEditOverlay.Children.Add(mark);
    }

    private void BringCaretIntoView(PageCaret caret)
    {
        var x = caret.X * zoom;
        var y = caret.Y * zoom;
        if (x < PageScroll.HorizontalOffset || x + 2 > PageScroll.HorizontalOffset + PageScroll.ViewportWidth ||
            y < PageScroll.VerticalOffset || y + caret.Height * zoom > PageScroll.VerticalOffset + PageScroll.ViewportHeight)
        {
            PageScroll.ChangeView(Math.Max(0, x - 16), Math.Max(0, y - 24), null, true);
        }
    }

    private void ClearDirectGeometry()
    {
        directGeometry = null;
        DirectEditOverlay.Children.Clear();
    }

    private void DismissDirectEditor()
    {
        directEditing = false;
        platformComposing = false;
        dragPointer = null;
        dragAnchor = null;
        pointerGesture++;
        preferredX = null;
        previewVersion++;
        ClearDirectGeometry();
        DirectInput.Visibility = Visibility.Collapsed;
        EmptyInputControl();
    }

    private void RestorePageEditFocus()
    {
        if (directEditing && DirectInput.Visibility == Visibility.Visible &&
            DirectInput.Focus(FocusState.Programmatic))
            return;
        if (!closing && page is not null && PageCanvas.IsLoaded && !PageCanvas.Focus(FocusState.Programmatic))
            PageScroll.Focus(FocusState.Programmatic);
    }
}
