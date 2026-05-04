using System;
using System.Runtime.InteropServices;
using System.Text;

namespace Rhwp;

public static class RhwpNative
{
    public const int AllPages = -1;

    private const string NativeLibraryName = "rhwp_native_ffi";

    public static string ExportText(string inputPath, string outputDirectory, int page = AllPages)
    {
        IntPtr result = rhwp_export_text(ToUtf8NullTerminated(inputPath), ToUtf8NullTerminated(outputDirectory), page);
        return TakeResultString(result);
    }

    public static string ExportMarkdown(string inputPath, string outputDirectory, int page = AllPages)
    {
        IntPtr result = rhwp_export_markdown(ToUtf8NullTerminated(inputPath), ToUtf8NullTerminated(outputDirectory), page);
        return TakeResultString(result);
    }

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr rhwp_export_text(byte[] inputPath, byte[] outputDirectory, int page);

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr rhwp_export_markdown(byte[] inputPath, byte[] outputDirectory, int page);

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void rhwp_string_free(IntPtr value);

    private static byte[] ToUtf8NullTerminated(string value)
    {
        if (value is null)
        {
            throw new ArgumentNullException(nameof(value));
        }

        byte[] utf8 = Encoding.UTF8.GetBytes(value);
        Array.Resize(ref utf8, utf8.Length + 1);
        return utf8;
    }

    private static string TakeResultString(IntPtr result)
    {
        if (result == IntPtr.Zero)
        {
            throw new InvalidOperationException("Native rhwp call returned a null result pointer.");
        }

        try
        {
            return Marshal.PtrToStringUTF8(result)
                ?? throw new InvalidOperationException("Native rhwp call returned invalid UTF-8.");
        }
        finally
        {
            rhwp_string_free(result);
        }
    }
}
