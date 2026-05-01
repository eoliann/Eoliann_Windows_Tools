$ErrorActionPreference = "Stop"

function Test-VcRuntimeInstalled {
    $paths = @(
        "HKLM:\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64",
        "HKLM:\SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x64"
    )

    foreach ($path in $paths) {
        if (Test-Path $path) {
            $props = Get-ItemProperty -Path $path -ErrorAction SilentlyContinue

            if ($null -ne $props) {
                if ($props.Installed -eq 1) {
                    return $true
                }

                if ($null -ne $props.Version) {
                    return $true
                }
            }
        }
    }

    $dllPath = Join-Path $env:WINDIR "System32\VCRUNTIME140.dll"

    if (Test-Path $dllPath) {
        return $true
    }

    return $false
}

if (Test-VcRuntimeInstalled) {
    exit 0
}

$url = "https://aka.ms/vc14/vc_redist.x64.exe"
$out = Join-Path $env:TEMP "vc_redist.x64.exe"

if (Test-Path $out) {
    Remove-Item $out -Force
}

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

Invoke-WebRequest -Uri $url -OutFile $out -UseBasicParsing

$sig = Get-AuthenticodeSignature -FilePath $out

if ($sig.Status -ne "Valid") {
    throw "The downloaded VC++ Redistributable is not signed correctly."
}

if ($null -eq $sig.SignerCertificate) {
    throw "The downloaded VC++ Redistributable has no signer certificate."
}

if ($sig.SignerCertificate.Subject -notmatch "Microsoft") {
    throw "The downloaded VC++ Redistributable is not signed by Microsoft."
}

$p = Start-Process `
    -FilePath $out `
    -ArgumentList "/install", "/passive", "/norestart" `
    -Wait `
    -PassThru

if ($p.ExitCode -notin @(0, 3010, 1638)) {
    throw "VC++ Redistributable installer failed. Exit code: $($p.ExitCode)"
}

exit 0