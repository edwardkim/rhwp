using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Windows.Foundation;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private async void OnPageRightTapped(object sender, RightTappedRoutedEventArgs args)
    {
        Point position = args.GetPosition(PageCanvas);
        if (dragPointer is not null || !AcceptPageInput(position, out PageInfo current, out Point location))
            return;

        args.Handled = true;
        bool textContext = false;
        TableContext? tableContext = null;
        await EnqueueDirectEdit(async () =>
        {
            tableContext = await FindTableContextAsync(current, location);
            if (tableContext is not null && IsSelectedTable(tableContext))
                return;
            textContext = IsWithinDocumentSelection(current, location) ||
                await MovePointerSelectionAsync(current, location, extend: false);
        });

        if (!DocumentActionsAvailable || page?.Target != current.Target)
            return;

        var menu = new MenuFlyout();
        if (textContext)
        {
            menu.Items.Add(ContextAction("잘라내기", "ContextCut", OnCutClicked, CutMenuItem.IsEnabled, "Ctrl+X"));
            menu.Items.Add(ContextAction("복사", "ContextCopy", OnCopyClicked, CopyMenuItem.IsEnabled, "Ctrl+C"));
            menu.Items.Add(ContextAction("붙여넣기", "ContextPaste", OnPasteClicked, PasteMenuItem.IsEnabled, "Ctrl+V"));
            menu.Items.Add(new MenuFlyoutSeparator());
            menu.Items.Add(ContextAction("전체 선택", "ContextSelectAll", OnContextSelectAll, true, "Ctrl+A"));
            if (InsertTableMenuItem.IsEnabled)
                menu.Items.Add(ContextAction("표 삽입…", "ContextInsertTable", OnInsertTableClicked, true));
            menu.Items.Add(new MenuFlyoutSeparator());
        }

        if (tableContext is TableContext table && IsCurrentTablePage(table.Page))
        {
            menu.Items.Add(ContextAction("표 전체 선택", "ContextSelectTableObject",
                async (_, _) => await EnqueueDirectEdit(() => SelectTableObjectAsync(table)), true));
            if (table.Table.CanResize)
                menu.Items.Add(ContextAction("표 크기…", "ContextResizeTableObject",
                    async (_, _) => await ShowTableObjectDialogAsync(table, resize: true), true));
            menu.Items.Add(ContextAction("표 속성…", "ContextTableObjectProperties",
                async (_, _) => await ShowTableObjectDialogAsync(table, resize: false), true));
            menu.Items.Add(new MenuFlyoutSeparator());
        }

        menu.Items.Add(ContextAction("찾기", "ContextFind", OnFindClicked, true, "Ctrl+F"));
        menu.Items.Add(ContextAction("바꾸기", "ContextReplace", OnFindReplaceClicked, true, "Ctrl+H"));
        menu.ShowAt(PageCanvas, new FlyoutShowOptions { Position = position, ShowMode = FlyoutShowMode.Transient });
    }

    private bool IsWithinDocumentSelection(PageInfo current, Point location) =>
        directGeometry is { } geometry && geometry.Target == current.Target &&
        geometry.FontGeneration == current.FontGeneration && geometry.Selection.Any(rectangle =>
            rectangle.Page == current.Page && location.X >= rectangle.Bounds.X && location.Y >= rectangle.Bounds.Y &&
            location.X <= rectangle.Bounds.X + rectangle.Bounds.Width && location.Y <= rectangle.Bounds.Y + rectangle.Bounds.Height);

    private async void OnContextSelectAll(object sender, RoutedEventArgs args) =>
        await EnqueueDirectEdit(() => ApplyDirectKeyAsync(NativeEditKey.SelectAll, extendSelection: false));

    private static MenuFlyoutItem ContextAction(string text, string id, RoutedEventHandler click, bool enabled,
        string? shortcut = null)
    {
        var item = new MenuFlyoutItem { Text = text, IsEnabled = enabled };
        if (shortcut is not null)
            item.KeyboardAcceleratorTextOverride = shortcut;
        AutomationProperties.SetAutomationId(item, id);
        item.Click += click;
        return item;
    }
}
