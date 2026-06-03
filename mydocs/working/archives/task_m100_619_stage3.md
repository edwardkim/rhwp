# Stage 3 단계별 보고서 — Task #619

회귀 검증

- 브랜치: `local/task619`
- 이슈: https://github.com/edwardkim/rhwp/issues/619

## 1. cargo test / clippy

```
$ cargo test --release --lib
... (전체 통과)
exit code 0

$ cargo clippy --release --all-targets -- -D warnings
... (warning 없음)
exit code 0
```

## 2. 회귀 가드 샘플 SVG 비교

변경 전 (HEAD~1 의 typeset.rs 만 빌드) vs 변경 후 (`local/task619`) 의 결과를 자동 비교.

### 2.1 LAYOUT_OVERFLOW 메시지 비교

```
$ for sample in 가드샘플들; do diff before/after_${sample}_overflow.txt; done
```

| 샘플 | overflow 메시지 (before / after) | 결과 |
|------|--------------------------------|------|
| `exam_eng.hwp` (Task #470 가드) | 3 / 3 | 동일 ✓ |
| `exam_kor.hwp` (issue #418 가드) | 1 / 1 | 동일 ✓ |
| `exam_science.hwp` (Task #568 가드) | 1 / 1 | 동일 ✓ |
| `exam_math.hwp` | 0 / 0 | 동일 ✓ |
| `exam_social.hwp` | 0 / 0 | 동일 ✓ |
| `hwp-multi-001.hwp` (Task #470 가드) | 1 / 1 | 동일 ✓ |
| `hwp-multi-002.hwp` | 0 / 0 | 동일 ✓ |
| `k-water-rfp.hwp` (Task #361 가드) | 2 / 2 | 동일 ✓ |
| `kps-ai.hwp` (Task #362 가드) | 7 / 7 | 동일 ✓ |
| `aift.hwp` (77p 다단 샘플) | 8 / 8 | 동일 ✓ |

→ **모든 회귀 가드 샘플의 overflow 메시지가 변경 전후 완전히 동일**. 본 변경에 의한 새로운 overflow 발생 없음.

### 2.2 페이지 분포 비교

```
$ for sample in 가드샘플들; do diff before/after_${sample}_pages.txt; done
```

| 샘플 | 페이지/단 분포 |
|------|------|
| `exam_eng.hwp` | 동일 ✓ |
| `exam_kor.hwp` | 동일 ✓ |
| `exam_science.hwp` | 동일 ✓ |
| `exam_math.hwp` | 동일 ✓ |
| `exam_social.hwp` | 동일 ✓ |
| `hwp-multi-001.hwp` | 동일 ✓ |
| `hwp-multi-002.hwp` | 동일 ✓ |
| `k-water-rfp.hwp` | 동일 ✓ |
| `kps-ai.hwp` | 동일 ✓ |

→ **모든 회귀 가드 샘플의 페이지/단 분포 완전 동일**. 본 변경은 다단 paragraph 내부 vpos-reset (line>0 + vertical_pos==0) 케이스 한정으로 동작하며, 가드 샘플들에는 해당 패턴이 없거나 영향 없음.

### 2.3 광역 다단 샘플

| 샘플 | pages | overflow |
|------|-------|----------|
| `2010-01-06.hwp` | 6 | 0 |
| `biz_plan.hwp` | 6 | 0 |
| `atop-equation-01.hwp` | 1 | 0 |
| `aift.hwp` | 77 | 8 (변경 전과 동일) |

추가 회귀 없음.

## 3. 결론

- cargo test / clippy 통과.
- 회귀 가드 샘플 10개 + 광역 샘플 4개 모두 변경 전후 동일한 결과.
- 본 변경의 영향 범위는 **다단 + 문단 내부 line.vertical_pos==0 케이스** 한정으로 검증됨.

## 4. 다음 단계

Stage 4 (최종 결과 보고서 + orders 갱신 + 머지 준비) 진행.
