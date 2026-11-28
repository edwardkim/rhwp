using System.Diagnostics;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Shapes;
using Windows.Foundation;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private sealed record TableContext(PageInfo Page, TextPosition Position, NativeTableObject Table);

    private sealed class SelectedTable(TableContext context)
    {
        public TableContext Context { get; } = context;
        public Dictionary<uint, CaretBounds?> Fragments { get; } = new()
        {
            [context.Page.Page] = context.Table.Bounds,
        };
        public bool Loading { get; set; }
    }

    private SelectedTable? selectedTable;

    private bool IsCurrentTablePage(PageInfo requested) => !closing && page is PageInfo current &&
        requested.Target == current.Target && requested.FontGeneration == current.FontGeneration;

    private bool IsSelectedTable(TableContext context) => selectedTable is { Context: var selected } &&
        selected.Page.Target == context.Page.Target && selected.Page.FontGeneration == context.Page.FontGeneration &&
        selected.Table.Address == context.Table.Address;

    private async Task<TableContext?> FindTableContextAsync(PageInfo current, Point location)
    {
        CancellationToken token = CancellationToken.None;
        if (!await worker.SupportsTableObjectsAsync(token))
            return null;

        TextHit? hit = await worker.HitTestAsync(current, location.X, location.Y, token);
        if (hit is null || hit.Position.CellPath.Length != 1 || hit.Caret?.Page != current.Page)
            return null;

        NativeTableObject table = await worker.GetTableObjectAsync(current, hit.Position, token);
        if (!IsCurrentTablePage(current) || table.Bounds is not { Width: > 0, Height: > 0 } bounds ||
            location.X < bounds.X || location.X > bounds.X + bounds.Width ||
            location.Y < bounds.Y || location.Y > bounds.Y + bounds.Height)
            return null;

        return new TableContext(current, hit.Position, table);
    }

    private async Task SelectTableObjectAsync(TableContext context)
    {
        if (!IsCurrentTablePage(context.Page) || HasActiveComposition)
            return;
        if (IsSelectedTable(context))
        {
            RestorePageEditFocus();
            return;
        }

        CancellationToken token = CancellationToken.None;
        TextSelection caret = new(context.Position, context.Position);
        if (editorInfo is { Editing: true } info && info.Target == context.Page.Target)
        {
            if (info.Selection is not { } selection || !SamePosition(selection.Anchor, context.Position) ||
                !SamePosition(selection.Focus, context.Position))
            {
                EditResult edit = await worker.SetSelectionAsync(context.Page.Target, caret, token);
                editorInfo = info with { Selection = edit.Outcome.Selection };
            }
        }
        else
        {
            editorInfo = await worker.BeginEditAsync(context.Page.Target, caret, token);
        }

        if (!IsCurrentTablePage(context.Page))
            return;
        DismissDirectEditor();
        selectedTable = new SelectedTable(context);
        DrawTableObjectSelection();
        StatusText.Text = TableSelectionStatus(context.Table);
        RestorePageEditFocus();
    }

    private async Task ShowTableObjectDialogAsync(TableContext context, bool resize)
    {
        await SettleDirectInputAsync();
        if (!DocumentActionsAvailable || !IsCurrentTablePage(context.Page))
            return;
        await EnqueueDirectEdit(() => SelectTableObjectAsync(context));
        if (!DocumentActionsAvailable || !IsSelectedTable(context) || page is not PageInfo active)
            return;

        editorBusy = true;
        RefreshButtons();
        try
        {
            var dialog = new TableObjectDialog(context.Table, resize) { XamlRoot = Root.XamlRoot };
            if (await dialog.ShowAsync() != ContentDialogResult.Primary || !IsCurrentTablePage(context.Page) ||
                !IsSelectedTable(context))
                return;

            CancellationToken token = CancellationToken.None;
            EditResult edit;
            if (dialog.SizeChange is { } size)
            {
                edit = await worker.ResizeTableObjectAsync(active.Target, context.Table.Address,
                    size.Width, size.Height, token);
            }
            else if (dialog.PropertyChanges is { } properties)
            {
                edit = await worker.SetTableObjectPropertiesAsync(active.Target, context.Table.Address, properties,
                    token);
            }
            else
            {
                return;
            }

            await RefreshEditedPageAsync(active, edit);
            if (edit.Outcome.DocumentChanged)
                await RestoreTableObjectAfterEditAsync(edit);
            StatusText.Text = edit.Outcome.DocumentChanged
                ? (resize ? "표 크기를 변경했습니다." : "표 속성을 변경했습니다.")
                : "표 설정이 변경되지 않았습니다.";
        }
        catch (Exception error)
        {
            StatusText.Text = "표 설정을 적용할 수 없습니다. " + error.Message;
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
            RestorePageEditFocus();
        }
    }

    private async Task RestoreTableObjectAfterEditAsync(EditResult edit)
    {
        if (page is not PageInfo current || current.Target != edit.Target)
            return;

        CancellationToken token = CancellationToken.None;
        TextPosition position = edit.Outcome.Selection.Focus;
        PageCaret? caret = await worker.CaretAsync(current.Target, position, token);
        PageInfo selectedPage = current with { Page = caret?.Page ?? current.Page };
        NativeTableObject table = await worker.GetTableObjectAsync(selectedPage, position, token);
        if (!IsCurrentTablePage(current))
            return;
        selectedTable = new SelectedTable(new TableContext(selectedPage, position, table));
        DrawTableObjectSelection();
    }

    private void DrawTableObjectSelection()
    {
        if (selectedTable is not { } selection || continuousPages is not { } layout)
            return;
        if (!IsCurrentTablePage(selection.Context.Page))
        {
            ClearTableObjectSelection();
            return;
        }
        TableObjectOverlay.Children.Clear();

        var requested = new List<uint>();
        foreach (PagePlacement placement in VisiblePages(layout))
        {
            if (!selection.Fragments.TryGetValue(placement.Index, out CaretBounds? bounds))
            {
                if (requested.Count < PagesPerRender)
                    requested.Add(placement.Index);
                continue;
            }
            if (bounds is not { Width: > 0, Height: > 0 })
                continue;
            var outline = new Rectangle
            {
                Stroke = (Brush)DirectEditOverlay.Resources["DirectSelectionBrush"],
                StrokeThickness = 2,
                Width = bounds.Width * zoom,
                Height = bounds.Height * zoom,
            };
            Canvas.SetLeft(outline, (placement.Left + bounds.X) * zoom);
            Canvas.SetTop(outline, (placement.Top + bounds.Y) * zoom);
            TableObjectOverlay.Children.Add(outline);
        }
        if (requested.Count > 0 && !selection.Loading)
            _ = LoadTableObjectFragmentsAsync(selection, requested);
    }

    private async Task LoadTableObjectFragmentsAsync(SelectedTable selection, IReadOnlyList<uint> pages)
    {
        selection.Loading = true;
        TableContext context = selection.Context;
        try
        {
            foreach (uint index in pages)
            {
                NativeTableObject geometry = await worker.GetTableObjectAsync(context.Page with { Page = index },
                    context.Position, CancellationToken.None);
                if (!ReferenceEquals(selectedTable, selection) || !IsCurrentTablePage(context.Page))
                    return;
                selection.Fragments[index] = geometry.Bounds;
            }
        }
        catch (Exception error)
        {
            Debug.WriteLine(error);
            return;
        }
        finally
        {
            selection.Loading = false;
        }
        DrawTableObjectSelection();
    }

    private void ClearTableObjectSelection()
    {
        if (selectedTable is not { Context.Table: var table })
            return;
        selectedTable = null;
        TableObjectOverlay.Children.Clear();
        if (StatusText.Text == TableSelectionStatus(table))
            StatusText.Text = "";
    }

    private static string TableSelectionStatus(NativeTableObject table) =>
        $"표 전체 선택됨 ({table.RowCount}행 × {table.ColumnCount}열)";
}
