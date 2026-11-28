using System.Diagnostics;

namespace Rhwp.WinUI;

public sealed partial class MainWindow
{
    private (DocumentTarget Target, ulong FontGeneration)? automationDocument;
    private CancellationTokenSource? automationRequest;

    private void UpdateAutomationDocument(PageInfo? current)
    {
        (DocumentTarget Target, ulong FontGeneration)? identity = current is null
            ? null : (current.Target, current.FontGeneration);
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
                DocumentAutomationSnapshot? snapshot = await worker.GetAutomationDocumentAsync(current, request.Token);
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
