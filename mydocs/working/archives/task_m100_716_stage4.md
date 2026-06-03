# Task #716 Stage 4 (회귀) 완료 보고서

**Issue**: [#716](https://github.com/edwardkim/rhwp/issues/716)
**Stage**: 4 — 회귀 검증
**작성일**: 2026-05-08

---

## cargo test --release 결과

```
test result: ok. 1157 passed; 0 failed; 2 ignored;  (lib unit tests)
test result: ok. 25 passed; 0 failed;
test result: ok. 14 passed; 0 failed;
test result: ok.  9 passed; 0 failed;
test result: ok.  8 passed; 0 failed;
test result: ok.  7 passed; 0 failed;  (svg_snapshot — golden SVG 회귀)
test result: ok.  4 passed; 0 failed;
test result: ok.  3 passed; 0 failed;
test result: ok.  2 passed; 0 failed;  (page_number_propagation)
test result: ok.  1 passed;            (issue_716 — 본 결함)
test result: ok.  1 passed;            (issue_713)
test result: ok.  1 passed;            (issue_712)
... (각 issue_*, snapshot 모두 0 failed)
```

**총 1500+ 테스트 0 failed**. 회귀 0건 확인.

### 골든 SVG 회귀 (svg_snapshot)

7개 골든 SVG 모두 PASS:
- `table_text_page_0`
- `issue_157_page_1`
- `issue_267_ktx_toc_page`
- `form_002_page_0`
- `render_is_deterministic_within_process`
- `issue_147_aift_page3`
- `issue_617_exam_kor_page5`

→ Task #716 의 fix_overlay 빈 paragraph skip 적용으로 골든 SVG 6개 샘플 (form-002, issue-147, issue-157, issue-267, issue-617, table-text) 의 시각 출력에 변화 없음.

## 본 샘플(hongbo) 시각 검증

`/tmp/hongbo-fixed/20250130-hongbo_001.svg` 생성. RED test (`issue_716`) PASS:

```
[issue_716] page 0 body=[x=75.59 y=94.47 w=642.53 h=933.57 bottom=1028.04]
            text_lines=38 max_bottom=1028.19 overflow=+0.15
test issue_716_page1_last_text_line_within_body ... ok
```

→ TextLine bbox 가 컬럼 하단 +0.15 px (허용 0.5 px 이내). Stage 1 측정의 +20.15 px 에서 +0.15 px 로 정정.

### stderr 출력

| 메시지 | Stage 1 (RED) | Stage 4 |
|--------|---------------|---------|
| `LAYOUT_OVERFLOW_DRAW: pi=15 line=2 overflow=20.1px` | 발생 | **0건** ✓ |
| `LAYOUT_OVERFLOW: pi=15 PartialParagraph overflow=31.3px` | 발생 | `overflow=11.3px` (잔존) |

`LAYOUT_OVERFLOW_DRAW` (실제 시각 cropping) 0건 — 본 결함 해소.

`LAYOUT_OVERFLOW` 11.3 px 잔존은 마지막 줄 trailing line_spacing 의 y_offset 누적으로, Task #452 의 의도된 동작 (pagination engine current_height 정합). 시각적 영향 없음.

## 사전 회귀 점검 (스팟 체크)

샘플 4개 LAYOUT_OVERFLOW_DRAW 카운트:

```
samples/2022년 국립국어원 업무계획.hwp : 0건
samples/exam_math.hwp                   : 0건
```

→ 본 정정 후에도 LAYOUT_OVERFLOW_DRAW 발생 0건 (회귀 0). 광범위 검증은 Stage 5.

---

## 다음 단계 (Stage 5 — 광범위)

1. 169개 샘플 (HWP + HWPX) 전수 LAYOUT_OVERFLOW_DRAW 카운트 (before / after)
2. 페이지 수 변동 0 확인
3. 음수 ls 보유 / TAC 표 다수 보유 샘플 시각 검증
4. 보고서 + 커밋

## 승인 요청

Stage 4 회귀 검증 완료. 0 failed, 골든 SVG 회귀 0. Stage 5 (광범위 검증) 진입 승인 요청.
