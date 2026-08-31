# ============================================
# HUSH GITHUB MASTER SETUP SCRIPT
# ============================================
$REPO = "seyallius/hush"
$OWNER = "seyallius"
$PROJECT_NUM = 6 # Based on your `gh project list` output

Write-Host "🧹 Phase 1: Cleaning up existing data..." -ForegroundColor Yellow

# Delete open issues
gh issue list --state open --json number --jq '.[].number' | ForEach-Object { 
    gh issue delete $_ --yes 2>$null
}

# Delete labels
gh label list --json name --jq '.[].name' | ForEach-Object { 
    gh label delete $_ --yes 2>$null
}

# Delete milestones
gh api repos/$REPO/milestones --jq '.[].number' | ForEach-Object { 
    gh api -X DELETE repos/$REPO/milestones/$_ 2>$null
}

# Delete project board
gh project delete $PROJECT_NUM --owner $OWNER --yes 2>$null

Write-Host "✅ Cleanup complete!`n" -ForegroundColor Green

Write-Host "🏗️ Phase 2: Creating Labels & Milestones..." -ForegroundColor Cyan

gh label create "priority:P0" --color "B60205" --description "MVP" >$null
gh label create "priority:P1" --color "D93F0B" --description "v0.2.0" >$null
gh label create "priority:P2" --color "FBCA04" --description "v1.0.0" >$null
gh label create "priority:P3" --color "0E8A16" --description "v2.0.0" >$null
gh label create "priority:P4" --color "1D76DB" --description "v3.0.0" >$null

gh label create "type:enhancement" --color "A2EEEF" --description "Feature" >$null
gh label create "type:bug" --color "D73A4A" --description "Bug" >$null
gh label create "type:documentation" --color "0075CA" --description "Docs" >$null
gh label create "type:testing" --color "F9D0C4" --description "Tests" >$null
gh label create "type:refactor" --color "1D76DB" --description "Refactor" >$null
gh label create "status:blocked" --color "D93F0B" --description "Blocked" >$null
gh label create "status:needs-review" --color "FBCA04" --description "Review" >$null

# Create milestones via API
gh api repos/$REPO/milestones -f title="v0.1.0 - MVP" -f description="Core MVP" -f due_on="2026-10-31T23:59:59Z" >$null
gh api repos/$REPO/milestones -f title="v0.2.0 - Flexibility" -f description="YubiKey, Config" -f due_on="2026-12-31T23:59:59Z" >$null
gh api repos/$REPO/milestones -f title="v1.0.0 - Production" -f description="Perf, ARM, Docs" -f due_on="2027-03-31T23:59:59Z" >$null
gh api repos/$REPO/milestones -f title="v2.0.0 - Vault" -f description="Dirs, FUSE, SSS" -f due_on="2027-06-30T23:59:59Z" >$null
gh api repos/$REPO/milestones -f title="v3.0.0 - Power User" -f description="Batch, Plugins" -f due_on="2027-09-30T23:59:59Z" >$null

Write-Host "🏗️ Phase 3: Creating Project Board..." -ForegroundColor Cyan
gh project create --title "Hush Development" --owner $OWNER >$null

Write-Host "🏗️ Phase 4: Creating Issues..." -ForegroundColor Cyan

# Helper function to avoid CLI escaping and here-string errors
function New-HushIssue {
    param([string]$Title, [string]$Body, [string]$Labels, [string]$Milestone)
    $temp = "temp_hush_issue.md"
    $Body | Out-File -FilePath $temp -Encoding utf8
	
    $out = gh issue create --title $Title --body-file $temp --label $Labels --milestone $Milestone
    $url = $out | Where-Object { $_ -match "https://github.com" } | Select-Object -First 1
    
    if ($url) {
        Write-Host "✅ $Title" -ForegroundColor Green
        gh project item-add $PROJECT_NUM --owner $OWNER --url $url >$null
    } else {
        Write-Host "❌ $Title" -ForegroundColor Red; Write-Host $out
    }
    Remove-Item $temp -ErrorAction SilentlyContinue
}

# --- MILESTONE 1: v0.1.0 - MVP ---
$M1 = "v0.1.0 - MVP"
$L1 = "type:enhancement,priority:P0"

