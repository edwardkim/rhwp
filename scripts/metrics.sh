#!/usr/bin/env bash
# rhwp 코드 품질 메트릭 수집 스크립트
# 사용법: ./scripts/metrics.sh [--snapshot [라벨]] [--no-coverage] [--diff <직전 스냅샷 dir>]
# 결과: mydocs/metrics/{dashboard.html,metrics.json,metrics_history.json} (tracked 발행)
#       + output/ 로컬 사본. 스냅샷은 mydocs/metrics/<날짜-라벨>/ 보관.
# --snapshot [라벨]: 수집 후 mydocs/metrics/{오늘날짜}[-라벨]/ 로 보관 (커밋해 공유 —
#             리팩토링 Phase 경계/릴리즈/코드 리뷰 등 의미 있는 시점만).
#             라벨을 주면 같은 날짜에 여러 스냅샷을 덮어쓰기 없이 보관할 수 있다
#             (예: --snapshot r1 → mydocs/metrics/2026-07-04-r1/).
# --no-coverage: cargo-tarpaulin 커버리지 측정 생략 (수십 분 소요 회피용 — coverage=null)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_DIR/output"
SNAPSHOT=false
SNAPSHOT_LABEL=""
RUN_COVERAGE=true
while [ $# -gt 0 ]; do
    case "$1" in
        --snapshot)
            SNAPSHOT=true
            # 다음 인자가 옵션이 아니면 라벨로 소비
            if [ $# -gt 1 ] && [ "${2#--}" = "$2" ]; then
                SNAPSHOT_LABEL="$2"
                shift
            fi
            ;;
        --no-coverage) RUN_COVERAGE=false ;;
        --diff)
            DIFF_BASE="$2"
            shift
            ;;
        *) echo "알 수 없는 옵션: $1" >&2; exit 2 ;;
    esac
    shift
done

mkdir -p "$OUTPUT_DIR"

echo "=== rhwp 코드 품질 메트릭 수집 ==="
echo "프로젝트: $PROJECT_DIR"
echo ""

# ── 1. 파일별 줄 수 (Rust) ──
echo "[1/5] 파일별 줄 수 측정..."
FILE_LINES_JSON="["
first=true
while IFS= read -r line; do
    lines=$(echo "$line" | awk '{print $1}')
    file=$(echo "$line" | awk '{print $2}' | sed "s|$PROJECT_DIR/||")
    if [ "$first" = true ]; then
        first=false
    else
        FILE_LINES_JSON+=","
    fi
    FILE_LINES_JSON+="{\"file\":\"$file\",\"lines\":$lines}"
done < <(find "$PROJECT_DIR/src" -name "*.rs" -exec wc -l {} \; | sort -rn)

# TS/CSS 파일 포함
while IFS= read -r line; do
    lines=$(echo "$line" | awk '{print $1}')
    file=$(echo "$line" | awk '{print $2}' | sed "s|$PROJECT_DIR/||")
    FILE_LINES_JSON+=",{\"file\":\"$file\",\"lines\":$lines}"
done < <(find "$PROJECT_DIR/rhwp-studio/src" \( -name "*.ts" -o -name "*.css" \) -exec wc -l {} \; | sort -rn)
FILE_LINES_JSON+="]"

# ── 2. Clippy 경고 수 ──
echo "[2/5] Clippy 경고 측정..."
CLIPPY_OUTPUT=$(cargo clippy --manifest-path "$PROJECT_DIR/Cargo.toml" 2>&1 || true)
CLIPPY_WARNINGS=$(echo "$CLIPPY_OUTPUT" | grep -c "^warning:" || true)
CLIPPY_WARNINGS=${CLIPPY_WARNINGS:-0}
CLIPPY_AUTOFIX=$(echo "$CLIPPY_OUTPUT" | grep -oP '\d+(?= warnings? .* can be fixed)' || echo "0")
CLIPPY_AUTOFIX=${CLIPPY_AUTOFIX:-0}

# ── 3. Cognitive Complexity (Clippy 기반) ──
echo "[3/5] Cognitive Complexity 측정..."
# clippy.toml에 낮은 임계값을 임시 설정하여 상위 함수들도 수집
CLIPPY_TOML="$PROJECT_DIR/clippy.toml"
CLIPPY_BACKUP=""
if [ -f "$CLIPPY_TOML" ]; then
    CLIPPY_BACKUP=$(cat "$CLIPPY_TOML")
fi
# 임계값 5로 낮춰서 CC ≥ 5인 함수 모두 수집
echo 'cognitive-complexity-threshold = 5' >> "$CLIPPY_TOML"
CC_OUTPUT=$(cargo clippy --manifest-path "$PROJECT_DIR/Cargo.toml" -- -W clippy::cognitive_complexity 2>&1 || true)
# clippy.toml 복원
if [ -n "$CLIPPY_BACKUP" ]; then
    echo "$CLIPPY_BACKUP" > "$CLIPPY_TOML"
else
    rm -f "$CLIPPY_TOML"
