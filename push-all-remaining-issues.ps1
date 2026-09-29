# Push All Remaining Issues (3-125) to GitHub
# This script will create all 123 remaining issues

$ErrorActionPreference = "Continue"
$repo = "shortheartone/SAFE-HAVEN"
$created = 0
$failed = 0

Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  Pushing Issues 3-125 to GitHub" -ForegroundColor Cyan
Write-Host "  Repository: $repo" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "This will take approximately 4-5 minutes..." -ForegroundColor Yellow
Write-Host ""

# Parse ISSUES.md and extract issues 3-125
$content = Get-Content "ISSUES.md" -Raw -Encoding UTF8

# Simple approach: extract each issue section
$issueBlocks = $content -split '(?=## Issue #)'

foreach ($block in $issueBlocks) {
    if ($block -match '## Issue #(\d+): (.+?)\n') {
        $num = [int]$matches[1]
        
        # Skip issues 1 and 2 (already created)
        if ($num -le 2) { continue }
        
        # Extract title
        if ($block -match '## Issue #\d+: (.+?)\n') {
            $title = $matches[1].Trim()
        }
        
        # Extract body (everything after the title line)
        $bodyMatch = $block -match '## Issue #\d+: .+?\n\n(.+?)(?=\n---|\Z)'
        if ($bodyMatch) {
            $body = $matches[1].Trim()
        } else {
            # Fallback: get everything after the title
            $lines = $block -split '\n'
            $body = ($lines[2..($lines.Length-1)] -join "`n").Trim()
            # Remove the trailing "---" if present
            $body = $body -replace '\n---\s*$', ''
        }
        
        Write-Host "[$num/125] Creating: " -NoNewline -ForegroundColor White
        Write-Host $title.Substring(0, [Math]::Min(50, $title.Length)) -ForegroundColor Gray
        
        try {
            $result = gh issue create --repo $repo --title $title --body $body 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host "  ✓ Success" -ForegroundColor Green
                $created++
            } else {
                Write-Host "  ✗ Failed: $result" -ForegroundColor Red
                $failed++
            }
        } catch {
            Write-Host "  ✗ Error: $_" -ForegroundColor Red
            $failed++
        }
        
        # Rate limiting delay
        Start-Sleep -Seconds 2
    }
}

Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host " SUMMARY" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "✓ Issues created: $created" -ForegroundColor Green
Write-Host "✗ Issues failed: $failed" -ForegroundColor Red
Write-Host "Total: $($created + $failed)" -ForegroundColor White
Write-Host ""
Write-Host "View all issues at:" -ForegroundColor Cyan
Write-Host "https://github.com/$repo/issues" -ForegroundColor White
Write-Host ""
