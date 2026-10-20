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

    /// <summary>
    /// Print-profile direct PDF export to a new file. Requires native-skia.
    /// The native JSON result reports featureDisabled when unavailable.
    /// </summary>
    public static string ExportPdfDirect(string inputPath, string outputFile, int page = AllPages)
    {
        IntPtr result = rhwp_export_pdf_direct(ToUtf8NullTerminated(inputPath), ToUtf8NullTerminated(outputFile), page);
        return TakeResultString(result);
    }

    /// <summary>
    /// 파일로 내보내지 않고 페이지 텍스트를 JSON 문자열로 돌려준다.
    /// </summary>
    /// <remarks>
    /// [#3891] 이 바인딩은 <c>rhwp_read_text</c> 가 C ABI 에 추가된 뒤에도 반영되지
    /// 않아 Swift 에만 있던 기능이었다. C 표면 계약 가드가 그 표류를 검출해 채웠다.
    /// </remarks>
    public static string ReadText(string inputPath, int page = AllPages)
    {
        IntPtr result = rhwp_read_text(ToUtf8NullTerminated(inputPath), page);
        return TakeResultString(result);
    }

    // Session and buffer handles must be used and released on their opening thread.
    // Each method returns the versioned JSON envelope, including structured errors.
    public static string SessionCapabilities() => TakeResultString(rhwp_session_capabilities());

    public static string SessionOpen(byte[] data, byte[]? password = null)
    {
        if (data is null) throw new ArgumentNullException(nameof(data));
        password ??= Array.Empty<byte>();
        return TakeResultString(rhwp_session_open(data, (UIntPtr)data.Length, password, (UIntPtr)password.Length));
    }

    public static string SessionRequest(ulong handle, string requestJson)
    {
        if (requestJson is null) throw new ArgumentNullException(nameof(requestJson));
        byte[] request = Encoding.UTF8.GetBytes(requestJson);
        return TakeResultString(rhwp_session_request(handle, request, (UIntPtr)request.Length));
    }

    public static string HostRequest(string requestJson, byte[]? data = null)
    {
        if (requestJson is null) throw new ArgumentNullException(nameof(requestJson));
        byte[] request = Encoding.UTF8.GetBytes(requestJson);
        data ??= Array.Empty<byte>();
        return TakeResultString(rhwp_native_host_request(request, (UIntPtr)request.Length, data, (UIntPtr)data.Length));
    }

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr rhwp_native_host_request(byte[] request, UIntPtr requestLength, byte[] data, UIntPtr dataLength);

    public static string SessionClose(ulong handle) => TakeResultString(rhwp_session_close(handle));

    public static string SessionBufferCopy(ulong buffer, byte[] destination)
    {
        if (destination is null) throw new ArgumentNullException(nameof(destination));
        return TakeResultString(rhwp_session_buffer_copy(buffer, destination, (UIntPtr)destination.Length));
    }

    public static string SessionBufferRelease(ulong buffer) => TakeResultString(rhwp_session_buffer_release(buffer));

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr rhwp_session_capabilities();

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr rhwp_session_open(byte[] data, UIntPtr length, byte[] password, UIntPtr passwordLength);

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr rhwp_session_request(ulong handle, byte[] request, UIntPtr requestLength);

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr rhwp_session_close(ulong handle);

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr rhwp_session_buffer_copy(ulong buffer, [Out] byte[] destination, UIntPtr capacity);

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr rhwp_session_buffer_release(ulong buffer);

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr rhwp_export_text(byte[] inputPath, byte[] outputDirectory, int page);

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr rhwp_export_markdown(byte[] inputPath, byte[] outputDirectory, int page);

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr rhwp_export_pdf_direct(byte[] inputPath, byte[] outputFile, int page);

    [DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr rhwp_read_text(byte[] inputPath, int page);

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
