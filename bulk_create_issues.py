#!/usr/bin/env python3
"""
Bulk create GitHub issues from ISSUES.md file
Repository: shortheartone/SAFE-HAVEN
"""

import re
import subprocess
import time
import sys

def parse_issues_from_markdown(file_path):
    """Parse ISSUES.md and extract individual issues."""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Split by issue headers
    issue_pattern = r'## Issue #(\d+): (.+?)\n\n\*\*Summary:\*\*\n(.+?)\n\n\*\*Scope of Work:\*\*\n(.+?)\n\n\*\*Out of Scope:\*\*\n(.+?)\n\n\*\*Acceptance Criteria:\*\*\n(.+?)(?=\n---\n|\n## Issue #|\Z)'
    
    matches = re.finditer(issue_pattern, content, re.DOTALL)
    
    issues = []
    for match in matches:
        issue_num = match.group(1)
        title = match.group(2).strip()
        summary = match.group(3).strip()
        scope = match.group(4).strip()
        out_of_scope = match.group(5).strip()
        acceptance = match.group(6).strip()
        
        body = f"""**Summary:**
{summary}

**Scope of Work:**
{scope}

**Out of Scope:**
{out_of_scope}

**Acceptance Criteria:**
{acceptance}"""
        
        issues.append({
            'number': int(issue_num),
            'title': title,
            'body': body
        })
    
    return issues

def create_github_issue(repo, title, body, dry_run=False):
    """Create a single GitHub issue using gh CLI."""
    if dry_run:
        print(f"[DRY RUN] Would create: {title}")
        return True
    
    try:
        result = subprocess.run(
            ['gh', 'issue', 'create', 
             '--repo', repo,
             '--title', title,
             '--body', body],
            capture_output=True,
            text=True,
            check=True
        )
        return True
    except subprocess.CalledProcessError as e:
        print(f"Error creating issue: {e.stderr}")
        return False

def main():
    repo = "shortheartone/SAFE-HAVEN"
    issues_file = "ISSUES.md"
    
    # Check if running in dry-run mode
    dry_run = '--dry-run' in sys.argv
    
    if dry_run:
        print("=" * 60)
        print("DRY RUN MODE - No issues will be created")
        print("=" * 60)
        print()
    
    print(f"Parsing issues from {issues_file}...")
    try:
        issues = parse_issues_from_markdown(issues_file)
        print(f"✓ Found {len(issues)} issues to create\n")
    except Exception as e:
        print(f"✗ Error parsing file: {e}")
        return
    
    if not dry_run:
        response = input(f"Create {len(issues)} issues in {repo}? (y/n): ")
        if response.lower() != 'y':
            print("Cancelled.")
            return
    
    print()
    print("=" * 60)
    print("Creating GitHub Issues...")
    print("=" * 60)
    print()
    
    created = 0
    failed = 0
    
    for issue in issues:
        issue_num = issue['number']
        title = issue['title']
        body = issue['body']
        
        print(f"[{issue_num}/125] Creating: {title[:50]}...")
        
        success = create_github_issue(repo, title, body, dry_run)
        
        if success:
            created += 1
            print(f"  ✓ Issue #{issue_num} created")
        else:
            failed += 1
            print(f"  ✗ Issue #{issue_num} FAILED")
        
        # Rate limiting: wait 2 seconds between issues to avoid API limits
        if not dry_run and issue_num < len(issues):
            time.sleep(2)
    
    print()
    print("=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"✓ Issues created: {created}")
    print(f"✗ Issues failed:  {failed}")
    print(f"Total:            {len(issues)}")
    print()
    
    if not dry_run:
        print(f"View issues at: https://github.com/{repo}/issues")

if __name__ == "__main__":
    print()
    print("╔════════════════════════════════════════════════════════════╗")
    print("║     GitHub Issues Bulk Creator for SAFE-HAVEN             ║")
    print("╚════════════════════════════════════════════════════════════╝")
    print()
    
    # Check if gh CLI is installed
    try:
        subprocess.run(['gh', '--version'], capture_output=True, check=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("✗ GitHub CLI (gh) is not installed or not in PATH")
        print("  Install from: https://cli.github.com/")
        sys.exit(1)
    
    main()