$b1 = @"
## Description
Implement chunked streaming encryption using XChaCha20-Poly1305 (AEAD) with 1MB chunk size.
## Tasks
- [ ] Research chacha20poly1305 crate streaming API
- [ ] Implement StreamCipher trait for XChaCha20-Poly1305
- [ ] Write unit tests for encryption/decryption
- [ ] Benchmark performance
## Acceptance Criteria
- [ ] Can encrypt a 1GB file using <50MB RAM
- [ ] Each chunk has unique nonce
- [ ] Integrity check works (tampering detection)
"@
New-HushIssue "Implement XChaCha20-Poly1305 streaming encryption" $b1 $L1 $M1

$b2 = @"
## Description
Create the VAUL binary container format with magic bytes, version, salt, and encrypted metadata.
## Tasks
- [ ] Create FileHeader struct
- [ ] Create FileMetadata struct (filename, mime, size, chunk offsets)
- [ ] Implement serialization with bincode
- [ ] Write header to file during encryption
"@
New-HushIssue "Define custom Envelope binary format" $b2 $L1 $M1

$b3 = @"
## Description
Implement key derivation using Argon2id (memory-hard KDF) for password-only mode.
## Tasks
- [ ] Research argon2 crate API
- [ ] Implement derive_key_from_password()
- [ ] Store salt in header
- [ ] Unit tests & Error handling for wrong password
"@
New-HushIssue "Argon2id password-based key derivation" $b3 $L1 $M1

$b4 = @"
## Description
Setup the CLI using Clap with all subcommands.
## Tasks
- [ ] Setup Clap with derive feature
- [ ] Implement all subcommands
- [ ] Add --help and --version
- [ ] Error handling with anyhow/thiserror
"@
New-HushIssue "CLI structure with Clap" $b4 $L1 $M1

$b5 = @"
## Description
Start a local HTTP server that streams decrypted content to VLC/media players.
## Tasks
- [ ] Research axum/warp HTTP frameworks
- [ ] Implement decryption-to-HTTP streaming
- [ ] Handle client disconnects gracefully
- [ ] Serve decrypted bytes on-the-fly
"@
New-HushIssue "HTTP streaming server for decrypted playback" $b5 $L1 $M1

$b6 = @"
## Description
Create a TUI interface with ratatui that shows encrypted files in current directory.
## Tasks
- [ ] Setup ratatui with crossterm
- [ ] File discovery in current directory
- [ ] Parse headers to show metadata
- [ ] Open VLC on selection
"@
New-HushIssue "Basic TUI with ratatui" $b6 $L1 $M1

$b7 = @"
## Description
When user selects a file in TUI, automatically launch VLC media player with the HTTP stream URL.
## Tasks
- [ ] Detect VLC installation path
- [ ] Start HTTP server in background
- [ ] Launch VLC with stream URL
- [ ] Handle process cleanup when VLC exits
"@
New-HushIssue "VLC auto-launch from TUI" $b7 $L1 $M1

# --- MILESTONE 2: v0.2.0 - Flexibility ---
$M2 = "v0.2.0 - Flexibility"
$L2 = "type:enhancement,priority:P1"

$b8 = @"
## Description
Implement YubiKey challenge-response authentication using HMAC-SHA1.
## Tasks
- [ ] Research yubikey crate
- [ ] Implement challenge-response flow
- [ ] Support YubiKey-only mode
- [ ] Unit tests with mock YubiKey
"@
New-HushIssue "YubiKey integration (HMAC-SHA1)" $b8 $L2 $M2

$b9 = @"
## Description
Combine password and YubiKey using HKDF for 2FA authentication.
## Tasks
- [ ] Implement combined mode in key derivation
- [ ] Store both salt and challenge in header
- [ ] Support all three modes
- [ ] CLI flag --key-mode combined
"@
New-HushIssue "Combined key mode (Password + YubiKey)" $b9 $L2 $M2

$b10 = @"
## Description
Support user configuration via TOML file (~/.config/hush/config.toml).
## Tasks
- [ ] Create config directory on first run
- [ ] Load config with default fallback
- [ ] CLI flags override config values
- [ ] Generate config template via --init
"@
New-HushIssue "Config file support" $b10 $L2 $M2

$b11 = @"
## Description
Implement AES-256-GCM as an alternative cipher via the StreamCipher trait.
## Tasks
- [ ] Implement AesCipher struct
- [ ] Use aes-gcm crate (with streaming support)
- [ ] Switch based on config
- [ ] Unit tests & Benchmark comparison
"@
New-HushIssue "AES-256-GCM cipher support" $b11 $L2 $M2

