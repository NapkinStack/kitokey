# D77: the Windows App Certification Kit tests the package before the Store does. Its report is
# written beside the package, and each failed test is named in the run (appcert.py). D102: the run
# fails on a required test that did not pass, and on a kit that did not run — said in one line, never
# skipped silently; an optional test failed is named and blocks nothing.
param([Parameter(Mandatory)][string]$Out)
$ErrorActionPreference = 'Stop'

$kit = 'C:\Program Files (x86)\Windows Kits\10\App Certification Kit\appcert.exe'
if (-not (Test-Path $kit)) {
    Write-Output "certification kit: not run — appcert.exe is not on this machine ($kit)"
    exit 2
}
$package = (Get-ChildItem (Join-Path $Out 'Kitokey-*-x64.msix') | Select-Object -First 1).FullName
$report = Join-Path (Resolve-Path $Out) 'appcert-report.xml'
# A report of an earlier run is never read as this one's (D102).
Remove-Item $report -ErrorAction SilentlyContinue
# The kit's return codes, as `appcert.exe /?` lists them.
$meaning = @{ -1 = 'invalid command line'; -2 = 'infrastructure error'; -3 = 'user initiated error'
              -4 = 'app installation error'; -5 = 'app unpackaging error' }

$certificate = Import-Certificate -FilePath (Join-Path $Out 'Kitokey-this-run.cer') `
    -CertStoreLocation 'Cert:\LocalMachine\TrustedPeople'
try {
    & $kit reset | Out-Host
    & $kit test -appxpackagepath $package -reportoutputpath $report | Out-Host
    $code = $LASTEXITCODE
} finally {
    Remove-Item "Cert:\LocalMachine\TrustedPeople\$($certificate.Thumbprint)" -ErrorAction SilentlyContinue
}
if ($code -notin 0, 1 -and -not (Test-Path $report)) {
    Write-Output "certification kit: not run — appcert.exe test ended with $code ($($meaning[$code]))"
    exit 2
}
& python (Join-Path $PSScriptRoot 'appcert.py') $report
exit $LASTEXITCODE
