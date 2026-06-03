# Task #722 Stage 7 단계별 보고서 — 광범위 회귀 sweep + 최종 검증

## 1. 광범위 페이지네이션 sweep

baseline (Stage 6 미적용) vs Stage 6 적용 후 209 fixture 페이지 수 비교:

```bash
find samples -type f \( -name "*.hwp" -o -name "*.hwpx" \) | sort | while read f; do
  pages=$(./target/release/rhwp dump-pages "$f" 2>&1 | grep -oE '[0-9]+페이지' | head -1 | grep -oE '[0-9]+')
  echo "$f|$pages"
done > /tmp/task722_${env}_pages.txt
```

| 환경 | fixture 수 | 페이지 수 합계 | DIFF |
|------|-----------|---------------|------|
| baseline | 209 | (동일) | - |
| Stage 6 | 209 | (동일) | **0** |

**전체 fixture 페이지 수 차이 0 — 회귀 0** 확인.

## 2. 결정적 검증 (release 모드)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | **1165 passed; 0 failed** |
| `cargo test --release` (전체 binary tests) | **전체 GREEN** |
| `cargo clippy --release` | 신규 경고 0 |

svg_snapshot, issue_546, issue_554, issue_516, issue_505, issue_514 등 모든 release 테스트 통과.

## 3. 시각 판정 (rsvg-convert PNG, PDF 권위 자료 정합)

| 페이지 | paragraph | LINE_SEG | image | 결과 | PDF 정합 |
|--------|-----------|----------|-------|------|---------|
| 8 | 175 (multi-line wrap) | 2 | y_voff=18680, mr=852 | image 우측 wrap zone + gap 3mm | ✓ |
| 27 | 779 (caption) | 1, room=9720 | y_voff=15400, mr=852 | image 위 자유 영역 좌측 정렬 | ✓ |
| 48 | 1394 (single-line wrap) | 1, room=-12 | y_voff=5668, mr=852 | image 우측 wrap zone + gap 3mm | ✓ |

본 task #722 의 3 paragraph 모두 한컴 PDF 권위 자료 정합.

## 4. 회귀 위험 영역 정합 검증

- typeset.rs: 2 곳 (다음 paragraph register + anchor host self register) 변경 — `has_non_tac_pic_square` 발현 영역 한정
- pagination.rs: WrapAnchorRef 데이터 모델 1 필드 추가 (anchor_image_margin_right)
- paragraph_layout.rs: wrap_anchor 처리 영역 1 곳 변경 (cs/sw 보정)
- IR 무수정 (HWP3 파서 / Document model 무영향)
- Task #604 영역 보존
- 209 fixture 페이지 수 차이 0

## 5. 최종 결과 보고서 작성 + 오늘할일 갱신 + closes #722

Stage 7 통과. 최종 결과 보고서 (`mydocs/report/task_m100_722_report.md`) 작성 + 오늘할일 (`mydocs/orders/20260508.md`) 갱신 + commit + close issue 진행.

### 승인 요청

최종 결과 보고서 + 오늘할일 갱신 + 커밋 + closes #722 진행 승인 요청.
