param(
    [switch]$Run
    )

cl /TC .\src\wsr.c /I .\src\ /W4 /Z7 /nologo /Fo:.\obj\wsr.obj /Fe:.\bin\wsr.exe

if (!$?)
{
    Write-Warning "Failed to compile!"
    return;
}

if ($Run)
{
    Write-Host -ForegroundColor Cyan "`nRunning:"
    .\bin\wsr.exe ".\test_data\Recording_20200813_1012.mht" "$env:TEMP"
}