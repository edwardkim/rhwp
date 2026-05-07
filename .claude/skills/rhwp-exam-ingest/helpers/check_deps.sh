#!/usr/bin/env bash
# rhwp-exam-ingest helper: 의존성 확인
#
# 사용법: check_deps.sh

set -u

ok=true

check() {
    local cmd="$1"
    local desc="$2"
    if command -v "$cmd" >/dev/null 2>&1; then
        echo "[OK]    $cmd — $desc"
    else
        echo "[MISS]  $cmd — $desc"
        ok=false
    fi
}

echo "=== rhwp-exam-ingest 의존성 점검 ==="
echo

echo "[필수]"
if [ -x "./target/release/rhwp" ]; then
    echo "[OK]    ./target/release/rhwp — rhwp 바이너리 (cargo build --release 결과)"
elif command -v rhwp >/dev/null 2>&1; then
    echo "[OK]    rhwp — PATH에 설치됨"
else
    echo "[MISS]  rhwp 바이너리 — 'cargo build --release' 또는 'cargo run --bin rhwp' 사용"
    ok=false
fi

echo
echo "[PDF 입력]"
check "pdftoppm" "PDF → PNG (poppler-utils 권장)"
check "pdftotext" "PDF 텍스트 layer 추출 (선택, 정확도 향상)"

echo
echo "[DOCX 입력]"
check "python3" "Python 3"
if command -v python3 >/dev/null 2>&1; then
    if python3 -c "import docx" 2>/dev/null; then
        echo "[OK]    python-docx — DOCX 정밀 추출"
    else
        echo "[INFO]  python-docx 없음 — fallback (정규식 추출) 사용. 설치 권장: 'pip install python-docx'"
    fi
fi

echo
echo "[이미지 자르기]"
if command -v magick >/dev/null 2>&1; then
    echo "[OK]    magick — ImageMagick 7"
elif command -v convert >/dev/null 2>&1; then
    echo "[OK]    convert — ImageMagick 6"
else
    echo "[MISS]  magick / convert — ImageMagick 필요. 설치: 'brew install imagemagick' (macOS)"
    ok=false
fi

echo
if $ok; then
    echo "✅ 모든 필수 의존성 OK. rhwp-exam-ingest Skill 사용 준비 완료."
    exit 0
else
    echo "❌ 일부 의존성 누락. 위 'MISS' 항목을 설치 후 재시도하세요."
    exit 1
fi
