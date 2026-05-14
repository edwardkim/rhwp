# Task #894 최종 결과 보고서 — Task #877 잔존 통합 (3 stages + 2 분리 task)

**이슈**: [edwardkim/rhwp#894](https://github.com/edwardkim/rhwp/issues/894)
**브랜치**: `local/task894` (base: `local/devel @ c2955b5`)
**마일스톤**: v1.0.0 (M100)
**선행 task**: #877 (closed via PR #890 — 메인테이너 머지 시점에 close)
**분리 task**: #895 (HWPX 페이지 수 inflate), #896 (sample16 페이지 18 추가 시각)
**기간**: 2026-05-14

## 1. 개요

[Task #877](https://github.com/edwardkim/rhwp/issues/877) (hwp3-sample16.hwp WASM 로드 + paragraph alignment + 시각 정합) 완료 후 분석된 잔존 3건 (HWP3 페이지 외곽선 좌표 / paragraph multi-line picture 중복 / HWP5 변환본 페이지 수 inflate) 의 통합 처리.

## 2. Scope 변경 이력

| Scope | 변경 | 사유 |
|-------|------|------|
| 항목 C (HWP5 변환본 inflate) | 제거 | #877 진행 중 이미 해결 — HWP5 정합 62/62 |
| 항목 C' (HWPX 변환본 inflate 72→62) | 대체 | Stage 1 진단 중 발견 |
| 항목 D (CLAUDE.md c2955b5) | 추가 | PR #890 미포함, task894 PR 로 메인테이너 전달 |
| **Stage 1 (C')** | **#895 로 분리** | Root cause 가 한컴 HWPX 변환기 본질적 한계 (lineseg vpos 0 reset 누락) |
| Stage 4 (◦ x 좌표 + WMF 텍스트) | **#896 으로 분리** | 영역이 다름 (paragraph_layout / WMF converter) |

## 3. 최종 결과

### 3.1 성과 요약

| Stage | 항목 | 결과 | 커밋 |
|-------|------|------|------|
| 1 (C') | HWPX self-closing run charPrIDRef 처리 | 정확성 보강 (페이지 수 inflate 는 #895 분리) | 55c6191 |
| **2 (B)** | **paragraph multi-line picture 중복 emit** | **image 3개 → 1개 ✅** | 5c177bd |
| **3 (A)** | **HWP3 페이지 외곽선 좌표** | **paper_based 정합, 페이지 번호 외곽선 안 ✅** | ddb7fa4 |
| D | CLAUDE.md 컨트리뷰터 워크플로우 보강 | base 자동 포함 | c2955b5 (devel) |

### 3.2 sample16 페이지 18 시각 정합 변화

| 항목 | 이전 | 이후 |
|------|------|------|
| paragraph 394 picture (WMF 다이어그램) | SVG image 3개 (중복) | **1개 ✅** |
| 페이지 외곽선 박스 (paragraph 2) | 페이지 번호 밖 | **페이지 번호 안 ✅** |

### 3.3 분리 task (잔존)

| Issue | 내용 | 분리 사유 |
|-------|------|----------|
| [#895](https://github.com/edwardkim/rhwp/issues/895) | HWPX 변환본 페이지 수 inflate (72→62) | Root cause 한컴 HWPX 변환기 lineseg vpos 0 reset 누락. 광범위 영향 + 회귀 점검 자료 부족 |
| [#896](https://github.com/edwardkim/rhwp/issues/896) | sample16 페이지 18 ◦ x 좌표 차이 + WMF 그림 안 텍스트 겹침 | paragraph_layout / WMF converter 영역. 본 task scope 외 |

## 4. Stage 별 결과

### 4.1 Stage 1 (항목 C') — HWPX 변환본 페이지 수 inflate

#### Fix 1 (`55c6191`) — HWPX self-closing `<hp:run charPrIDRef>` 처리

**진단**: 빈 paragraph 의 `<hp:run charPrIDRef="42"/>` self-closing element 가 HWPX 파서 `Event::Empty` 분기에 `b"run"` 처리 누락 → 빈 paragraph 의 char_shape 가 default (id=0) 로 잘못 설정.

**수정**: `src/parser/hwpx/section.rs` 의 `parse_paragraph` 의 `Event::Empty` 분기에 `b"run"` 처리 추가. paragraph 24 CharShape id: 0 → **42** 정확 인식.

**결과**:
- 정확성 보강 ✅
- sample16-hwp5.hwpx 페이지 수 (72) 미해결 — root cause 가 한컴 HWPX 변환기의 lineseg vpos 0 reset 누락 (별도 task #895)

#### Root Cause 확정 (별도 task 분리)

`typeset.rs:455~493` 의 vpos-reset trigger (`cv==0 && pv>5000`) 가 HWP5 의 페이지 break reset 신호를 사용. HWPX 의 lineseg vpos 는 누적값으로 reset 신호 손실 → trigger 발동 실패 → 페이지 break point 마다 1 페이지 누적 inflate.

상세: [`mydocs/working/task_m100_894_stage1.md`](../working/task_m100_894_stage1.md)

### 4.2 Stage 2 (항목 B) — paragraph multi-line picture SVG 중복 emit

#### Fix (`5c177bd`)

**진단**: paragraph 394 의 picture (WMF) 가 SVG 에 3 번 emit. ROOT CAUSE: HWP3 파서가 `char_offsets` 를 sequential `[0,1,2,3,4]` 만 push 하고 control marker 위치에 +8 gap 을 추가하지 않음 → `control_text_positions()` 의 갭 분석 실패 → 모든 control 이 fallback 으로 paragraph 끝으로 push → 마지막 line (`is_last_run=true`) 처리 시 3 control 모두 매치되어 picture 1개가 3 번 emit.

**수정**: `src/model/paragraph.rs` 의 `control_text_positions()` 함수의 fallback 강화 — 갭 분석으로 발견되지 않은 control 의 위치를 text 의 `\u{FFFC}` marker 위치로 매핑.

**결과**: SVG `<image>` 3개 → **1개** ✅. cargo test 1234 passed, sample 회귀 없음.

상세: [`mydocs/working/task_m100_894_stage2.md`](../working/task_m100_894_stage2.md)

### 4.3 Stage 3 (항목 A) — HWP3 페이지 외곽선 좌표 정합

#### Fix (`ddb7fa4`)

**진단**: Task #877 c8ba53b 의 `page_border_fill.attr=0` (body_based) 가 한컴 viewer PDF 출력과 불일치. 외곽선 박스가 paragraph 의 right margin 부근 텍스트 (페이지 번호) 를 포함하지 못함.

**수정**: `src/parser/hwp3/mod.rs` 의 `page_border_fill.attr = 1` (paper_based).

**결과**:
- 외곽선 x 범위: 80.3~713.4 → **18.93~774.77** px
- 페이지 번호 (x=728) 외곽선 안 ✅
- cargo test --all-targets **1355 passed**, 회귀 없음

상세: [`mydocs/working/task_m100_894_stage3.md`](../working/task_m100_894_stage3.md)

### 4.4 Stage 4 — sample16 페이지 18 추가 시각 차이 진단

작업지시자 추가 발견 (◦ 글머리 누락 + 그림 안 텍스트 겹침) 진단. 두 차이 모두 **본 task scope 외**:

| 차이 | 영역 | 결정 |
|------|------|------|
| ◦ x 좌표 차이 (paragraph 397~399) | paragraph_layout (rhwp 렌더러) | #896 분리 |
| WMF 그림 안 텍스트 겹침 | WMF converter | #896 분리 |

paragraph_layout 변경은 모든 paragraph 영향 (회귀 매우 높), WMF converter 는 별도 영역 + 회귀 점검 자료 부족.

## 5. 검증

### 5.1 cargo test

```
cargo test --release --all-targets: 1355 passed, 0 failed
```

### 5.2 HWP3/HWPX/HWP5 sample 페이지 수 회귀

| 샘플 | 페이지 수 | 회귀 |
|------|---------|------|
| hwp3-sample.hwp | 16 | — |
| hwp3-sample4.hwp | 36 | 없음 |
| hwp3-sample5.hwp | 64 | 없음 |
| hwp3-sample10.hwp | 763 | 없음 |
| hwp3-sample13.hwp | 3 | 없음 |
| hwp3-sample14.hwp | 11 | 없음 |
| hwp3-sample16.hwp | 64 | 없음 |
| 모든 HWPX 샘플 (10종) | 동일 | 없음 |

### 5.3 신규 검증 케이스

- sample16 페이지 18 paragraph 394 picture SVG `<image>` 개수: 1개 (이전 3개)
- sample16 페이지 2 외곽선 박스 / 페이지 번호: 정합

## 6. 변경 파일

### 6.1 소스

- `src/parser/hwpx/section.rs` (+12 lines) — Stage 1 Fix 1 (self-closing run charPrIDRef)
- `src/model/paragraph.rs` (+23 lines, -2 lines) — Stage 2 Fix (control_text_positions fallback)
- `src/parser/hwp3/mod.rs` (+4 lines, -1 line) — Stage 3 Fix (page_border_fill paper_based)

### 6.2 문서

- `mydocs/plans/task_m100_894.md` — 수행 계획서
- `mydocs/plans/task_m100_894_impl.md` — 구현 계획서
- `mydocs/working/task_m100_894_stage1.md` — Stage 1 진단 (Fix 1 + root cause 분리 결정)
- `mydocs/working/task_m100_894_stage2.md` — Stage 2 완료 (picture 중복 해소)
- `mydocs/working/task_m100_894_stage3.md` — Stage 3 완료 (page border paper_based)
- `mydocs/report/task_m100_894_report.md` — 본 최종 보고서

### 6.3 task 877 의 미포함 변경 (자동 포함)

- `CLAUDE.md` (c2955b5) — 컨트리뷰터 워크플로우 + 실수 회피 가이드 보강

## 7. 잔존 (후속 task 권장)

본 task 의 분리 사항:

| Issue | 내용 |
|-------|------|
| [#895](https://github.com/edwardkim/rhwp/issues/895) | HWPX 변환본 lineseg vpos 페이지 break reset 누락 — 페이지 수 inflate |
| [#896](https://github.com/edwardkim/rhwp/issues/896) | sample16 페이지 18 추가 시각 — ◦ x 좌표 + WMF 텍스트 |

## 8. 결론

본 task #894 의 핵심 목표 (Task #877 잔존 3건 처리) 중:
- **2/3 stages 완전 해소** (Stage 2 picture 중복, Stage 3 page border)
- **1/3 stage 정확성 보강 + 별도 task 분리** (Stage 1 의 self-closing run 처리 + 페이지 수 inflate root cause #895 분리)

추가로 Stage 4 의 새 발견 (paragraph_layout / WMF) 도 #896 으로 분리. 본 PR 의 변경은 cargo test 1355 passed + sample 6종 회귀 없음으로 검증.
