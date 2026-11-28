using Microsoft.UI.Xaml;
using Microsoft.Windows.AppLifecycle;
using Windows.ApplicationModel.Activation;
using Windows.Storage;

namespace Rhwp.WinUI;

public partial class App : Application
{
    private StudioWindow? window;

    public App()
    {
        InitializeComponent();
    }

    protected override async void OnLaunched(Microsoft.UI.Xaml.LaunchActivatedEventArgs args)
    {
        StudioWindow current = window ??= new StudioWindow();
        current.Activate();

        var activation = AppInstance.GetCurrent().GetActivatedEventArgs();
        string? path = activation?.Data is IFileActivatedEventArgs files
            ? StudioWindow.FindStartupDocument(files.Files.OfType<StorageFile>().Select(file => file.Path))
            : StudioWindow.FindStartupDocument(Environment.GetCommandLineArgs().Skip(1));
        if (path is not null)
            await current.OpenStartupDocumentAsync(path);
    }
}
