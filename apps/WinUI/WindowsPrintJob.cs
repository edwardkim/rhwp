using System.ComponentModel;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace Rhwp.WinUI;

internal sealed class WindowsPrintJob : IDisposable
{
    private const uint PageNumbers = 0x00000002;
    private const uint NoSelection = 0x00000004;
    private const uint ReturnDeviceContext = 0x00000100;
    private const uint UseDriverCopiesAndCollation = 0x00040000;
    private const uint CurrentPage = 0x00400000;
    private const uint HidePrintToFile = 0x00100000;
    private const uint ResultPrint = 1;
    private const int MaxPageRanges = 32;

    private const int HorzRes = 8;
    private const int VertRes = 10;
    private const int LogPixelsX = 88;
    private const int LogPixelsY = 90;
    private const int PhysicalWidth = 110;
    private const int PhysicalHeight = 111;
    private const int PhysicalOffsetX = 112;
    private const int PhysicalOffsetY = 113;
    private const int Halftone = 4;
    private const uint SrcCopy = 0x00CC0020;
    private const int ErrorCancelled = 1223;
    private const int SpUserAbort = -3;
    private const double PageUnitsPerInch = 96;
    private const double MaxRasterDpi = 300;

    private readonly PrinterDevice device;
    private readonly PageRange[] ranges;

    private WindowsPrintJob(PrinterDevice device, PageRange[] ranges)
    {
        this.device = device;
        this.ranges = ranges;
    }

    public static WindowsPrintJob? ShowDialog(nint owner, PageInfo current)
    {
        var rangeBuffer = new PageRange[MaxPageRanges];
        GCHandle pinned = GCHandle.Alloc(rangeBuffer, GCHandleType.Pinned);
        var dialog = new PrintDialog
        {
            Size = (uint)Marshal.SizeOf<PrintDialog>(),
            Owner = owner,
            Flags = ReturnDeviceContext | UseDriverCopiesAndCollation | NoSelection | HidePrintToFile,
            MaxPageRanges = MaxPageRanges,
            PageRanges = pinned.AddrOfPinnedObject(),
            MinPage = 1,
            MaxPage = current.PageCount,
            Copies = 1,
            StartPage = uint.MaxValue, // START_PAGE_GENERAL
        };
        try
        {
            int result = PrintDlgExW(ref dialog);
            if (result < 0)
                throw new ExternalException($"Windows 인쇄 창을 열 수 없습니다. (0x{result:X8})", result);
            if (dialog.ResultAction != ResultPrint)
                return null;
            if (dialog.DeviceContext == 0)
                throw new InvalidOperationException("선택한 프린터를 사용할 수 없습니다.");

            PageRange[] selected = ReadPageRanges(dialog, rangeBuffer, current);
            var device = new PrinterDevice(dialog.DeviceContext);
            dialog.DeviceContext = 0;
            return new WindowsPrintJob(device, selected);
        }
        finally
        {
            pinned.Free();
            if (dialog.DeviceMode != 0)
                GlobalFree(dialog.DeviceMode);
            if (dialog.DeviceNames != 0)
                GlobalFree(dialog.DeviceNames);
            if (dialog.DeviceContext != 0)
                DeleteDC(dialog.DeviceContext);
        }
    }

    private static PageRange[] ReadPageRanges(PrintDialog dialog, PageRange[] rangeBuffer, PageInfo current)
    {
        if ((dialog.Flags & CurrentPage) != 0)
            return [new(current.Page + 1, current.Page + 1)];
        if ((dialog.Flags & PageNumbers) == 0)
            return [new(1, current.PageCount)];
        if (dialog.PageRangeCount is 0 or > MaxPageRanges)
            throw new InvalidDataException("선택한 인쇄 범위가 올바르지 않습니다.");

        PageRange[] selected = rangeBuffer[..(int)dialog.PageRangeCount];
        if (selected.Any(range => range.From == 0 || range.To < range.From || range.To > current.PageCount))
            throw new InvalidDataException("선택한 인쇄 범위가 올바르지 않습니다.");
        return selected.OrderBy(range => range.From).ToArray();
    }

