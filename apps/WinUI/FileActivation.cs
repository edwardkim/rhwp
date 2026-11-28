namespace Rhwp.WinUI;

internal static class FileActivation
{
    public static string? FirstDocumentPath(IEnumerable<string> paths) =>
        paths.FirstOrDefault(path =>
        {
            string? extension = Path.GetExtension(path);
            return string.Equals(extension, ".hwp", StringComparison.OrdinalIgnoreCase) ||
                   string.Equals(extension, ".hwpx", StringComparison.OrdinalIgnoreCase);
        });
}
