# 단계 6 보고서 — Task #356 추가 수정 (페이지 3 잔존 2줄 제거)

- **단계**: 6 (단계 5 종료 후 작업지시자 추가 피드백 반영)
- **브랜치**: `local/task356`

## 1. 추가 발견 증상

작업지시자 피드백:
> 2022년 국립국어원 업무계획.hwp — 3 page 하단에 2줄이 더 그려짐 (국어사전 정보...) PDF 는 없음.

PDF page 3 ends with table "전문가 감수, 사전 반영", page 4 starts with "ㅇ 국어사전 정보보완심의회 운영을 통한 국어사전 정보 수정 및 보완 ..." (= pi=39).

본 fix 전(단계 5 결과):
- page 3 의 마지막 항목이 pi=39 (FullParagraph) 로 남음
- HWP 권위값: pi=39 ls[1].vpos+lh+ls = 70761 HU > body 70014 HU (10 px 초과)
- 그러나 px 누적 평가는 fits → 그대로 page 3 에 배치

## 2. 원인 분석

`dump-pages` 비교:
```
페이지 3 (단계 5): used=792.6px, hwp_used≈933.1px, diff=-140.5px
```

px 누적 (792.6) 이 HWP 권위값 누적 (933.1) 보다 140 px 작음. 페이지 3 의 TAC 표 4개 (pi=20, 22, 26, 38) 측정 결과가 HWP 의 실제 vpos 진행보다 작아 *cumulative drift* 발생. drift 누적분 만큼 pi=39 가 fits 판정을 받아 페이지 3 에 들어감.

## 3. 해결

세 가지 보정을 typeset 엔진에 추가:

### (a) `page_vpos_base` 도입 + drift 보정 (TypesetState)

페이지 첫 항목의 first_vpos 를 base 로 두고, 후속 문단의 `(first_vpos - base) px` 가 누적 `current_height` 보다 크면 끌어올린다. TAC 표 누적 drift 를 HWP 권위값으로 반복 보정.

### (b) HWP 권위값 overflow → 강제 페이지 분기

각 문단의 마지막 LINE_SEG `vpos+lh+ls` 가 body 높이를 초과하면 (본 샘플 pi=39: 70761 > 70014), 다음 페이지로 advance. 단, 다음 조건 모두 만족 시에만:
- 페이지 첫 항목 아님 (`current_items` 비어있지 않음)
- px 누적 평가는 fits (= drift 의심)
- 새 페이지에서 full para 가 들어감 (= keep-together 보장)

### (c) 인접 문단 vpos 리셋 검사 보강

- **suppress_next_inter_para_advance 플래그**: HWP overflow 로 force_advance 한 직후 다음 문단의 inter-para vpos 리셋 검사를 한 번 스킵. force_advanced 된 문단이 새 페이지 첫 항목이 되었으므로, HWP 의 다음 원래 문단이 vpos=작은값 으로 시작해도 그것은 정상 진행.
- **prev_was_partial 가드**: 이전 항목이 PartialParagraph (페이지 분할된 문단) 이면 prev.first_vpos 가 다른 페이지 좌표라 inter-para 검사가 잘못된 advance 를 유발 (예: hongbo.hwp pi=15 split → pi=16 회귀 발생). PartialParagraph 직후에는 inter-para 검사 스킵.

## 4. 변경 파일

`src/renderer/typeset.rs`:

| 변경점 | 라인 (대략) |
|--------|------|
| TypesetState 에 `page_vpos_base`, `suppress_next_inter_para_advance` 필드 추가 | 122~140 |
| 초기화 / `reset_for_new_page` 에서 page_vpos_base 클리어 | 152~165 |
| 메인 루프: items[0] 으로부터 page_vpos_base 유도 | 412~432 |
| 메인 루프: prev_was_partial / suppress 플래그 가드 | 391~410 |
| `typeset_paragraph`: HWP overflow 검증 + drift 보정 | 645~691 |

## 5. 검증

### 본 샘플
| 항목 | 단계 5 | 단계 6 |
|------|--------|--------|
| 페이지 수 | 35 (PDF 37 미달) | **37** (PDF 일치 ✅) |
| 페이지 3 마지막 | pi=39 (잔존) | pi=38 (Table) ✅ |
| 페이지 4 시작 | pi=40 | pi=39 ✅ |
| LAYOUT_OVERFLOW | 0 | **0** ✅ |

### 다중 샘플 회귀

| 샘플 | 베이스라인 | 단계 5 | 단계 6 |
|------|-----------|--------|--------|
| `2022년 국립국어원 업무계획.hwp` | 35p / 5+ | 35p / 0 | **37p / 0** ✅ |
| `aift.hwp` | 74p / 30 | 86p / 16 | **83p / 4** (overflow 87% 감소) |
| `exam_eng.hwp` (다단) | 8p / 0 | 8p / 0 | **8p / 0** ✅ |
| `exam_math.hwp` | 20p / 0 | 20p / 0 | **20p / 0** ✅ |
| `2010-01-06.hwp` | 6p / 0 | 6p / 0 | 7p / 0 (overflow 0 유지, 페이지 분배 미세 변화) |

### 단위 + 통합 + 골든

```
cargo test --release
test result: ok. 1014 passed; 0 failed; 1 ignored
+ 14 + 25 + 6 + 1 + 1 PASS, 0 FAIL

cargo test --release --test svg_snapshot
6 passed; 0 failed (form-002, issue-147/157/267, table-text, 결정성)
```

회귀 추적: hongbo.hwp 의 `test_task78_rectangle_textbox_inline_images` 가 처음에는 회귀(images 페이지 이동)했으나 `prev_was_partial` 가드 추가로 통과 복구.

## 6. 다음 단계

최종 보고서(report) 갱신 후 머지 준비.
