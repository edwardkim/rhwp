namespace Rhwp.WinUI;

internal static class FileActivation
{
    private static readonly string[] pictureExtensions = [".png", ".jpg", ".jpeg"];
    public static IReadOnlyList<string> PictureExtensions => pictureExtensions;

    public static string? FirstDocumentPath(IEnumerable<string> paths) =>
        paths.FirstOrDefault(path =>
        {
            string? extension = Path.GetExtension(path);
            return string.Equals(extension, ".hwp", StringComparison.OrdinalIgnoreCase) ||
                   string.Equals(extension, ".hwpx", StringComparison.OrdinalIgnoreCase);
        });

    public static string? FirstPicturePath(IEnumerable<string> paths) =>
        paths.FirstOrDefault(path => pictureExtensions.Any(extension =>
            string.Equals(Path.GetExtension(path), extension, StringComparison.OrdinalIgnoreCase)));
}