fi
# paste 결과: "warning: ...complexity of (N/5)@   --> file:line:col"
# warning 줄이 먼저, --> 줄이 뒤에 오는 순서
# sed로 complexity와 file:line 추출 (grep -P 없는 환경 대응)
CC_RAW=$(echo "$CC_OUTPUT" | grep -E "(-->.*\.rs:|cognitive complexity of)" | \
    paste -d'@' - - | \
    grep "cognitive complexity" | \
    sed 's/.*of (\([0-9]*\)\/[0-9]*).*--> \([^:]*:[0-9]*\).*/\2	\1/' | \
    sort -t'	' -k2 -rn)
CC_JSON="["
cc_first=true
while IFS=$'\t' read -r location complexity; do
    if [ -z "$location" ] || [ -z "$complexity" ]; then continue; fi
    file=$(echo "$location" | cut -d: -f1)
    line=$(echo "$location" | cut -d: -f2)
    if [ "$cc_first" = true ]; then
        cc_first=false
    else
        CC_JSON+=","
    fi
    CC_JSON+="{\"file\":\"$file\",\"line\":$line,\"complexity\":$complexity}"
done <<< "$CC_RAW"
CC_JSON+="]"

# ── 4. 테스트 현황 ──
echo "[4/5] 테스트 실행..."
TEST_OUTPUT=$(cargo test --manifest-path "$PROJECT_DIR/Cargo.toml" 2>&1 || true)
# 여러 test result 줄에서 합산
TEST_PASSED=$(echo "$TEST_OUTPUT" | grep "^test result:" | grep -oP '\d+(?= passed)' | awk '{s+=$1}END{print s+0}')
TEST_FAILED=$(echo "$TEST_OUTPUT" | grep "^test result:" | grep -oP '\d+(?= failed)' | awk '{s+=$1}END{print s+0}')
TEST_IGNORED=$(echo "$TEST_OUTPUT" | grep "^test result:" | grep -oP '\d+(?= ignored)' | awk '{s+=$1}END{print s+0}')

# ── 5. 커버리지 (cargo-tarpaulin 있을 때만, --no-coverage 로 생략 가능) ──
COVERAGE="null"
if [ "$RUN_COVERAGE" = false ]; then
    echo "[5/5] 커버리지 측정 생략 (--no-coverage)"
elif command -v cargo-tarpaulin &> /dev/null; then
    echo "[5/5] 커버리지 측정..."
    TARP_OUTPUT=$(cargo tarpaulin --manifest-path "$PROJECT_DIR/Cargo.toml" --skip-clean 2>&1 || true)
    COVERAGE=$(echo "$TARP_OUTPUT" | grep -oP '[\d.]+(?=% coverage)' | tail -1 || echo "null")
else
    echo "[5/5] 커버리지 측정 생략 (cargo-tarpaulin 미설치)"
fi

# ── 타임스탬프 ──
TIMESTAMP=$(date -Iseconds)

# ── JSON 출력 ──
cat > "$OUTPUT_DIR/metrics.json" << ENDJSON
{
  "timestamp": "$TIMESTAMP",
  "label": "$SNAPSHOT_LABEL",
  "file_lines": $FILE_LINES_JSON,
  "clippy": {
    "warnings": $CLIPPY_WARNINGS,
    "autofix": ${CLIPPY_AUTOFIX:-0}
  },
  "cognitive_complexity": $CC_JSON,
  "tests": {
    "passed": $TEST_PASSED,
    "failed": $TEST_FAILED,
    "ignored": $TEST_IGNORED
  },
  "coverage": $COVERAGE,
  "thresholds": {
    "max_lines": 1200,
    "max_cognitive_complexity": 15,
    "warn_cognitive_complexity": 25,
    "target_clippy_warnings": 0,
    "target_coverage": 70
  }
}
ENDJSON

# ── 히스토리 저장 ──
HISTORY_DIR="$OUTPUT_DIR/metrics_history"
mkdir -p "$HISTORY_DIR"
DATE_STAMP=$(date +%Y%m%d_%H%M%S)
cp "$OUTPUT_DIR/metrics.json" "$HISTORY_DIR/metrics_${DATE_STAMP}.json"
# 최근 30개만 유지
ls -t "$HISTORY_DIR"/metrics_*.json 2>/dev/null | tail -n +31 | xargs rm -f 2>/dev/null || true

