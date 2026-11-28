using System.Runtime.InteropServices;
using System.Text;
using Windows.Foundation;

namespace Rhwp.WinUI;

// WinUI handles projected StorageItems. Some shell clients deliver the classic
// HDROP message to the HWND, so route that format through the same file handler.
internal sealed class WindowsFileDrop : IDisposable
{
    private const uint DropFilesMessage = 0x0233;
    private const uint DestroyMessage = 0x0082;
    private const nuint SubclassId = 1;
    private readonly SubclassProcedure subclass;
    private readonly Action<IReadOnlyList<string>, Point?> receive;
    private nint window;

    public WindowsFileDrop(nint window, Action<IReadOnlyList<string>, Point?> receive)
    {
        this.receive = receive;
        subclass = OnWindowMessage;
        if (!SetWindowSubclass(window, subclass, SubclassId, 0))
            throw new InvalidOperationException("Windows 파일 드롭을 연결할 수 없습니다.");

        this.window = window;
        DragAcceptFiles(window, true);
    }

    private nint OnWindowMessage(nint hwnd, uint message, nuint wParam, nint lParam, nuint id, nuint data)
    {
        if (message == DropFilesMessage)
        {
            nint drop = (nint)wParam;
            try
            {
                uint count = DragQueryFileW(drop, uint.MaxValue, null, 0);
                var paths = new List<string>();
                for (uint index = 0; index < count; index++)
                {
                    uint length = DragQueryFileW(drop, index, null, 0);
                    if (length == 0)
                        continue;
                    var path = new StringBuilder(checked((int)length + 1));
                    if (DragQueryFileW(drop, index, path, (uint)path.Capacity) != 0)
                        paths.Add(path.ToString());
                }

                Point? point = DragQueryPoint(drop, out NativePoint position) ? new Point(position.X, position.Y) : null;
                receive(paths, point);
            }
            catch (Exception error)
            {
                System.Diagnostics.Debug.WriteLine(error);
            }
            finally
            {
                DragFinish(drop);
            }
            return 0;
        }

        if (message == DestroyMessage)
            Dispose();
        return DefSubclassProc(hwnd, message, wParam, lParam);
    }

    public void Dispose()
    {
        if (window == 0)
            return;

        DragAcceptFiles(window, false);
        RemoveWindowSubclass(window, subclass, SubclassId);
        window = 0;
    }

    [StructLayout(LayoutKind.Sequential)]
    private readonly struct NativePoint
    {
        public readonly int X;
        public readonly int Y;
    }

    private delegate nint SubclassProcedure(nint hwnd, uint message, nuint wParam, nint lParam, nuint id, nuint data);

    [DllImport("comctl32.dll", ExactSpelling = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetWindowSubclass(nint hwnd, SubclassProcedure procedure, nuint id, nuint data);

    [DllImport("comctl32.dll", ExactSpelling = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool RemoveWindowSubclass(nint hwnd, SubclassProcedure procedure, nuint id);

    [DllImport("comctl32.dll", ExactSpelling = true)]
    private static extern nint DefSubclassProc(nint hwnd, uint message, nuint wParam, nint lParam);

    [DllImport("shell32.dll", ExactSpelling = true)]
    private static extern void DragAcceptFiles(nint hwnd, [MarshalAs(UnmanagedType.Bool)] bool enabled);

    [DllImport("shell32.dll", CharSet = CharSet.Unicode, ExactSpelling = true)]
    private static extern uint DragQueryFileW(nint drop, uint index, StringBuilder? buffer, uint length);

    [DllImport("shell32.dll", ExactSpelling = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DragQueryPoint(nint drop, out NativePoint point);

    [DllImport("shell32.dll", ExactSpelling = true)]
    private static extern void DragFinish(nint drop);
}
