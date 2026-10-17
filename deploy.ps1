param(
    [string]$ProjectName = "cadflow",
    [string]$Branch = "main"
)

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host " CADFlow - Cloudflare Pages 部署脚本" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. 检查 dist 目录是否存在
if (-not (Test-Path "dist\index.html")) {
    Write-Host "[1/3] 正在编译构建 WebAssembly 生产包..." -ForegroundColor Yellow
    Push-Location "cadflow-web"
    trunk build --release
    Pop-Location
} else {
    Write-Host "[1/3] 已检测到最新 dist 发布包。" -ForegroundColor Green
}

# 2. 检查 WASM 体积并确保小于 25MB 限制
Get-ChildItem -Path "dist/*.wasm" | ForEach-Object {
    if ($_.Length -gt 25000000) {
        Write-Host "检测到 $($_.Name) 体积超过 25MB，正在使用 wasm-opt 执行尺寸压缩..." -ForegroundColor Yellow
        npx wasm-opt -Oz $_.FullName -o $_.FullName
    }
}

# 3. 确保 _headers 文件存在
if (-not (Test-Path "dist\_headers")) {
    Copy-Item "cadflow-web\_headers" -Destination "dist\_headers" -Force
}

# 4. 部署到 Cloudflare Pages
Write-Host "[2/3] 检查 Cloudflare 认证状态..." -ForegroundColor Yellow
if (-not $env:CLOUDFLARE_API_TOKEN) {
    Write-Host "提示: 若未配置 Token，将使用已登录的 Cloudflare OAuth 凭据。" -ForegroundColor Gray
}

Write-Host "[3/3] 正在发布至 Cloudflare Pages ($ProjectName)..." -ForegroundColor Yellow
npx wrangler pages deploy dist --project-name $ProjectName --branch $Branch --commit-dirty=true

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✓ 恭喜！CADFlow 已成功发布至 Cloudflare Pages！" -ForegroundColor Green
} else {
    Write-Host "`n! 部署未完成，请检查 Cloudflare 登录凭证或设置 CLOUDFLARE_API_TOKEN。" -ForegroundColor Red
    Write-Host "登录命令: npx wrangler login" -ForegroundColor Cyan
}

