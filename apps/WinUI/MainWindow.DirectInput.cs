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
    private const int MaxDeferredCaretUpdates = 4;
    private int directCaretUpdatesPending;
    private bool changingInput;
    private string capturedInputText = "";
    private string compositionInputPrefix = "";
    private bool platformComposing;
    private TaskCompletionSource? platformCompositionFinished;
    private bool nativePreviewAvailable;
    private string previewText = "";
    private long previewVersion;
    private uint? dragPointer;
    private TextPosition? dragAnchor;
    private TextSelection? dragTextUnit;
    private long pointerGesture;
    private long dragVersion;
    private double? preferredX;
    private (DocumentTarget Target, ulong FontGeneration, PageCaret? Caret, SelectionRectangle[] Selection)? directGeometry;

    private bool HasActiveComposition => platformComposing || editorInfo?.CompositionId is not null;

    private void OnPagePointerPressed(object sender, PointerRoutedEventArgs args)
    {
        if (args.Pointer.PointerDeviceType != PointerDeviceType.Mouse)
            return;
        PointerPoint point = args.GetCurrentPoint(PageCanvas);
        if (!point.Properties.IsLeftButtonPressed)
            return;

        ClearTableObjectSelection();
        ClearPictureObjectSelection();
        if (!AcceptPageInput(point.Position, out PageInfo current, out Point location))
        {
            lastTextClick = null;
            return;
        }

        args.Handled = true;
        NativeTextUnit? unit = BeginTextClick(current, args.GetCurrentPoint(Root).Position, point.Timestamp);
        dragPointer = PageCanvas.CapturePointer(args.Pointer) ? args.Pointer.PointerId : null;
        QueuePointerSelection(current, location, extend: false, unit: unit);
    }

    private void OnPagePointerMoved(object sender, PointerRoutedEventArgs args)
    {
        if (dragPointer != args.Pointer.PointerId || !ContinueTextDrag(args.GetCurrentPoint(Root).Position) ||
            !AcceptPageInput(args.GetCurrentPoint(PageCanvas).Position, out PageInfo current, out Point location,
                extend: true))
            return;

        args.Handled = true;
        QueuePointerSelection(current, location, extend: true);
    }

    private void OnPagePointerReleased(object sender, PointerRoutedEventArgs args)
    {
        if (dragPointer != args.Pointer.PointerId)
            return;

        args.Handled = true;
        if (ContinueTextDrag(args.GetCurrentPoint(Root).Position) &&
            AcceptPageInput(args.GetCurrentPoint(PageCanvas).Position, out PageInfo current, out Point location,
                extend: true))
            QueuePointerSelection(current, location, extend: true, final: true);
        dragPointer = null;
        PageCanvas.ReleasePointerCapture(args.Pointer);
        RefreshFormattingActions();
    }

    private void OnPagePointerCaptureLost(object sender, PointerRoutedEventArgs args)
    {
        if (dragPointer == args.Pointer.PointerId)
        {
            dragPointer = null;
            lastTextClick = null;
            RefreshFormattingActions();
        }
    }

    private void OnPageTapped(object sender, TappedRoutedEventArgs args)
    {
        if (args.PointerDeviceType == PointerDeviceType.Mouse ||
            !AcceptPageInput(args.GetPosition(PageCanvas), out PageInfo current, out Point location))
            return;

        args.Handled = true;
        lastTextClick = null;
        QueuePointerSelection(current, location, extend: false);
    }

    private void OnPageEditKeyDown(object sender, KeyRoutedEventArgs args)
    {
        if (KeyDown(VirtualKey.Control) && !KeyDown(VirtualKey.Shift) && !KeyDown(VirtualKey.Menu) &&
            TryZoomFromKey(args.Key))
        {
            args.Handled = true;
            return;
        }

        bool exitObject = args.Key == VirtualKey.Escape && (selectedTable is not null || selectedPicture is not null);
        if ((!exitObject && (args.Key != VirtualKey.F2 || KeyDown(VirtualKey.Control))) ||
            !DocumentActionsAvailable || page is not PageInfo current)
            return;

        args.Handled = true;
        ClearTableObjectSelection();
        ClearPictureObjectSelection();
        EnqueueDirectEdit(async () =>
        {
            if (editorInfo is { Editing: true, Selection: not null } info && info.Target == current.Target)
            {
                directEditing = true;
                await UpdateDirectGeometryAsync(followCaret: true, focus: true);
                return;
            }

            if (continuousPages is not { } layout || !layout.TryGet(current.Page, out PagePlacement placement))
                return;

            var location = new Point(
                Math.Clamp((PageScroll.HorizontalOffset + 4) / zoom - placement.Left, 0, Math.Max(0, placement.Width - 1)),
                Math.Clamp((PageScroll.VerticalOffset + 4) / zoom - placement.Top, 0, Math.Max(0, placement.Height - 1)));
            await MovePointerSelectionAsync(current, location, extend: false);
        });
    }

    private bool AcceptPageInput(Point point, out PageInfo current, out Point location, bool extend = false)
    {
        current = null!;
        location = default;
        if (page is not { PageCount: > 0 } active || PageScroll.ZoomFactor != 1 || HasActiveComposition || !CanQueueDirectEdit ||
            continuousPages is not { } layout || !layout.Matches(active))
            return false;

        double x = point.X / zoom;
        double y = point.Y / zoom;
        if (!layout.TryHit(x, y, out PagePlacement placement))
        {
            if (!extend)
                return false;
            placement = layout.NearestPage(y);
        }

        PageInfo? measured = layout.Measured(active, placement.Index);
        if (measured is null && !extend)
            return false;

        current = measured ?? layout.Describe(active, placement);
        location = new Point(x - placement.Left, y - placement.Top);
        return true;
    }

    private void QueuePointerSelection(PageInfo current, Point location, bool extend, bool final = false,
        NativeTextUnit? unit = null)
    {
        long gesture = extend ? pointerGesture : ++pointerGesture;
        long version = ++dragVersion;
        EnqueueDirectEdit(() =>
            !extend || final || (gesture == pointerGesture && version == dragVersion)
                ? MovePointerSelectionAsync(current, location, extend, unit)
                : Task.CompletedTask);
    }

    private async Task<bool> MovePointerSelectionAsync(PageInfo original, Point location, bool extend,
        NativeTextUnit? unit = null)
    {
        if (!extend)
        {
            dragAnchor = null;
            dragTextUnit = null;
        }
        if (page is not PageInfo activePage || activePage.Target.DocumentId != original.Target.DocumentId ||
            continuousPages is not { } layout || !layout.TryGet(original.Page, out PagePlacement placement))
            return false;

        CancellationToken token = CancellationToken.None;
        PageInfo current = layout.Measured(activePage, placement.Index) ??
            await worker.GetPageAsync(activePage, placement.Index, token);
        if (extend)
        {
            // A captured drag can pass through the canvas margin or between sheets.
            location = new Point(Math.Clamp(location.X, 0, Math.Max(0, current.Width - 1)),
                Math.Clamp(location.Y, 0, Math.Max(0, current.Height - 1)));
        }
        else if (location.X >= current.Width || location.Y >= current.Height)
        {
            return false;
        }

        TextHit? hit = await worker.HitTestAsync(current, location.X, location.Y, token);
        if (hit?.Caret is not PageCaret caret || caret.Page != current.Page)
        {
            if (!extend && await FindPictureContextAsync(current, location) is { } picture)
                SelectPictureObject(picture);
            return false;
        }

        if (!extend)
            dragAnchor = hit.Position;
        if (dragAnchor is not TextPosition anchor || anchor.Section != hit.Position.Section)
            return false;
        if ((anchor.CellPath.Length != 0 || hit.Position.CellPath.Length != 0) &&
            !SharesTextLeaf(anchor, hit.Position) &&
            (!SharesTableCell(anchor, hit.Position) || !await worker.SupportsCellParagraphSelectionAsync(token)))
            return false;

        var selection = new TextSelection(anchor, hit.Position);
        if (extend && dragTextUnit is { } selectedUnit)
        {
            selection = CompareInTextContainer(hit.Position, selectedUnit.Anchor) < 0
                ? new TextSelection(selectedUnit.Focus, hit.Position)
                : CompareInTextContainer(hit.Position, selectedUnit.Focus) > 0 ?
                    new TextSelection(selectedUnit.Anchor, hit.Position) : selectedUnit;
        }
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

        if (!extend && unit is { } textUnit &&
            await worker.SelectTextUnitAsync(current.Target, textUnit, hit.Position, token) is { } unitEdit)
        {
            dragTextUnit = unitEdit.Outcome.Selection;
            editorInfo = editorInfo! with { Selection = unitEdit.Outcome.Selection };
        }

        if (!extend && !platformComposing)
        {
            ClearTableObjectSelection();
            ClearPictureObjectSelection();
            EmptyInputControl();
        }
        preferredX = null;
        directEditing = true;
        await UpdateDirectGeometryAsync(followCaret: false, focus: !extend);
        return true;
    }

    private static bool SharesTextLeaf(TextPosition left, TextPosition right) =>
        left.Section == right.Section && left.Paragraph == right.Paragraph &&
        left.CellPath.SequenceEqual(right.CellPath);

    private static bool SharesTableCell(TextPosition left, TextPosition right)
    {
        int depth = left.CellPath.Length;
        if (depth == 0 || right.CellPath.Length != depth || left.Section != right.Section ||
            left.Paragraph != right.Paragraph)
            return false;

        CellPathEntry a = left.CellPath[^1];
        CellPathEntry b = right.CellPath[^1];
        return a.Control == b.Control && a.Cell == b.Cell &&
            left.CellPath.AsSpan(0, depth - 1).SequenceEqual(right.CellPath.AsSpan(0, depth - 1));
    }

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
        {
            directQueueRunning = true;
            var finished = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            pendingDirectEdits = finished.Task;
            // TextChanging can run during XAML layout. Capture its text in
            // order and leave visual tree and document operations to the queue.
            if (!DispatcherQueue.TryEnqueue(async () =>
            {
                try
                {
                    await DrainDirectEditsAsync();
                }
                finally
                {
                    finished.TrySetResult();
                }
            }))
            {
                directActions.Clear();
                directQueueRunning = false;
                finished.TrySetResult();
            }
        }
        return pendingDirectEdits;
    }

    private async Task SettleDirectInputAsync()
    {
        if (platformCompositionFinished is { Task: var finished })
        {
            PageCanvas.Focus(FocusState.Programmatic);
            try
            {
                await finished.WaitAsync(TimeSpan.FromSeconds(1));
            }
            catch (TimeoutException)
            {
                StatusText.Text = "입력기 조합이 끝난 뒤 다시 시도하세요.";
            }
        }

        await pendingDirectEdits;
    }

    private async Task DrainDirectEditsAsync()
    {
        editorBusy = true;
        RefreshButtons();
        try
        {
            while (!closing)
            {
                try
                {
                    if (directCaretUpdatesPending >= MaxDeferredCaretUpdates ||
                        (directCaretUpdatesPending != 0 && directActions.Count == 0))
                    {
                        // Present the newest caret when idle and periodically
                        // while rapid input keeps the queue populated.
                        await UpdateDirectGeometryAsync(followCaret: true);
                    }
                    else if (directActions.TryDequeue(out Func<Task>? action))
                    {
                        await action();
                    }
                    else
                    {
                        break;
                    }
                }
                catch (Exception error)
                {
                    try
                    {
                        await CancelNativePreviewAsync();
                    }
                    catch (Exception cleanup)
                    {
                        System.Diagnostics.Debug.WriteLine(cleanup);
                    }
                    if (error is not NativeRequestException { IsUnsupportedTextPosition: true })
                        StatusText.Text = "문서를 편집할 수 없습니다. " + error.Message;
                }
            }
        }
        finally
        {
            directQueueRunning = false;
            editorBusy = false;
            if (!closing)
                UpdateAutomationDocument(page);
            RefreshButtons();
        }
    }

    private void OnDirectTextChanging(TextBox sender, TextBoxTextChangingEventArgs args)
    {
        if (changingInput || platformComposing || !directEditing || !CanQueueDirectEdit)
            return;

        string current = sender.Text;
        DirectInputChange change = DirectInputChange.Between(capturedInputText, current);
        capturedInputText = current;
        if (change.RemovedSuffix.Length != 0 || change.InsertedSuffix.Length != 0)
            EnqueueDirectEdit(() => ApplyDirectInputChangeAsync(change));
    }

    private void EmptyInputControl()
    {
        capturedInputText = "";
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
        directCaretUpdatesPending++;
    }

    private async Task ApplyDirectInputChangeAsync(DirectInputChange change)
    {
        if (change.RemovedSuffix.Length == 0)
        {
            await InsertDirectTextAsync(change.InsertedSuffix);
            return;
        }

        if (!directEditing || page is not PageInfo current || editorInfo is not { Editing: true, Selection: { } selection })
            return;

        // The platform can rewrite characters it previously delivered. Replace
        // that suffix only if it still sits immediately before the native caret.
        CancellationToken token = CancellationToken.None;
        ImeEditContext context = await worker.ReadImeEditContextAsync(current.Target, token);
        if (context.CompositionId is not null ||
            !SamePosition(selection.Anchor, selection.Focus) ||
            !SamePosition(context.Selection.Anchor, context.Selection.Focus) ||
            !SamePosition(context.Selection.Focus, selection.Focus) ||
            !SamePosition(context.Context.Position, selection.Focus))
            throw new InvalidOperationException("입력기 조합 중 문서의 커서 위치가 바뀌었습니다.");

        ImeReplacement replacement = change.ReplacementAtCaret(context.Context);
        EditResult edit = await worker.CommitImeAsync(current.Target, context, replacement, null, token);
        preferredX = null;
        await RefreshEditedPageAsync(current, edit);
        directCaretUpdatesPending++;
    }

    private void OnDirectCompositionStarted(TextBox sender, TextCompositionStartedEventArgs args)
    {
        if (!directEditing || changingInput || platformComposing || !CanQueueDirectEdit)
            return;

        compositionInputPrefix = capturedInputText;
        platformComposing = true;
        platformCompositionFinished = new(TaskCreationOptions.RunContinuationsAsynchronously);
        previewVersion++;
        EnqueueDirectEdit(BeginNativePreviewAsync);
    }

    private void OnDirectCompositionChanged(TextBox sender, TextCompositionChangedEventArgs args)
    {
        if (!directEditing || !platformComposing)
            return;

        long version = ++previewVersion;
        string current = sender.Text;
        if (!current.StartsWith(compositionInputPrefix, StringComparison.Ordinal))
        {
            EnqueueDirectEdit(() => version == previewVersion ? CancelNativePreviewAsync() : Task.CompletedTask);
            return;
        }

        string text = current[compositionInputPrefix.Length..];
        EnqueueDirectEdit(() => version == previewVersion ? SetNativePreviewAsync(text, updateView: true) :
            Task.CompletedTask);
    }

    private void OnDirectCompositionEnded(TextBox sender, TextCompositionEndedEventArgs args)
    {
        if (!platformComposing)
            return;
        string current = sender.Text;
        DirectInputChange change = DirectInputChange.Between(compositionInputPrefix, current);
        TaskCompletionSource? finished = platformCompositionFinished;
        platformCompositionFinished = null;
        compositionInputPrefix = "";
        platformComposing = false;
        previewVersion++;
        capturedInputText = current;
        EnqueueDirectEdit(() => FinishNativePreviewAsync(change));
        finished?.TrySetResult();
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
        EditResult preview;
        try
        {
            ImeEditContext context = await worker.ReadImeEditContextAsync(current.Target, token);
            Utf16Range range = context.Preedit ?? throw new InvalidOperationException("입력기 조합 범위가 없습니다.");
            preview = await worker.ContinueCompositionAsync(current.Target, context,
                new ImeReplacement(range, text), null, token);
        }
        catch (InvalidOperationException error) when (
            error.Message.EndsWith("(limitExceeded)", StringComparison.Ordinal) ||
            error.Message.EndsWith("(unsupported)", StringComparison.Ordinal))
        {
            // Preedit is optional. Restore the original range so the IME's
            // final text can still be inserted in one committed operation.
            await CancelNativePreviewAsync();
            return;
        }

        previewText = text;
        await RefreshEditedPageAsync(current, preview);
        if (updateView)
            await UpdateDirectGeometryAsync(followCaret: true);
    }

    private async Task FinishNativePreviewAsync(DirectInputChange change)
    {
        if (change.RemovedSuffix.Length != 0)
        {
            await CancelNativePreviewAsync();
            await ApplyDirectInputChangeAsync(change);
            return;
        }

        string text = change.InsertedSuffix;
        if (text.Length == 0)
        {
            await CancelNativePreviewAsync();
        }
        else
        {
            await SetNativePreviewAsync(text, updateView: false);
            if (nativePreviewAvailable && page is PageInfo current &&
                editorInfo?.CompositionId is ulong composition)
            {
                EditResult edit = await worker.CommitCompositionAsync(current.Target, composition, CancellationToken.None);
                await RefreshEditedPageAsync(current, edit);
            }
            else
            {
                await InsertDirectTextAsync(text);
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

    private async void OnDirectPreviewKeyDown(object sender, KeyRoutedEventArgs args)
    {
        if (!directEditing || KeyDown(VirtualKey.Menu))
            return;

        bool control = KeyDown(VirtualKey.Control);
        bool shift = KeyDown(VirtualKey.Shift);
        if (control && args.Key == VirtualKey.S)
        {
            args.Handled = true;
            await SaveDocumentAsync(saveAs: shift);
            RestorePageEditFocus();
            return;
        }
        if (platformComposing)
            return;

        if (control && !shift && TryZoomFromKey(args.Key))
        {
            args.Handled = true;
            return;
        }

        if (control && args.Key is VirtualKey.Z or VirtualKey.Y)
        {
            args.Handled = true;
            EmptyInputControl();
            bool redo = args.Key == VirtualKey.Y || shift;
            EnqueueDirectEdit(() => EditHistoryAsync(redo));
            return;
        }
        if (control && args.Key is VirtualKey.C or VirtualKey.X or VirtualKey.V)
        {
            args.Handled = true;
            if (args.Key != VirtualKey.C)
                EmptyInputControl();
            EnqueueClipboardAction(args.Key);
            return;
        }

        if (!control && args.Key == VirtualKey.Tab &&
            editorInfo?.Selection is { Focus.CellPath.Length: > 0 })
        {
            args.Handled = true;
            EmptyInputControl();
            EnqueueDirectEdit(() => NavigateDirectTableCellAsync(shift));
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
            EmptyInputControl();
            EnqueueDirectEdit(() => ApplyDirectKeyAsync(command, shift));
        }
    }

    private static bool KeyDown(VirtualKey key) =>
        (InputKeyboardSource.GetKeyStateForCurrentThread(key) & CoreVirtualKeyStates.Down) != 0;

    private async Task NavigateDirectTableCellAsync(bool previous)
    {
        if (page is not PageInfo current || editorInfo?.Selection is not { Focus.CellPath.Length: > 0 })
            return;

        EditResult edit = await worker.NavigateTableCellAsync(current.Target, previous, CancellationToken.None);
        preferredX = null;
        await RefreshEditedPageAsync(current, edit);
        await UpdateDirectGeometryAsync(followCaret: true);
    }

    private async Task ApplyDirectKeyAsync(NativeEditKey key, bool extendSelection)
    {
        if (page is not PageInfo current || editorInfo?.Selection is not { } selection)
            return;

        CancellationToken token = CancellationToken.None;
        if (selection.Focus.CellPath.Length != 0 && key != NativeEditKey.SplitParagraph &&
            !await worker.SupportsCellTextNavigationAsync(token))
        {
            if (key is NativeEditKey.DeleteBackward or NativeEditKey.DeleteForward &&
                !SamePosition(selection.Anchor, selection.Focus))
            {
                EditResult removed = await worker.ReplaceAsync(current.Target, "", token);
                await RefreshEditedPageAsync(current, removed);
                await UpdateDirectGeometryAsync(followCaret: true);
                return;
            }
            StatusText.Text = "이 Native 라이브러리는 표 셀 방향키와 삭제 명령을 지원하지 않습니다.";
            return;
        }

        NativeCommandResult result = await worker.ExecuteEditCommandAsync(current.Target, key, token, extendSelection,
            preferredX);
        preferredX = result.PreferredX;
        await RefreshEditedPageAsync(current, result.Edit);
        await UpdateDirectGeometryAsync(followCaret: true);
    }

    private async Task UpdateDirectGeometryAsync(bool followCaret, bool focus = false)
    {
        directCaretUpdatesPending = 0;
        if (!directEditing || page is not PageInfo current || editorInfo?.Selection is not { } selection)
            return;

        CancellationToken token = CancellationToken.None;
        PageCaret? caret = await worker.CaretAsync(current.Target, selection.Focus, token);
        SelectionRectangle[] rectangles = SamePosition(selection.Anchor, selection.Focus) ? [] :
            await worker.GetSelectionRectanglesAsync(current, selection, token);
        if (page is not { } active || active.Target != current.Target || active.FontGeneration != current.FontGeneration)
            return;

        directGeometry = (current.Target, current.FontGeneration, caret, rectangles);
        if (followCaret && caret is not null &&
            await EnsurePagePlacementAsync(current, caret.Page, token) is { } placement)
            ShowDocumentPoint(placement, caret.X, caret.Y, caret.Height);
        DrawDirectGeometry();
        if (focus && caret is not null)
        {
            PageCanvas.UpdateLayout();
            DirectInput.Focus(FocusState.Programmatic);
        }
    }

    private void DrawDirectGeometry()
    {
        DirectEditOverlay.Children.Clear();
        if (page is not PageInfo current || directGeometry is not { } geometry ||
            geometry.Target != current.Target || geometry.FontGeneration != current.FontGeneration ||
            continuousPages is not { } layout)
            return;

        foreach (SelectionRectangle rectangle in geometry.Selection)
            if (layout.TryGet(rectangle.Page, out PagePlacement placement) &&
                rectangle.Bounds.Width > 0 && rectangle.Bounds.Height > 0)
                DrawMark(placement.Left + rectangle.Bounds.X, placement.Top + rectangle.Bounds.Y,
                    rectangle.Bounds.Width * zoom, rectangle.Bounds.Height * zoom,
                    (Brush)DirectEditOverlay.Resources["DirectSelectionBrush"], 0.32);

        if (geometry.Caret is not { Height: > 0 } caret || !layout.TryGet(caret.Page, out PagePlacement caretPage))
            return;

        double x = caretPage.Left + caret.X;
        double y = caretPage.Top + caret.Y;
        if (geometry.Selection.Length == 0)
            DrawMark(x, y, 1.5, caret.Height * zoom, (Brush)DirectEditOverlay.Resources["DirectCaretBrush"], 1);

        // Keep the input service's native candidate window anchored while it composes.
        if (platformComposing)
            return;

        DirectInput.Visibility = Visibility.Visible;
        DirectInput.Height = Math.Max(18, caret.Height * zoom);
        DirectInput.FontSize = Math.Clamp(caret.Height * zoom * 0.8, 9, 48);
        Canvas.SetLeft(DirectInput, x * zoom);
        Canvas.SetTop(DirectInput, y * zoom);
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

    private void ClearDirectGeometry()
    {
        directGeometry = null;
        DirectEditOverlay.Children.Clear();
        ClearTableObjectSelection();
    }

    private void DismissDirectEditor()
    {
        directEditing = false;
        directCaretUpdatesPending = 0;
        platformComposing = false;
        platformCompositionFinished?.TrySetResult();
        platformCompositionFinished = null;
        compositionInputPrefix = "";
        dragPointer = null;
        dragAnchor = null;
        dragTextUnit = null;
        lastTextClick = null;
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
