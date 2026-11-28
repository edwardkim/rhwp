using System.Diagnostics;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private (DocumentTarget Target, ulong FontGeneration, uint Page)? automationDocument;
    private CancellationTokenSource? automationRequest;

    private void UpdateAutomationDocument(PageInfo? current)
    {
        PageCanvas.UpdateAutomationViewport(PageScroll, zoom);
        (DocumentTarget Target, ulong FontGeneration, uint Page)? identity = current is null
            ? null : (current.Target, current.FontGeneration, current.Page);
        if (identity == automationDocument)
            return;

        automationRequest?.Cancel();
        automationRequest = null;
        automationDocument = identity;
        PageCanvas.SetAutomationDocument(null);

        if (current is not null)
        {
            var request = new CancellationTokenSource();
            automationRequest = request;
            _ = LoadAutomationDocumentAsync(current, request);
        }
    }

    private async Task LoadAutomationDocumentAsync(PageInfo current, CancellationTokenSource request)
    {
        using (request)
        {
            try
            {
                DocumentAutomationSnapshot? snapshot = await worker.GetAutomationPageAsync(current, request.Token);
                if (!closing && !request.IsCancellationRequested && ReferenceEquals(automationRequest, request))
                    PageCanvas.SetAutomationDocument(snapshot);
            }
            catch (OperationCanceledException) when (closing || request.IsCancellationRequested)
            {
            }
            catch (Exception error)
            {
                if (!closing && ReferenceEquals(automationRequest, request))
                {
                    automationDocument = null;
                    Debug.WriteLine(error);
                }
            }
            finally
            {
                if (ReferenceEquals(automationRequest, request))
                    automationRequest = null;
            }
        }
    }
}
