# ============================================================================
# Bulk Create GitHub Issues from ISSUES.md
# Repository: shortheartone/SAFE-HAVEN
# ============================================================================

param(
    [switch]$DryRun = $false,
    [int]$StartFrom = 1,
    [int]$EndAt = 125,
    [int]$DelaySeconds = 2
)

$ErrorActionPreference = "Continue"
$repo = "shortheartone/SAFE-HAVEN"
$issuesFile = "ISSUES.md"

Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║     GitHub Issues Bulk Creator for SAFE-HAVEN               ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Check if ISSUES.md exists
if (-not (Test-Path $issuesFile)) {
    Write-Host "✗ Error: $issuesFile not found!" -ForegroundColor Red
    exit 1
}

# Check if gh CLI is available
try {
    $ghVersion = gh --version 2>$null
    Write-Host "✓ GitHub CLI detected: $($ghVersion[0])" -ForegroundColor Green
} catch {
    Write-Host "✗ GitHub CLI (gh) is not installed or not authenticated" -ForegroundColor Red
    Write-Host "  Run 'gh auth login' first" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
if ($DryRun) {
    Write-Host "═══════════════════════════════════════" -ForegroundColor Yellow
    Write-Host " DRY RUN MODE - No issues will be created" -ForegroundColor Yellow
    Write-Host "═══════════════════════════════════════" -ForegroundColor Yellow
    Write-Host ""
}

# Parse ISSUES.md file
Write-Host "Parsing $issuesFile..." -ForegroundColor Cyan
$content = Get-Content $issuesFile -Raw

# Extract issues using regex
$issuePattern = '## Issue #(\d+): ([^\n]+)\n\n\*\*Summary:\*\*\n([^\*]+)\n\*\*Scope of Work:\*\*\n([^\*]+)\n\*\*Out of Scope:\*\*\n([^\*]+)\n\*\*Acceptance Criteria:\*\*\n([^#\-]+)'

$matches = [regex]::Matches($content, $issuePattern, [System.Text.RegularExpressions.RegexOptions]::Singleline)

$issues = @()
foreach ($match in $matches) {
    $issueNum = [int]$match.Groups[1].Value
    $title = $match.Groups[2].Value.Trim()
    $summary = $match.Groups[3].Value.Trim()
    $scope = $match.Groups[4].Value.Trim()
    $outOfScope = $match.Groups[5].Value.Trim()
    $acceptance = $match.Groups[6].Value.Trim()
    
    $body = @"
**Summary:**
$summary

**Scope of Work:**
$scope

**Out of Scope:**
$outOfScope

**Acceptance Criteria:**
$acceptance
"@
    
    if ($issueNum -ge $StartFrom -and $issueNum -le $EndAt) {
        $issues += @{
            Number = $issueNum
            Title = $title
            Body = $body
        }
    }
}

Write-Host "✓ Found $($issues.Count) issues to create (from #$StartFrom to #$EndAt)" -ForegroundColor Green
Write-Host ""

if ($issues.Count -eq 0) {
    Write-Host "✗ No issues found to create!" -ForegroundColor Red
    exit 1
}

# Confirmation prompt
if (-not $DryRun) {
    $estimatedTime = [math]::Ceiling(($issues.Count * $DelaySeconds) / 60)
    Write-Host "This will create $($issues.Count) issues in repository: $repo" -ForegroundColor Yellow
    Write-Host "Estimated time: ~$estimatedTime minutes (with $DelaySeconds second delay between issues)" -ForegroundColor Yellow
    Write-Host ""
    $response = Read-Host "Continue? (y/n)"
    if ($response -ne 'y' -and $response -ne 'Y') {
        Write-Host "Cancelled." -ForegroundColor Yellow
        exit 0
    }
}

Write-Host ""
Write-Host "════════════════════════════════════════" -ForegroundColor Cyan
Write-Host " Creating GitHub Issues..." -ForegroundColor Cyan
Write-Host "════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

$created = 0
$failed = 0
$failedIssues = @()

foreach ($issue in $issues) {
    $num = $issue.Number
    $title = $issue.Title
    $body = $issue.Body
    
    $shortTitle = if ($title.Length -gt 55) { $title.Substring(0, 55) + "..." } else { $title }
    Write-Host "[$num/125] " -NoNewline -ForegroundColor White
    Write-Host "$shortTitle" -ForegroundColor Gray
    
    if ($DryRun) {
        Write-Host "  ✓ [DRY RUN] Would create this issue" -ForegroundColor Green
        $created++
    } else {
        try {
            # Create issue using gh CLI
            $result = gh issue create --repo $repo --title $title --body $body 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host "  ✓ Issue #$num created successfully" -ForegroundColor Green
                $created++
            } else {
                Write-Host "  ✗ Failed: $result" -ForegroundColor Red
                $failed++
                $failedIssues += $num
            }
            
            # Rate limiting delay (except for last issue)
            if ($num -lt $issues[-1].Number) {
                Start-Sleep -Seconds $DelaySeconds
            }
        } catch {
            Write-Host "  ✗ Error: $_" -ForegroundColor Red
            $failed++
            $failedIssues += $num
        }
    }
}

# Summary
Write-Host ""
Write-Host "════════════════════════════════════════" -ForegroundColor Cyan
Write-Host " SUMMARY" -ForegroundColor Cyan
Write-Host "════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "✓ Issues created: " -NoNewline -ForegroundColor Green
Write-Host $created
Write-Host "✗ Issues failed:  " -NoNewline -ForegroundColor Red
Write-Host $failed
Write-Host "Total processed:  $($issues.Count)"
Write-Host ""

if ($failedIssues.Count -gt 0) {
    Write-Host "Failed issue numbers: " -NoNewline -ForegroundColor Yellow
    Write-Host ($failedIssues -join ", ")
    Write-Host ""
    Write-Host "To retry failed issues, run:" -ForegroundColor Yellow
    Write-Host "  .\Create-AllIssues.ps1 -StartFrom $($failedIssues[0]) -EndAt $($failedIssues[-1])" -ForegroundColor White
    Write-Host ""
}

if (-not $DryRun -and $created -gt 0) {
    Write-Host "View issues at: https://github.com/$repo/issues" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "Done!" -ForegroundColor Green
Write-Host ""
