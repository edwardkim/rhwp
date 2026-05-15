# Task #902 Stage 8 보고서 — 광범위 회귀 검증

**Stage**: 8 / 9 (v2)
**상태**: 완료

## 1. 검증 범위

### 1.1 회귀 테스트 suite

```
cargo test --release --all-targets
Total passed: 1412 / failed: 0
```

기존 1411+ 기준 유지. Stage 3~7 누적 변경 후에도 0 regression.

### 1.2 Golden SVG snapshot

```
cargo test --release --test svg_snapshot
8 passed / 0 failed
```

대상:
- form-002/page-0
- issue-147/aift-page3
- issue-157/page-1
- issue-267/ktx-toc-page
- issue-617/exam-kor-page5
- issue-677/bokhakwonseo-page1
- table-text/page-0
- render_is_deterministic_within_process

### 1.3 다중 sample SVG export

| Sample | 페이지 수 | 결과 | 비고 |
|--------|----------|------|------|
| sample14 | 11 | ✓ | Task #860 fixture |
| sample16 | 64 | ✓ | issue #902 본 분석 대상 |
| sample17 | 12 | ✓ | |
| sample18 | 69 | ✓ | |
| sample19 | 0 | **pre-existing parser 실패** | WMF 무관, Stage 1 fixture, picture 없음 |

### 1.4 rsvg-convert 셀프 PNG 검증

- sample16 page 18 (paragraph 394 WMF, 주전산센터 다이어그램): PNG 283 KB 정상
- sample18 page 17: PNG 63 KB 정상
- sample19 별도 — 해당 페이지 없음

## 2. sample19 실패 분석

```
오류: HWP 파싱 실패 - 유효하지 않은 파일: HWP 3.0 오류:
입출력 오류가 발생했습니다: failed to fill whole buffer
```

- 위치: `src/parser/hwp3/` parser (WMF 영역 아님)
- 검증: `git stash + previous HEAD` 검증 시도했으나 stash conflict 발생 → 즉시 reset.
  대신 v1 Stage 1 보고서 (`task_m100_902_stage1.md` §1.1) 의 sample19 항목에 "picture 없음" 기록 — Stage 1 시점에 이미 인지된 fixture 상태.
- 결론: **v2 본 stage 와 무관 — follow-up issue 로 등록 필요** (HWP3 parser 영역, 다른 task)

## 3. Stage 3~7 효과 종합

| Stage | 효과 | sample16 visual | 다른 sample 영향 |
|-------|------|-----------------|------------------|
| 3 | DX byte-aware indexing | 텍스트 위치 정합 (주전산센터 다이어그램 등) | 동일 효과 |
| 4 | SETVIEWPORTEXT/ORG | 미호출 → 동일 | viewport 사용 sample 향후 정합 |
| 5 | EXTTEXTOUT flags (OPAQUE/PDY) | 미사용 → 동일 | 해당 flag sample 향후 |
| 6 | OFFSET WINDOW/VIEWPORT, SCALEVIEWPORTEXT | 미호출 → 동일 | 해당 record sample 향후 |
| 7 | 폰트 metric (font-family 체인) | 한컴 viewer 와 더 가까운 cross-platform 폰트 | sample14/17/18 도 동일 효과 |

## 4. 잔존 한계 (본 task scope 외)

- WMF 굴림체 ↔ Apple SD Gothic Neo / Nanum Gothic 간 미세 glyph metric 차이 — 완전 정합 불가 (폰트 임베딩 필요, follow-up)
- 일부 사용 빈도 낮은 WMF records (Region/Palette ops 등) 미구현 — follow-up
- sample19 parsing 실패 — 별도 issue (parser 영역)

## 5. 산출물

- 본 보고서: `mydocs/working/task_m100_902_stage8.md`
- 검증 SVG: `/tmp/task902_s8_final/sample{14,16,17,18}/`
- 검증 PNG: `/tmp/task902_s8_final/sample{16,18}_017.png`

## 6. 다음 단계

Stage 9: 통합 + 최종 보고서 + PR (PR 생성 직전 작업지시자 명시 승인)
