[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $AppPath,
    [Parameter(Mandatory)]
    [string] $Hwp,
    [Parameter(Mandatory)]
    [string] $Hwpx,
    [ValidateRange(15, 120)]
    [int] $TimeoutSeconds = 45
)

$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSEdition -ne 'Desktop' -or -not [Environment]::UserInteractive) {
    throw 'Run this script in Windows PowerShell on an interactive Windows desktop.'
}

Add-Type -AssemblyName 'UIAutomationTypes'
Add-Type -AssemblyName 'UIAutomationClient'
$exe = (Resolve-Path -LiteralPath $AppPath).ProviderPath
$workingDirectory = Split-Path -LiteralPath $exe
$documents = @(
    @{ Path = (Resolve-Path -LiteralPath $Hwp).ProviderPath; Extension = '.hwp' },
    @{ Path = (Resolve-Path -LiteralPath $Hwpx).ProviderPath; Extension = '.hwpx' }
)
$descendants = [System.Windows.Automation.TreeScope]::Descendants

function Find-AutomationId($parent, [string] $id) {
    $property = [System.Windows.Automation.AutomationElement]::AutomationIdProperty
    $condition = [System.Windows.Automation.PropertyCondition]::new($property, $id)
    return $parent.FindFirst($descendants, $condition)
}

foreach ($document in $documents) {
    if ([IO.Path]::GetExtension($document.Path) -ne $document.Extension) {
        throw "Unexpected fixture extension: $($document.Path)"
    }

    $fileName = [IO.Path]::GetFileName($document.Path)
    $argument = '"' + $document.Path + '"'
    $app = Start-Process -FilePath $exe -ArgumentList $argument -WorkingDirectory $workingDirectory -PassThru
    try {
        $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
        $ready = $false
        $observedStatus = ''
        while ([DateTime]::UtcNow -lt $deadline) {
            Start-Sleep -Milliseconds 250
            $app.Refresh()
            if ($app.HasExited) {
                throw "WinUI exited with code $($app.ExitCode) while opening $fileName"
            }
            if ($app.MainWindowHandle -eq [IntPtr]::Zero -or $app.MainWindowTitle.IndexOf(
                    $fileName, [StringComparison]::OrdinalIgnoreCase) -lt 0) {
                continue
            }

            try {
                $window = [System.Windows.Automation.AutomationElement]::FromHandle($app.MainWindowHandle)
                $position = Find-AutomationId $window 'PagePosition'
                $canvas = Find-AutomationId $window 'DocumentCanvas'
                $status = Find-AutomationId $window 'DocumentStatus'
                $observedStatus = if ($status) { $status.Current.Name } else { '' }
                if (-not $position -or -not $canvas -or $position.Current.Name -notmatch '^1\s*/\s*[1-9][0-9]*' -or
                    -not [string]::IsNullOrEmpty($observedStatus)) {
                    continue
                }

                $controlType = [System.Windows.Automation.AutomationElement]::ControlTypeProperty
                $text = [System.Windows.Automation.PropertyCondition]::new(
                    $controlType, [System.Windows.Automation.ControlType]::Text)
                $ready = $null -ne $canvas.FindFirst($descendants, $text)
                if ($ready) { break }
            }
            catch [System.Windows.Automation.ElementNotAvailableException] {
                # The page and its accessibility tree can be replaced while loading.
            }
        }
        if (-not $ready) {
            throw "First-page rendering or document text was unavailable for $fileName. Status: $observedStatus"
        }
        Write-Output "Opened and rendered $fileName; first-page text exposed to UI Automation."
    }
    finally {
        if (-not $app.HasExited) {
            # Only this script's fixture process is targeted, even if it failed to open.
            $null = $app.CloseMainWindow()
            if (-not $app.WaitForExit(5000)) { $app.Kill() }
        }
        $app.Dispose()
    }
}