# ── 히스토리 요약 JSON 생성 (대시보드 트렌드용) ──
SUMMARY="["
sfirst=true
for hfile in $(ls -t "$HISTORY_DIR"/metrics_*.json 2>/dev/null | head -20 | tac); do
    ts=$(python3 -c "import json; d=json.load(open('$hfile')); print(d.get('timestamp',''))" 2>/dev/null || echo "")
    tp=$(python3 -c "import json; d=json.load(open('$hfile')); print(d['tests']['passed'])" 2>/dev/null || echo "0")
    tf=$(python3 -c "import json; d=json.load(open('$hfile')); print(d['tests']['failed'])" 2>/dev/null || echo "0")
    cw=$(python3 -c "import json; d=json.load(open('$hfile')); print(d['clippy']['warnings'])" 2>/dev/null || echo "0")
    cc=$(python3 -c "import json; d=json.load(open('$hfile')); print(len(d.get('cognitive_complexity',[])))" 2>/dev/null || echo "0")
    # json.dumps 로 출력해야 커버리지 미수집(None)이 JSON null 로 직렬화된다.
    # (print(None)은 파이썬 리터럴 "None"을 써서 metrics_history.json 전체가 무효 JSON이 됨
    #  → dashboard.html 의 추세/델타 카드가 조용히 비어 보이는 버그)
    cv=$(python3 -c "import json; d=json.load(open('$hfile')); print(json.dumps(d.get('coverage')))" 2>/dev/null || echo "null")
    fl=$(python3 -c "import json; d=json.load(open('$hfile')); print(len(d.get('file_lines',[])))" 2>/dev/null || echo "0")
    # [#2132 후속] 총량 지표 4종 (§5.1 v2.1) — cc_count(전체 함수 수, 기존 호환)와 별개.
    ccx=$(python3 -c "
import json
d=json.load(open('$hfile'))
v=[x['complexity'] for x in d.get('cognitive_complexity',[])]
print(sum(v), sum(sorted(v,reverse=True)[:20]), sum(x for x in v if x>25), len([x for x in v if x>25]))" 2>/dev/null || echo "0 0 0 0")
    read -r ccsum cctop20 ccosum ccocnt <<< "$ccx"
    lb=$(python3 -c "import json; d=json.load(open('$hfile')); print(d.get('label',''))" 2>/dev/null || echo "")
    if [ "$sfirst" = true ]; then sfirst=false; else SUMMARY+=","; fi
    SUMMARY+="{\"timestamp\":\"$ts\",\"label\":\"$lb\",\"tests_passed\":$tp,\"tests_failed\":$tf,\"clippy_warnings\":$cw,\"cc_count\":$cc,\"cc_sum\":$ccsum,\"cc_top20\":$cctop20,\"cc_over25_sum\":$ccosum,\"cc_over25\":$ccocnt,\"coverage\":$cv,\"file_count\":$fl}"
done
SUMMARY+="]"
echo "$SUMMARY" > "$OUTPUT_DIR/metrics_history.json"

# ── 대시보드 발행 (mydocs/metrics/ = tracked — 컨트리뷰터/클론 열람용) ──
PUBLISH_DIR="$PROJECT_DIR/mydocs/metrics"
mkdir -p "$PUBLISH_DIR"
cp "$OUTPUT_DIR/metrics.json" "$PUBLISH_DIR/metrics.json"
cp "$OUTPUT_DIR/metrics_history.json" "$PUBLISH_DIR/metrics_history.json"
if [ -f "$SCRIPT_DIR/dashboard.html" ]; then
    cp "$SCRIPT_DIR/dashboard.html" "$PUBLISH_DIR/dashboard.html"
    # 로컬 편의용 사본 (output/ 은 gitignore)
    cp "$SCRIPT_DIR/dashboard.html" "$OUTPUT_DIR/dashboard.html"
    echo ""
    echo "대시보드: $PUBLISH_DIR/dashboard.html (tracked — 커밋 대상)"
fi

# ── 스냅샷 보관 (--snapshot) ──
if [ "$SNAPSHOT" = true ]; then
    SNAP_DIR="$PROJECT_DIR/mydocs/metrics/$(date +%F)${SNAPSHOT_LABEL:+-$SNAPSHOT_LABEL}"
    if [ -d "$SNAP_DIR" ]; then
        echo "경고: $SNAP_DIR 이미 존재 — 덮어쓰지 않으려면 --snapshot <라벨> 로 구분할 것" >&2
    fi
    mkdir -p "$SNAP_DIR"
    cp "$OUTPUT_DIR/metrics.json" "$SNAP_DIR/metrics.json"
    # 추세 요약도 포함 — 없으면 dashboard.html 의 델타 카드/추세 차트가 비어 보인다.
    cp "$OUTPUT_DIR/metrics_history.json" "$SNAP_DIR/metrics_history.json" 2>/dev/null || true
    cp "$SCRIPT_DIR/dashboard.html" "$SNAP_DIR/dashboard.html"
    echo ""
    echo "스냅샷 보관: $SNAP_DIR (mydocs/metrics/README.md 목록도 갱신할 것)"
fi

echo ""
echo "=== 측정 완료 ==="
echo "결과: $OUTPUT_DIR/metrics.json"
echo ""
echo "요약:"
echo "  파일 수: $(echo "$FILE_LINES_JSON" | grep -o '"file"' | wc -l)"
echo "  Clippy 경고: $CLIPPY_WARNINGS"
CC_TOTAL=$(echo "$CC_JSON" | grep -o '"complexity"' | wc -l)
CC_OVER25=$(echo "$CC_JSON" | grep -oE '"complexity":[0-9]+' | awk -F: '$2 > 25' | wc -l)
echo "  Cognitive Complexity > 25: ${CC_OVER25}개 함수 (측정 대상 전체 ${CC_TOTAL}개)"
echo "  테스트: $TEST_PASSED passed / $TEST_FAILED failed / $TEST_IGNORED ignored"
echo "  커버리지: $COVERAGE"