    public void Print(NativeWorkerDispatcher worker, PageInfo current, string title, CancellationToken token)
    {
        // PrintDlgEx transferred the context. Spool on one background thread and keep
        // the context alive until the job ends or AbortDoc has returned.
        token.ThrowIfCancellationRequested();
        int dpiX = GetDeviceCaps(device, LogPixelsX);
        int dpiY = GetDeviceCaps(device, LogPixelsY);
        int printableWidth = GetDeviceCaps(device, HorzRes);
        int printableHeight = GetDeviceCaps(device, VertRes);
        if (dpiX <= 0 || dpiY <= 0 || printableWidth <= 0 || printableHeight <= 0)
            throw new InvalidOperationException("프린터에서 인쇄 가능 영역을 확인할 수 없습니다.");

        int physicalWidth = GetDeviceCaps(device, PhysicalWidth);
        int physicalHeight = GetDeviceCaps(device, PhysicalHeight);
        int offsetX = GetDeviceCaps(device, PhysicalOffsetX);
        int offsetY = GetDeviceCaps(device, PhysicalOffsetY);
        var doc = new DocInfo { Size = Marshal.SizeOf<DocInfo>(), Name = title };
        AbortProcedure shouldContinue = (_, _) => token.IsCancellationRequested ? 0 : 1;
        bool started = false;
        try
        {
            Check(SetAbortProc(device, shouldContinue), "인쇄 취소 처리 설정", token);
            Check(StartDocW(device, ref doc), "인쇄 시작", token);
            started = true;
            ulong nextPage = 1;
            foreach (PageRange range in ranges)
            {
                // Merge overlaps while streaming; even a large range holds only one frame.
                for (ulong number = Math.Max(nextPage, range.From); number <= range.To; number++)
                {
                    token.ThrowIfCancellationRequested();
                    PageInfo page = worker.GetPageAsync(current, (uint)(number - 1), token).GetAwaiter().GetResult();
                    double fit = Math.Min(1, Math.Min(printableWidth * PageUnitsPerInch / dpiX / page.Width,
                        printableHeight * PageUnitsPerInch / dpiY / page.Height));
                    int width = Math.Clamp((int)Math.Round(page.Width * fit * dpiX / PageUnitsPerInch), 1, printableWidth);
                    int height = Math.Clamp((int)Math.Round(page.Height * fit * dpiY / PageUnitsPerInch), 1, printableHeight);
                    int x = Math.Clamp((Math.Max(physicalWidth, printableWidth) - width) / 2 - offsetX,
                        0, printableWidth - width);
                    int y = Math.Clamp((Math.Max(physicalHeight, printableHeight) - height) / 2 - offsetY,
                        0, printableHeight - height);

                    double scale = Math.Min(Math.Min(MaxRasterDpi, Math.Max(dpiX, dpiY) * fit) / PageUnitsPerInch,
                        (PageViewport.MaxRasterDimension - 2) / Math.Max(page.Width, page.Height));
                    var viewport = new PageViewport(new RenderRegion(0, 0, page.Width, page.Height), scale, 1);
                    PageFrame frame = worker.RenderAsync(page, viewport, token).GetAwaiter().GetResult();
                    SpoolPage(frame, x, y, width, height, token);
                }
                nextPage = Math.Max(nextPage, (ulong)range.To + 1);
            }

            token.ThrowIfCancellationRequested();
            Check(EndDoc(device), "인쇄 완료", token);
            started = false;
        }
        finally
        {
            if (started)
                AbortDoc(device);
            GC.KeepAlive(shouldContinue);
        }
    }

    private void SpoolPage(PageFrame frame, int x, int y, int width, int height, CancellationToken token)
    {
        // The worker returns top-down premultiplied BGRA. BI_RGB ignores alpha,
        // so composite any transparent pixels onto the white paper first.
        byte[] pixels = frame.Bgra;
        for (int i = 0; i < pixels.Length; i += 4)
        {
            int white = 255 - pixels[i + 3];
            if (white == 0)
                continue;
            pixels[i] = (byte)(pixels[i] + white);
            pixels[i + 1] = (byte)(pixels[i + 1] + white);
            pixels[i + 2] = (byte)(pixels[i + 2] + white);
            pixels[i + 3] = 255;
        }

        var bitmap = new BitmapInfoHeader
        {
            Size = (uint)Marshal.SizeOf<BitmapInfoHeader>(),
            Width = frame.PixelWidth,
            Height = -frame.PixelHeight,
            Planes = 1,
            BitCount = 32,
        };
        token.ThrowIfCancellationRequested();
        Check(StartPage(device), "인쇄 쪽 시작", token);
        if (SetStretchBltMode(device, Halftone) != 0)
            SetBrushOrgEx(device, 0, 0, 0);
        Check(StretchDIBits(device, x, y, width, height, 0, 0, frame.PixelWidth, frame.PixelHeight,
            pixels, ref bitmap, 0, SrcCopy), "인쇄 쪽 그리기", token);
        Check(EndPage(device), "인쇄 쪽 완료", token);
    }

