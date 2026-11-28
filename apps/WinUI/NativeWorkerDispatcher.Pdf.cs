using System.Text.Json;

namespace Rhwp.WinUI;

internal sealed partial class NativeWorkerDispatcher
{
    public Task<LivePdfLimits?> GetLivePdfLimitsAsync(CancellationToken cancellation) =>
        Dispatch(_ => ReadLivePdfLimits(), cancellation);

    public Task<byte[]> ExportPagePdfAsync(PageInfo page, int maxOutputBytes, CancellationToken cancellation) =>
        Dispatch(token => ExportPagePdf(page, maxOutputBytes, token), cancellation);

    private static LivePdfLimits? ReadLivePdfLimits()
    {
        using var response = ReadSuccess(RhwpNative.SessionCapabilities());
        return LivePdfContract.Limits(response.RootElement);
    }

    private byte[] ExportPagePdf(PageInfo page, int maxOutputBytes, CancellationToken cancellation)
    {
        LivePdfLimits limits = ReadLivePdfLimits() ??
            throw new NotSupportedException("이 Native 라이브러리는 열린 문서의 PDF 출력을 지원하지 않습니다.");
        if (maxOutputBytes <= 0 || maxOutputBytes > limits.MaxOutputBytes)
            throw new ArgumentOutOfRangeException(nameof(maxOutputBytes));
        if (page.Page >= page.PageCount)
            throw new ArgumentOutOfRangeException(nameof(page));

        JsonElement receipt = Submit(EnsureWorker(), new
        {
            op = "exportPdf",
            target = page.Target,
            page.FontGeneration,
            firstPage = page.Page,
            pageCount = 1,
            maxOutputBytes,
        }, cancellation);

        ulong buffer = receipt.GetProperty("buffer").GetUInt64();
        try
        {
            cancellation.ThrowIfCancellationRequested();
            int length = LivePdfContract.ValidateReceipt(receipt, page, maxOutputBytes);
            byte[] bytes = CopyExactBuffer(buffer, length);
            LivePdfContract.ValidateBytes(bytes);
            cancellation.ThrowIfCancellationRequested();
            return bytes;
        }
        finally
        {
            ReleaseBuffer(buffer);
        }
    }
}
