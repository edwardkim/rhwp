using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Shapes;
using Windows.System;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private DocumentSearchResult? searchResults;
    private int searchIndex;
    private bool searchComposing;
    private bool searchUiReady;
    private bool? replaceAllAvailable;
    private (DocumentTarget Target, ulong FontGeneration, SearchRectangle[] Rectangles)? searchHighlight;

    private void OnFindClicked(object sender, RoutedEventArgs args) => ShowSearchPanel(replace: false);
    private void OnFindReplaceClicked(object sender, RoutedEventArgs args) => ShowSearchPanel(replace: true);
    private void OnCloseSearchClicked(object sender, RoutedEventArgs args) => CloseSearchPanel();

    private void ShowSearchPanel(bool replace)
    {
        if (!DocumentActionsAvailable || page is null)
            return;

        SearchPanel.Visibility = Visibility.Visible;
        ReplaceRow.Visibility = replace ? Visibility.Visible : Visibility.Collapsed;
        if (replace && replaceAllAvailable is null)
            _ = LoadReplaceAllCapabilityAsync();
        RefreshSearchActions();
        DispatcherQueue.TryEnqueue(() =>
        {
            if (SearchPanel.Visibility == Visibility.Visible && !closing)
            {
                SearchQuery.Focus(FocusState.Programmatic);
                SearchQuery.SelectAll();
            }
        });
    }

    private void CloseSearchPanel()
    {
        if (editorBusy || searchComposing)
            return;

        SearchPanel.Visibility = Visibility.Collapsed;
        InvalidateSearchResults();
        RestorePageEditFocus();
    }

    private void OnSearchQueryChanged(object sender, TextChangedEventArgs args) => SearchCriteriaChanged();
    private void OnSearchCaseChanged(object sender, RoutedEventArgs args) => SearchCriteriaChanged();

    private void SearchCriteriaChanged()
    {
        if (!searchUiReady)
            return;

        InvalidateSearchResults();
        RefreshSearchActions();
    }

    private void OnSearchCompositionStarted(TextBox sender, TextCompositionStartedEventArgs args) =>
        searchComposing = true;

    private void OnSearchCompositionEnded(TextBox sender, TextCompositionEndedEventArgs args) =>
        searchComposing = false;

    private async void OnSearchKeyDown(object sender, KeyRoutedEventArgs args)
    {
        if (searchComposing || editorBusy)
            return;

        if (args.Key == VirtualKey.Escape)
        {
            args.Handled = true;
            CloseSearchPanel();
        }
        else if (args.Key == VirtualKey.Enter && SearchQuery.FocusState != FocusState.Unfocused)
        {
            args.Handled = true;
            await FindNextAsync(1);
        }
    }

    private async void OnPreviousMatchClicked(object sender, RoutedEventArgs args) => await FindNextAsync(-1);
    private async void OnNextMatchClicked(object sender, RoutedEventArgs args) => await FindNextAsync(1);

    private Task FindNextAsync(int step)
    {
        string query = SearchQuery.Text;
        bool caseSensitive = SearchCaseSensitive.IsChecked == true;
        if (query.Length == 0)
            return Task.CompletedTask;

        return RunSearchActionAsync(async current =>
        {
            if (searchResults is not DocumentSearchResult results || results.Target != current.Target)
            {
                if (!await worker.CanSearchAsync(CancellationToken.None))
                    throw new InvalidOperationException("사용 중인 Native 라이브러리는 문서 검색을 지원하지 않습니다.");

                DocumentSearchResult found = await worker.SearchAsync(current.Target, query, caseSensitive,
                    CancellationToken.None);
                if (SearchQuery.Text != query || (SearchCaseSensitive.IsChecked == true) != caseSensitive)
                    return;

                searchResults = found;
                searchIndex = step > 0 ? 0 : Math.Max(0, found.Matches.Length - 1);
            }
            else if (results.Matches.Length > 0)
            {
                searchIndex = (searchIndex + step + results.Matches.Length) % results.Matches.Length;
            }

            await ShowCurrentSearchMatchAsync(current);
        });
    }

    private async void OnReplaceMatchClicked(object sender, RoutedEventArgs args)
    {
        if (searchResults is not DocumentSearchResult results || results.Matches.Length == 0 ||
            !results.Matches[searchIndex].Editable)
            return;

        int index = searchIndex;
        string replacement = ReplacementText.Text;
        await RunSearchActionAsync(async current =>
        {
            if (results.Target != current.Target)
                throw new InvalidOperationException("문서가 변경되었습니다. 다시 검색하세요.");

            CancellationToken token = CancellationToken.None;
            NativeDocumentInfo info = editorInfo is { } known && known.Target == current.Target
                ? known : await worker.GetDocumentInfoAsync(current.Target, token);
            if (!info.Editing)
                info = await worker.BeginEditAsync(current.Target, results.Matches[index].Selection, token);
            editorInfo = info;

            EditResult edit = await worker.ReplaceSearchMatchAsync(results, index, replacement, token);
            InvalidateSearchResults();
            if (edit.Outcome.DocumentChanged)
            {
                await RefreshEditedPageAsync(current, edit);
                if (directEditing)
                    await UpdateDirectGeometryAsync(followCaret: false);
                SearchStatus.Text = "바꿨습니다. 다음 항목을 찾으려면 다시 검색하세요.";
            }
            else
            {
                if (editorInfo is NativeDocumentInfo active)
                    editorInfo = active with { Selection = edit.Outcome.Selection };
                SearchStatus.Text = "변경된 내용이 없습니다.";
            }
        });
    }

    private async Task LoadReplaceAllCapabilityAsync()
    {
        replaceAllAvailable = false;
        try
        {
            replaceAllAvailable = await worker.CanReplaceAllAsync(CancellationToken.None);
            if (!closing && replaceAllAvailable == true)
            {
                ReplaceAllButton.Visibility = Visibility.Visible;
                RefreshSearchActions();
            }
        }
        catch (Exception error)
        {
            System.Diagnostics.Debug.WriteLine(error);
            replaceAllAvailable = null;
        }
    }

    private async void OnReplaceAllClicked(object sender, RoutedEventArgs args)
    {
        string query = SearchQuery.Text;
        if (replaceAllAvailable != true || query.Length == 0)
            return;

        string replacement = ReplacementText.Text;
        bool caseSensitive = SearchCaseSensitive.IsChecked == true;
        await RunSearchActionAsync(async current =>
        {
            CancellationToken token = CancellationToken.None;
            NativeDocumentInfo info = editorInfo is { } known && known.Target == current.Target
                ? known : await worker.GetDocumentInfoAsync(current.Target, token);
            if (!info.Editing)
            {
                // A viewer needs an editable position to enter the native editor.
                DocumentSearchResult found = await worker.SearchAsync(current.Target, query, caseSensitive, token);
                if (found.Truncated)
                    throw new InvalidOperationException("검색 결과 한도에 도달했습니다. 검색 범위를 좁혀 다시 시도하세요.");
                SearchMatch? first = found.Matches.FirstOrDefault(match => match.Editable);
                if (first is null)
                {
                    InvalidateSearchResults();
                    SearchStatus.Text = ReplaceAllStatus(0, found.Matches.Length, found.OmittedContent);
                    return;
                }
                info = await worker.BeginEditAsync(current.Target, first.Selection, token);
            }
            editorInfo = info;

            ReplaceAllResult result = await worker.ReplaceAllMatchesAsync(current.Target, query, caseSensitive,
                replacement, token);
            InvalidateSearchResults();
            if (result.Edit.Outcome.DocumentChanged)
            {
                await RefreshEditedPageAsync(current, result.Edit);
                if (directEditing)
                    await UpdateDirectGeometryAsync(followCaret: false);
            }
            SearchStatus.Text = ReplaceAllStatus(result.ReplacedCount, result.SkippedReadOnly, result.OmittedContent);
        });
    }

    private static string ReplaceAllStatus(int replaced, int skipped, bool omitted) =>
        (replaced == 0 ? "바뀐 내용이 없습니다." : $"{replaced}개 항목을 바꿨습니다.")
        + (skipped == 0 ? "" : $" · 읽기 전용 {skipped}개")
        + (omitted ? " · 검색할 수 없는 문서 내용이 있습니다." : "");

    private async Task RunSearchActionAsync(Func<PageInfo, Task> action)
    {
        if (!DocumentActionsAvailable || searchComposing || page is not PageInfo current)
            return;

        editorBusy = true;
        RefreshButtons();
        try
        {
            await action(current);
        }
        catch (Exception error)
        {
            SearchStatus.Text = "검색 작업을 처리할 수 없습니다. " + error.Message;
        }
        finally
        {
            editorBusy = false;
            RefreshButtons();
        }
    }

    private async Task ShowCurrentSearchMatchAsync(PageInfo current)
    {
        searchHighlight = null;
        DrawSearchHighlight(null);
        if (searchResults is not DocumentSearchResult results)
            return;

        if (results.Matches.Length == 0)
        {
            SearchStatus.Text = "일치하는 내용을 찾지 못했습니다." + SearchLimitMessage(results);
            return;
        }

        SearchMatch match = results.Matches[searchIndex];
        SearchRectangle[] rectangles = await worker.GetSearchRectanglesAsync(current, results, searchIndex,
            CancellationToken.None);
        if (!ReferenceEquals(searchResults, results))
            return;

        SearchRectangle? first = rectangles.OrderBy(rectangle => rectangle.Page)
            .ThenBy(rectangle => rectangle.Bounds.Y).ThenBy(rectangle => rectangle.Bounds.X).FirstOrDefault();
        SearchStatus.Text = $"{searchIndex + 1} / {results.Matches.Length}"
            + (first is null ? " · 이 항목의 화면 위치를 확인할 수 없습니다." : $" · {first.Page + 1}쪽")
            + (match.Editable ? "" : " · 읽기 전용")
            + SearchLimitMessage(results);
        if (first is null)
            return;

        PagePlacement? placement = await EnsurePagePlacementAsync(current, first.Page, CancellationToken.None);
        if (placement is not { } destination || !ReferenceEquals(searchResults, results))
            return;

        searchHighlight = (current.Target, current.FontGeneration, rectangles);
        DrawSearchHighlight(current);
        ShowDocumentPoint(destination, first.Bounds.X, first.Bounds.Y, first.Bounds.Height);
        QueueRender();
    }

    private static string SearchLimitMessage(DocumentSearchResult results) =>
        (results.Truncated ? " · 검색 범위 또는 결과 한도에 도달했습니다." : "")
        + (results.OmittedContent ? " · 검색할 수 없는 문서 내용이 있습니다." : "");

    private void InvalidateSearchResults()
    {
        searchResults = null;
        searchHighlight = null;
        SearchStatus.Text = "";
        DrawSearchHighlight(null);
    }

    private void RefreshSearchActions()
    {
        if (!searchUiReady)
            return;

        bool available = DocumentActionsAvailable;
        if ((page is not null && searchResults is { } results && page.Target != results.Target) ||
            (page is null && available))
            InvalidateSearchResults();
        if (page is null && available && SearchPanel.Visibility == Visibility.Visible)
            SearchPanel.Visibility = Visibility.Collapsed;

        bool enabled = available && page is not null;
        FindToolbarButton.IsEnabled = enabled;
        SearchQuery.IsReadOnly = !enabled;
        ReplacementText.IsReadOnly = !enabled;
        SearchCaseSensitive.IsEnabled = enabled;
        PreviousMatchButton.IsEnabled = enabled && SearchQuery.Text.Length > 0;
        NextMatchButton.IsEnabled = PreviousMatchButton.IsEnabled;
        ReplaceMatchButton.IsEnabled = enabled && searchResults is { Matches.Length: > 0 } current
            && current.Matches[searchIndex].Editable;
        ReplaceAllButton.IsEnabled = PreviousMatchButton.IsEnabled && replaceAllAvailable == true;
        CloseSearchButton.IsEnabled = !editorBusy;
    }

    private void DrawSearchHighlight(PageInfo? current)
    {
        if (!searchUiReady)
            return;

        SearchHighlightCanvas.Children.Clear();
        if (current is null || searchHighlight is not { } highlight ||
            highlight.Target != current.Target || highlight.FontGeneration != current.FontGeneration ||
            continuousPages is not { } layout)
            return;

        var brush = (Brush)SearchHighlightCanvas.Resources["SearchHighlightBrush"];
        foreach (SearchRectangle rectangle in highlight.Rectangles)
        {
            if (!layout.TryGet(rectangle.Page, out PagePlacement placement))
                continue;

            CaretBounds bounds = rectangle.Bounds;
            double left = Math.Clamp(bounds.X, 0, placement.Width);
            double top = Math.Clamp(bounds.Y, 0, placement.Height);
            double right = Math.Clamp(bounds.X + bounds.Width, 0, placement.Width);
            double bottom = Math.Clamp(bounds.Y + bounds.Height, 0, placement.Height);
            if (right <= left || bottom <= top)
                continue;

            var outline = new Rectangle
            {
                Width = (right - left) * zoom, Height = (bottom - top) * zoom,
                Stroke = brush, StrokeThickness = 2,
            };
            Canvas.SetLeft(outline, (placement.Left + left) * zoom);
            Canvas.SetTop(outline, (placement.Top + top) * zoom);
            SearchHighlightCanvas.Children.Add(outline);
        }
    }
}