    private static void Check(int result, string operation, CancellationToken token)
    {
        if (result > 0)
            return;
        int error = Marshal.GetLastPInvokeError();
        token.ThrowIfCancellationRequested();
        if (error == ErrorCancelled || result == SpUserAbort)
            throw new OperationCanceledException();
        throw new Win32Exception(error, $"{operation} 단계에서 Windows 인쇄 오류가 발생했습니다. ({result}, {error})");
    }

    public void Dispose() => device.Dispose();

    [StructLayout(LayoutKind.Sequential)]
    private readonly record struct PageRange(uint From, uint To);

    private sealed class PrinterDevice : SafeHandleZeroOrMinusOneIsInvalid
    {
        public PrinterDevice(nint handle) : base(ownsHandle: true) => SetHandle(handle);

        protected override bool ReleaseHandle() => DeleteDC(handle);
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct PrintDialog
    {
        public uint Size;
        public nint Owner;
        public nint DeviceMode;
        public nint DeviceNames;
        public nint DeviceContext;
        public uint Flags;
        public uint Flags2;
        public uint ExclusionFlags;
        public uint PageRangeCount;
        public uint MaxPageRanges;
        public nint PageRanges;
        public uint MinPage;
        public uint MaxPage;
        public uint Copies;
        public nint Instance;
        public nint PrintTemplateName;
        public nint Callback;
        public uint PropertyPageCount;
        public nint PropertyPages;
        public uint StartPage;
        public uint ResultAction;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct DocInfo
    {
        public int Size;
        public string Name;
        public nint Output;
        public nint DataType;
        public uint Type;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct BitmapInfoHeader
    {
        public uint Size;
        public int Width;
        public int Height;
        public ushort Planes;
        public ushort BitCount;
        public uint Compression;
        public uint ImageSize;
        public int XPixelsPerMeter;
        public int YPixelsPerMeter;
        public uint ColorsUsed;
        public uint ImportantColors;
    }

    [UnmanagedFunctionPointer(CallingConvention.Winapi)]
    private delegate int AbortProcedure(nint device, int error);

    [DllImport("comdlg32.dll", ExactSpelling = true)]
    private static extern int PrintDlgExW(ref PrintDialog dialog);

    [DllImport("kernel32.dll", ExactSpelling = true)]
    private static extern nint GlobalFree(nint handle);

    [DllImport("gdi32.dll", ExactSpelling = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DeleteDC(nint device);

    [DllImport("gdi32.dll", ExactSpelling = true)]
    private static extern int GetDeviceCaps(PrinterDevice device, int index);

    [DllImport("gdi32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern int SetAbortProc(PrinterDevice device, AbortProcedure procedure);

    [DllImport("gdi32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern int StartDocW(PrinterDevice device, ref DocInfo info);

    [DllImport("gdi32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern int StartPage(PrinterDevice device);

    [DllImport("gdi32.dll", ExactSpelling = true)]
    private static extern int SetStretchBltMode(PrinterDevice device, int mode);

    [DllImport("gdi32.dll", ExactSpelling = true)]
    private static extern int SetBrushOrgEx(PrinterDevice device, int x, int y, nint previous);

    [DllImport("gdi32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern int StretchDIBits(PrinterDevice device, int x, int y, int width, int height,
        int sourceX, int sourceY, int sourceWidth, int sourceHeight, [In] byte[] pixels,
        ref BitmapInfoHeader bitmap, uint usage, uint operation);

    [DllImport("gdi32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern int EndPage(PrinterDevice device);

    [DllImport("gdi32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern int EndDoc(PrinterDevice device);

    [DllImport("gdi32.dll", ExactSpelling = true)]
    private static extern int AbortDoc(PrinterDevice device);
}
