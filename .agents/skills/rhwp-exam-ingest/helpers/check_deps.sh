#!/usr/bin/env bash
# rhwp-exam-ingest helper: 의존성 확인
#
# 사용법:
#   check_deps.sh [--json]
#
# 종료 코드:
#   0  필수 의존성 충족 (선택 누락은 경고만)
#   1  필수 의존성 누락 (rhwp / ImageMagick). PDF-only 도구는
#      필수 아님 — 입력 종류가 PDF 일 때만 막는다.
#   2  사용법 오류
#
# --json 이면 stdout 에 실패/성공 봉투를 쓴다. stderr 는 사람용 표.

set -u

JSON=0
if [ "${1:-}" = "--json" ]; then
    JSON=1
    shift
elif [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
    echo "사용법: check_deps.sh [--json]" >&2
    exit 0
elif [ -n "${1:-}" ]; then
    echo "사용법: check_deps.sh [--json]" >&2
    exit 2
fi

ok=true
miss_rhwp=0
miss_magick=0
miss_pdftoppm=0
miss_pdftotext=0
miss_python=0
miss_python_docx=0
have_magick=0
have_convert=0
have_python_docx=0
rhwp_path=""

echo "=== rhwp-exam-ingest 의존성 점검 ===" >&2
echo >&2

echo "[필수]" >&2
if [ -x "./target/release/rhwp" ]; then
    echo "[OK]    ./target/release/rhwp — rhwp 바이너리 (cargo build --release 결과)" >&2
    rhwp_path="./target/release/rhwp"
elif command -v rhwp >/dev/null 2>&1; then
    echo "[OK]    rhwp — PATH에 설치됨" >&2
    rhwp_path="$(command -v rhwp)"
else
    echo "[MISS]  rhwp 바이너리 — 'cargo build --release' 또는 'cargo run --bin rhwp' 사용" >&2
    ok=false
    miss_rhwp=1
fi

echo >&2
echo "[PDF 입력]" >&2
if command -v pdftoppm >/dev/null 2>&1; then
    echo "[OK]    pdftoppm — PDF → PNG (poppler-utils 권장)" >&2
else
    echo "[MISS]  pdftoppm — PDF → PNG (poppler-utils). PDF 입력 시 필요. magick fallback 가능" >&2
    miss_pdftoppm=1
fi
if command -v pdftotext >/dev/null 2>&1; then
    echo "[OK]    pdftotext — PDF 텍스트 layer 추출 (선택, 정확도 향상)" >&2
else
    echo "[MISS]  pdftotext — PDF 텍스트 layer 추출 (선택)" >&2
    miss_pdftotext=1
fi

echo >&2
echo "[DOCX 입력]" >&2
if command -v python3 >/dev/null 2>&1; then
    echo "[OK]    python3 — Python 3" >&2
    if python3 -c "import docx" 2>/dev/null; then
        echo "[OK]    python-docx — DOCX 정밀 추출" >&2
        have_python_docx=1
    else
        echo "[INFO]  python-docx 없음 — fallback (정규식 추출) 사용. 설치 권장: 'pip install python-docx'" >&2
        miss_python_docx=1
    fi
else
    echo "[MISS]  python3 — DOCX 입력 시 필요" >&2
    miss_python=1
    miss_python_docx=1
fi

echo >&2
echo "[이미지 자르기]" >&2
if command -v magick >/dev/null 2>&1; then
    echo "[OK]    magick — ImageMagick 7" >&2
    have_magick=1
elif command -v convert >/dev/null 2>&1; then
    echo "[OK]    convert — ImageMagick 6" >&2
    have_convert=1
else
    echo "[MISS]  magick / convert — ImageMagick 필요. 설치: 'brew install imagemagick' (macOS)" >&2
    ok=false
    miss_magick=1
fi

echo >&2

emit_json() {
    # 실패 봉투: 누락 도구별로 기계가 읽을 수 있는 이유 코드.
    # 새 rhwp CLI 가 아니다. 이 스킬 helper 의 stdout 계약이다.
    python3 - "$JSON" "$ok" "$miss_rhwp" "$miss_magick" "$miss_pdftoppm" \
        "$miss_pdftotext" "$miss_python" "$miss_python_docx" \
        "$have_magick" "$have_convert" "$have_python_docx" "$rhwp_path" <<'PY'
import json, sys
args = sys.argv[1:]
(
    _want, ok_s, miss_rhwp, miss_magick, miss_pdftoppm, miss_pdftotext,
    miss_python, miss_python_docx, have_magick, have_convert,
    have_python_docx, rhwp_path,
) = args
ok = ok_s == "true"
missing = []
optional = []
envelopes = []
if miss_rhwp == "1":
    missing.append("rhwp")
    envelopes.append({
        "code": "DEP_MISS_RHWP",
        "severity": "required",
        "tool": "rhwp",
        "exit": 1,
        "hint": "cargo build --release 또는 cargo run --bin rhwp",
        "blocks": ["build-from-ingest"],
    })
if miss_magick == "1":
    missing.append("imagemagick")
    envelopes.append({
        "code": "DEP_MISS_IMAGEMAGICK",
        "severity": "required",
        "tool": "magick|convert",
        "exit": 1,
        "hint": "brew install imagemagick 또는 apt install imagemagick",
        "blocks": ["crop_image.sh", "pdf_to_pngs.sh(magick fallback)"],
    })
if miss_pdftoppm == "1":
    optional.append("pdftoppm")
    envelopes.append({
        "code": "DEP_MISS_POPPLER",
        "severity": "pdf_input",
        "tool": "pdftoppm",
        "exit": 0 if ok else 1,
        "hint": "brew install poppler 또는 apt install poppler-utils. magick fallback 가능",
        "blocks": ["pdf_to_pngs.sh (preferred)"],
    })
if miss_pdftotext == "1":
    optional.append("pdftotext")
    envelopes.append({
        "code": "DEP_MISS_PDFTOTEXT",
        "severity": "optional",
        "tool": "pdftotext",
        "exit": 0,
        "hint": "poppler-utils. 없어도 Vision 만으로 진행 가능",
        "blocks": [],
    })
if miss_python == "1":
    optional.append("python3")
    envelopes.append({
        "code": "DEP_MISS_PYTHON3",
        "severity": "docx_input",
        "tool": "python3",
        "exit": 0 if ok else 1,
        "hint": "DOCX 입력이면 python3 필요",
        "blocks": ["extract_docx.py"],
    })
if miss_python_docx == "1":
    optional.append("python-docx")
    envelopes.append({
        "code": "DEP_MISS_PYTHON_DOCX",
        "severity": "docx_input_soft",
        "tool": "python-docx",
        "exit": 0,
        "hint": "pip install python-docx. 없어도 zip+정규식 fallback",
        "blocks": [],
        "fallback": "extract_docx.py zip regex",
    })
doc = {
    "schemaVersion": "1.0",
    "helper": "check_deps.sh",
    "ok": ok,
    "rhwp": rhwp_path or None,
    "imagemagick": (
        "magick" if have_magick == "1"
        else ("convert" if have_convert == "1" else None)
    ),
    "pythonDocx": have_python_docx == "1",
    "missingRequired": missing,
    "missingOptional": optional,
    "envelopes": envelopes,
}
print(json.dumps(doc, ensure_ascii=False, indent=2))
PY
}

if [ "$JSON" = "1" ]; then
    emit_json
fi

if $ok; then
    echo "✅ 모든 필수 의존성 OK. rhwp-exam-ingest Skill 사용 준비 완료." >&2
    exit 0
else
    echo "❌ 일부 의존성 누락. 위 'MISS' 항목을 설치 후 재시도하세요." >&2
    echo "   PDF 전용 누락(poppler)은 PDF 입력이 아니면 진행 가능." >&2
    echo "   python-docx 누락은 DOCX 정규식 fallback 으로 진행 가능." >&2
    exit 1
fi
