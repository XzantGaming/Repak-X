<#
Usage:
  1) Open Powershell in repo root.
  2) .\scripts\color-mapper.ps1        -> generates detected-colors.json and mappings.json (with suggestions)
  3) Edit mappings.json and set replacement value strings (e.g. "var(--accent-primary)").
  4) .\scripts\color-mapper.ps1 -Apply -> applies mappings to CSS/SCSS files (creates .bak backups).
#>

param(
  [switch]$Apply
)

$repo = Resolve-Path .
$skipPaths = @('node_modules','dist','.git')
$themeFile = Join-Path $repo 'repak-gui\src\styles\theme.css'
$outColors = Join-Path $repo 'repak-gui\detected-colors.json'
$outMap = Join-Path $repo 'mappings.json'

Write-Host "Scanning CSS/SCSS files for hex colors..."

$files = Get-ChildItem -Path (Join-Path $repo 'repak-gui') -Recurse -Include *.css,*.scss -File |
  Where-Object { ($_.FullName -notmatch [string]::Join('|',$skipPaths)) -and ($_.FullName -notmatch 'src\\styles\\theme.css$') }

$colorSet = [System.Collections.Generic.HashSet[string]]::new()

$regex = '#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})\b'

foreach ($f in $files) {
  $text = Get-Content $f.FullName -Raw
  [regex]::Matches($text, $regex) | ForEach-Object {
    $colorSet.Add($_.Value.ToLower()) | Out-Null
  }
}

$colors = $colorSet | Sort-Object
$colors | ConvertTo-Json | Out-File -Encoding UTF8 $outColors

Write-Host "Found $($colors.Count) unique colors. Written to $outColors"

# If mappings.json doesn't exist, create a starter with common suggestions
if (-not (Test-Path $outMap)) {
  $suggest = @{}
  foreach ($c in $colors) {
    switch ($c) {
      '#141414' { $suggest[$c] = 'var(--bg-darkest)'; continue }
      '#1a1a1a' { $suggest[$c] = 'var(--bg-darker)'; continue }
      '#202020' { $suggest[$c] = 'var(--bg-dark)'; continue }
      '#2a2a2a' { $suggest[$c] = 'var(--bg-light)'; continue }
      '#323232' { $suggest[$c] = 'var(--bg-lighter)'; continue }
      '#e6e6e6' { $suggest[$c] = 'var(--bg-light)'; continue }
      '#e0e0e0' { $suggest[$c] = 'var(--text-primary)'; continue }
      '#a7a7a7' { $suggest[$c] = 'var(--text-secondary)'; continue }
      '#4a9eff' { $suggest[$c] = 'var(--accent-primary)'; continue }
      '#3a8eef' { $suggest[$c] = 'var(--accent-secondary)'; continue }
      '#f44336' { $suggest[$c] = 'var(--danger)'; continue }
      '#4caf50' { $suggest[$c] = 'var(--success)'; continue }
      '#ffffff' { $suggest[$c] = 'var(--bg-darkest)'; continue }
      default { $suggest[$c] = "" }
    }
  }
  $suggest | ConvertTo-Json -Depth 3 | Out-File -Encoding UTF8 $outMap
  Write-Host 'Created $outMap. Edit this file to set replacements (e.g. "var(--accent-primary)")'
} else {
  Write-Host "Mappings file already exists: $outMap"
}

if ($Apply) {
  if (-not (Test-Path $outMap)) {
    Write-Error "Mappings file not found. Run the script without -Apply first to generate mappings.json"
    exit 1
  }
  $mappings = Get-Content $outMap -Raw | ConvertFrom-Json
  if (-not $mappings) { Write-Error "Invalid mappings.json"; exit 1 }

  foreach ($f in $files) {
    $text = Get-Content $f.FullName -Raw
    $orig = $text
    foreach ($k in $mappings.PSObject.Properties.Name) {
      $v = $mappings.$k
      if (-not [string]::IsNullOrWhiteSpace($v)) {
        # replace hex occurrences (case-insensitive)
        $pattern = [regex]::Escape($k)
        $text = [regex]::Replace($text, $pattern, $v, 'IgnoreCase')
      }
    }
    if ($text -ne $orig) {
      Copy-Item -Path $f.FullName -Destination ($f.FullName + '.bak') -Force
      Set-Content -Path $f.FullName -Value $text -Encoding UTF8
      Write-Host "Patched: $($f.FullName) (backup .bak created)"
    }
  }
  Write-Host "Apply complete. Review changes, run git diff / commit when ready."
}