$b12 = @"
## Description
Improve TUI to show more file metadata.
## Tasks
- [ ] Parse and display full metadata
- [ ] Add sorting options (name, size, date)
- [ ] Filter/search for files
- [ ] Show file count and total size
"@
New-HushIssue "Enhanced TUI with metadata display" $b12 $L2 $M2

# --- MILESTONE 3: v1.0.0 - Production ---
$M3 = "v1.0.0 - Production"
$L3 = "type:enhancement,priority:P2"

$b13 = @"
## Description
Refactor encryption/decryption to handle chunk offsets dynamically without reading the whole file.
## Tasks
- [ ] Research streaming-write patterns
- [ ] Implement temporary file approach
- [ ] Or implement two-pass approach
- [ ] Memory usage < 10MB for any file size
"@
New-HushIssue "True zero-RAM streaming (dynamic chunk offsets)" $b13 $L3 $M3

$b14 = @"
## Description
Ensure builds work on ARM architectures (Raspberry Pi, M1/M2 Macs, ARM Windows).
## Tasks
- [ ] Test cross-compilation
- [ ] Add CI workflows for ARM
- [ ] Document ARM installation
- [ ] Ensure dependencies are ARM-compatible
"@
New-HushIssue "ARM support" $b14 $L3 $M3

$b15 = @"
## Description
Create comprehensive documentation (README, usage examples, file format spec).
## Tasks
- [ ] Write all documentation
- [ ] Add code examples
- [ ] Create GIF demos
- [ ] Publish to crates.io
"@
New-HushIssue "Full documentation with examples" $b15 "type:documentation,priority:P2" $M3

# --- MILESTONE 4: v2.0.0 - Vault ---
$M4 = "v2.0.0 - Vault"
$L4 = "type:enhancement,priority:P3"

$b16 = @"
## Description
Encrypt an entire directory, preserving structure, with a single index file.
## Tasks
- [ ] Design index format
- [ ] Implement recursive directory traversal
- [ ] Encrypt each file as separate chunk
- [ ] Single password unlocks everything
"@
New-HushIssue "Directory encryption with index" $b16 $L4 $M4

$b17 = @"
## Description
Mount encrypted directory as a virtual filesystem (FUSE).
## Tasks
- [ ] Research fuse crate
- [ ] Implement file operations (read, stat, list)
- [ ] Handle large directory trees
- [ ] Linux/Mac/Windows support
"@
New-HushIssue "FUSE filesystem mount" $b17 $L4 $M4

$b18 = @"
## Description
Implement Shamir's Secret Sharing (SSS) to split master key into shares.
## Tasks
- [ ] Research sss crate
- [ ] Implement key splitting
- [ ] Implement key recovery
- [ ] Generate key shares as QR codes
"@
New-HushIssue "Shamir's Secret Sharing for key recovery" $b18 $L4 $M4

$b19 = @"
## Description
Generate QR codes containing recovery parameters (salt, argon2 params, etc).
## Tasks
- [ ] Research qrcode crate
- [ ] Design encoding format (JSON + zlib)
- [ ] Print QR to terminal
- [ ] Optional: Save as PNG
"@
New-HushIssue "Recovery QR codes for offline backup" $b19 $L4 $M4

# --- MILESTONE 5: v3.0.0 - Power User ---
$M5 = "v3.0.0 - Power User"
$L5 = "type:enhancement,priority:P4"

$b20 = @"
## Description
Support encrypting/decrypting multiple files in one command using glob patterns.
## Tasks
- [ ] Support glob patterns
- [ ] Parallel processing (rayon)
- [ ] Progress bars for each file
- [ ] Error handling (continue on error)
"@
New-HushIssue "Batch operations" $b20 $L5 $M5

$b21 = @"
## Description
Allow re-encrypting files with a new password without decrypting to disk.
## Tasks
- [ ] Implement --rekey flag
- [ ] Support both file and directory re-encryption
- [ ] Preserve metadata
- [ ] Handle cancellation safely
"@
New-HushIssue "Key rotation" $b21 $L5 $M5

$b22 = @"
## Description
Allow users to define hooks for custom behavior (before/after encryption, etc).
## Tasks
- [ ] Design hook configuration format
- [ ] Implement hook execution
- [ ] Handle process spawning and waiting
"@
New-HushIssue "Plugin system for user hooks" $b22 $L5 $M5

Write-Host "`n=========================================" -ForegroundColor Cyan
Write-Host "🎉 ALL ISSUES CREATED AND ADDED TO PROJECT!" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan