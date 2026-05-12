# Task m100 #857 Stage 3 완료 보고서

> Issue: [#857](https://github.com/edwardkim/rhwp/issues/857)
> Stage 목표: 회귀 sweep + 시각 회귀 + 수동 E2E + 최종 보고서
> 작성일: 2026-05-12

## 1. 자동 회귀 검증

### 1.1 전체 cargo test (debug)
```
$ cargo test 2>&1 | tee /tmp/task857_full_test.log | tail
test result: ok. 1232 passed; 0 failed; 2 ignored (unit tests)
test result: ok (35 integration test suites)
```
**0 fail, 1232 unit tests + 35 test suites 모두 PASS**

### 1.2 핵심 관련 테스트 개별 확인
| 테스트 | 결과 |
|---|---|
| `tests/issue_717_table_cell_hit_test.rs` | 3 passed (Task #717 회귀 보장 유지) |
| `tests/issue_630.rs` | 1 passed |
| `tests/issue_nested_table_border.rs` | 1 passed |
| `tests/issue_table_vpos_01_page5_cell_hit_test.rs` | **13 passed** (본 RED, 모두 GREEN) |

### 1.3 clippy
```
$ cargo clippy --tests
```
- 본 변경 (cursor_rect.rs:643-666, 테스트 파일) 관련 새 warning 없음
- 기존 56 warning 은 다른 파일들 무관 사안

## 2. 시각 회귀 검증

### 2.1 page 5 (table-vpos-01.hwp) SVG diff
```
$ diff -q /tmp/tvp01-p5/table-vpos-01_005.svg /tmp/svg-after/table-vpos-01_005.svg
$ echo $?
0
```
**byte-identical** — 시각 회귀 없음. fix 가 hit-test 분기만 변경하고 렌더링 단은 미터치임을 입증.

### 2.2 추가 페이지/문서 SVG 생성 정상 확인
- `samples/table-vpos-01.hwp` page 4 (page index 3): 정상 export
- `samples/exam_social.hwp` page 1 (page index 0): 정상 export

## 3. 수동 E2E 검증 (rhwp-studio)

### 3.1 WASM 재빌드
```
docker compose --env-file .env.docker run --rm wasm
→ Finished `release` profile [optimized] target(s) in 52.59s
→ pkg/rhwp_bg.wasm 4.35 MB (2026-05-12 22:57 갱신)
```

### 3.2 작업지시자 시연 확인
- localhost:7700 에서 `samples/table-vpos-01.hwp` 5쪽 로드
- 작업지시자 직접 확인: c=2 column 본문 셀("포용과 균형의 기본사회 구현" 등) 클릭 시 **정상 동작 확인**

## 4. Fix 요약

### 4.1 수정 위치
`src/document_core/queries/cursor_rect.rs:643-666` (1차 bbox 매칭)

### 4.2 변경 내용 (요약)
- **Before** (first-match): `if hit_cell.is_none() { hit_cell = Some(...); }`
- **After** (min area best-match): 매칭된 cell-context TextRun 중 bbox 면적 최소 선택
- Task #717 의 cell_bboxes selection (L671-675) 과 동일 패턴

### 4.3 변경 영향
| | 변경 영향 |
|---|---|
| 셀 vs 본문 우선순위 | 동일 (셀 우선) |
| 셀 후보 단일 | 동일 |
| 셀 후보 복수 (중첩 표) | **개선 — 가장 specific 셀 선택** |
| 본문 TextRun | 동일 (first-match) |
| 코드 일관성 | L587-588 / L671-675 / L680 모두 best-match → 본 분기만 first-match 였던 정책 차이 해소 |

## 5. 사용자 증상 해소 확인

| 사용자 관찰 (Issue #857) | Stage 2 fix 후 |
|---|---|
| c=2 본문 "포용과 균형의 기본사회 구현" 클릭 시 셀 진입 못함 | **정상 진입** |
| 글자 입력 시 "4 공공 AX" 행 배경에 silent 삽입 | **클릭한 inner 셀에 정상 삽입** |
| 콘솔 에러 없는 silent misroute | **misroute 해소** |
| c=0 라벨 셀 회귀 위험 | **회귀 없음** (4개 케이스 모두 PASS) |

## 6. Git Tree 상태

```
local/task857 ← b10a83f0 Stage 2 보고서
              ← 1135c028 Stage 2 (GREEN, closes #857)
              ← 37e7b7b0 Stage 1 보고서
              ← 07168934 Stage 1 (RED)
local/devel   ← 2bd50a3a (= devel)
```

본 Stage 종료 후 추가 커밋:
- Stage 3 보고서 (본 파일)
- 최종 결과 보고서 (`mydocs/report/task_m100_857_report.md`)
- `mydocs/orders/20260512.md` 갱신

## 7. Stage 3 종료 후 처리 (작업지시자 승인 후)

1. `local/task857` → `local/devel` merge (`--no-ff`)
2. `local/devel` → `devel` merge + push (`origin/devel`)
3. Issue #857 자동 close (Stage 2 커밋의 `closes #857` 으로)

## 8. 잔존 위험·미해결

- **HWPX 변환** (`samples/table-vpos-01.hwpx`): 미확인. 별도 task 권장.
- **#850 (exam_social/exam_science 성명 칸)**: 본 fix 와 별개 메커니즘 → 본 fix 가 #850 해결 영향 미지수. 별도 처리 필요.
- 다른 first-match 분기 (L592-641 inline shape) — 본 fix 영향 없음.

## 9. 작업지시자 승인 요청

Stage 3 완료. **최종 결과보고서 작성 + orders 갱신 + merge 단계 진행 승인** 부탁드립니다.
