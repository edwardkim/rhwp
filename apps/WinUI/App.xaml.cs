using Microsoft.UI.Xaml;
using Microsoft.Windows.AppLifecycle;
using Windows.ApplicationModel.Activation;
using Windows.Storage;

namespace Rhwp.WinUI;

public partial class App : Application
{
    private MainWindow? window;

    public App()
    {
        InitializeComponent();
    }

    protected override async void OnLaunched(Microsoft.UI.Xaml.LaunchActivatedEventArgs args)
    {
        MainWindow current = window ??= new MainWindow();
        current.Activate();
        await current.WaitForWindowLoadedAsync();
        if (await current.RestoreRecoveryAtStartupAsync())
            return;

        var activation = AppInstance.GetCurrent().GetActivatedEventArgs();
        string? path = activation?.Data is IFileActivatedEventArgs files
            ? FileActivation.FirstDocumentPath(files.Files.OfType<StorageFile>().Select(file => file.Path))
            : FileActivation.FirstDocumentPath(Environment.GetCommandLineArgs().Skip(1));
        if (path is not null)
            await current.OpenFileAsync(path);
    }
}
