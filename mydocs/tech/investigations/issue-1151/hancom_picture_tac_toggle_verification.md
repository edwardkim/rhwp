---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1151/README.md
last_verified: 2026-07-16
---

# 한컴 picture "글자처럼 취급" 토글 — 가설 검증 protocol

대상 task: [Task #1151 v2](../../../plans/archives/task_m100_1151_v2.md) · 1차 분석: [hancom_picture_tac_toggle.md](hancom_picture_tac_toggle.md) · 1차 산출물: `samples/pic-in-table-with-toggle.hwp`

## 1. 목적

1차 산출물 dump 만으로는 한컴이 셀 위 floating picture 를 "글자처럼 취급" 으로 토글할 때 picture 가 어디로 가는지 (outer paragraph 의 sibling? 새 paragraph 로 분리? 셀 안?) 단정할 수 없다. fix 정책 (`plan §3-2`) 의 두 갈래 (B-i / B-ii) 중 어느 쪽이 한컴 정합인지 확정하기 위해, 사용자가 한컴에서 **명세된 절차**대로 시나리오 hwp 를 만들어 공유하고 rhwp 가 dump 로 비교한다.

## 2. 가설 매트릭스

| 가설 | 토글 후 picture 의 위치 | LINE_SEG.line_height | 한컴이 그렇게 동작한다면 plan §3-2 의 fix 정책 |
|------|-------------------------|-----------------------|------------------------------------------|
| **H1 (B-i)** | outer paragraph 의 sibling control 그대로 (위치 불변) | outer paragraph 의 ls 가 picture height 로 갱신 | `set_picture_properties_native` 가 outer paragraph 에 `attach_picture_inline` 호출. picture 위치 그대로. |
| **H2 (B-ii)** | 새 paragraph 가 outer 다음에 추가되고 picture 가 그쪽으로 이동 | 새 paragraph 의 ls 가 picture height 로 설정 | `set_picture_properties_native` 가 (a) picture 를 outer 에서 제거 → (b) 새 paragraph 를 section.paragraphs 에 삽입 → (c) 새 paragraph 의 controls 에 picture push → (d) `attach_picture_inline` 호출. |
| **H3 (A)** | 대상 셀의 첫 paragraph 안 inline 으로 이동 | 셀 paragraph 의 ls 가 picture height 로 설정 | (사용자 직접 검증 2026-05-29 에서 이미 기각됨) |
| **H4 (기타)** | 위 셋 모두에 해당하지 않는 패턴 | — | 1차 분석으로 돌아가 새 가설 수립 + 추가 검증. |

## 3. 검증 시나리오 — 사용자 한컴 작업 절차

각 시나리오는 **빈 신규 문서에서 시작**한다. 매번 신규 문서로 시작해야 다른 시나리오의 잔여 control 이 섞이지 않는다. 한컴 2022 (Windows) 기준.

### Scenario A — 1×1 표 + 작은 picture 의 토글

목적: 가장 단순한 케이스. picture 가 셀 크기보다 충분히 작은 경우의 한컴 동작 확정.

1. 한컴 2022 실행, 신규 빈 문서.
2. 메뉴 → 입력 → 표 → 1행 1열 선택 (또는 단축키 Ctrl+N, T → 1×1) → 셀 크기는 기본값.
3. 셀 안에 마우스 커서 두기 (또는 셀 클릭).
4. 메뉴 → 입력 → 그림 → 임의 작은 이미지 파일 선택 (대략 셀의 1/3 이하 크기 권장. 예: 가로 약 30mm 의 사진).
5. 그림이 셀 영역에 떠 있는 상태 (floating, tac=false) 인지 시각 확인.
6. **이 상태로 한 번 저장**: `samples/tac-verify/scenario-A-before.hwp`.
7. 그림을 한 번 클릭 (그림 속성 선택 모드) → 우클릭 → "개체 속성" → 그림 속성 대화상자.
8. **"글자처럼 취급" 체크박스 ON** → "설정". (그 외 속성은 손대지 않는다.)
9. 다른 곳 클릭하여 그림 속성 모드 해제.
10. **현재 상태로 저장**: `samples/tac-verify/scenario-A-after.hwp`.

### Scenario B — 1×1 표 + 큰 picture 의 토글

목적: picture 가 셀 크기보다 큰 경우. picture 크기에 따라 한컴 동작이 분기되는지 확인.

1~10 단계는 Scenario A 와 동일. 단:
- 4단계의 이미지를 **셀보다 큰 크기**로 (예: 가로 약 100mm).
- 저장 파일명: `scenario-B-before.hwp` / `scenario-B-after.hwp`.

### Scenario C — 3×3 표 중 (1,1) 셀의 picture 토글

목적: 대상 셀이 표의 중앙/내부 셀일 때의 한컴 동작. (가장자리 셀과 다른 처리가 있을 가능성 검증.)

1. 신규 빈 문서.
2. 표 → 3행 3열 → 기본 크기.
3. (1,1) 셀 = 가운데 셀에 커서.
4. 그림 → 임의 작은 이미지 (셀의 1/2 이하 권장).
5. floating 상태 저장: `scenario-C-before.hwp`.
6. 그림 우클릭 → 개체 속성 → "글자처럼 취급" ON → 저장: `scenario-C-after.hwp`.

### Scenario D — 셀 외부 본문 floating picture 의 토글 (대조군)

목적: 셀과 무관한 본문 floating picture 의 토글 동작과 셀 케이스 (A/B/C) 의 결과를 대조. 본문 floating 도 같은 path 인지 확인 (한컴이 셀/본문을 구분하지 않는지).

1. 신규 빈 문서. 표 만들지 않는다.
2. 본문에 그림 삽입 (어울림/floating 으로 만들어지는 기본 상태).
3. floating 저장: `scenario-D-before.hwp`.
4. 그림 → 개체 속성 → "글자처럼 취급" ON → 저장: `scenario-D-after.hwp`.

## 4. 저장 위치와 명명

- 폴더: `samples/tac-verify/` (신규 생성). 검증 종료 후 archive 또는 삭제 결정.
- 파일명: `scenario-{A,B,C,D}-{before,after}.hwp` 8개.
- 한컴 PDF 저장 (선택, 시각 비교용): 같은 폴더 `scenario-{A,B,C,D}-{before,after}.pdf`.

## 5. rhwp 가 확인할 항목 (dump 비교)

각 시나리오의 before/after 한 쌍에 대해:

```bash
./target/debug/rhwp dump samples/tac-verify/scenario-A-before.hwp > /tmp/sa-before.txt
./target/debug/rhwp dump samples/tac-verify/scenario-A-after.hwp  > /tmp/sa-after.txt
diff /tmp/sa-before.txt /tmp/sa-after.txt
```

비교 핵심 필드:

| 필드 | before 기댓값 | after 의 확인 사항 |
|------|---------------|---------------------|
| Picture 의 paragraph 위치 | 셀 paragraph 의 controls 또는 outer paragraph 의 sibling | **동일 paragraph 인가? 새 paragraph 로 이동했나? 셀 안으로 이동했나?** |
| `treat_as_char` | false | true |
| `horizontal_offset` / `vertical_offset` | 0이 아닌 값 (floating 좌표) | 0 으로 클리어되는가? |
| `horz_rel_to` / `vert_rel_to` | Page / Paper | Para 로 바뀌는가? |
| `text_wrap` | Square (또는 다른 floating wrap) | 변화 있는가? 무시되는가? |
| 부모 paragraph 의 LINE_SEG[0].line_height | floating 이전의 기본값 (예: 1000) | picture height 로 갱신되는가? |
| 부모 paragraph 의 `text_len` / `char_offsets` | 0 (또는 기존) | sentinel char 추가로 변화 있는가? |
| 새 paragraph 가 section.paragraphs 에 추가되었는가 | — | H2 검증. paragraph count 변화. |

## 6. 결과 매핑 표

dump 비교 결과에 따른 가설 매칭:

| 관찰 결과 | 매칭 가설 | fix 정책 |
|----------|----------|----------|
| picture 가 같은 paragraph 의 같은 control index 에 남고 부모 ls.lh 가 picture height 로 갱신 | H1 (B-i) | plan §3-2 그대로 진행 |
| section.paragraphs 의 paragraph count 가 +1 늘고, picture 가 새 paragraph 로 이동, 새 paragraph ls.lh = picture height | H2 (B-ii) | plan §3-2 재작성 — 새 paragraph 삽입 helper 추가 |
| picture 가 대상 셀의 첫 paragraph 안 controls 로 이동, 셀 paragraph ls.lh = picture height | H3 (A) | (가능성 낮음 — 2026-05-29 사용자 직접 검증으로 기각) plan 전면 재설계 |
| 시나리오 A/B/C 가 서로 다른 가설로 분류 | (C 분기) | 한컴이 picture 크기/셀 위치별 분기 사용. plan §3-2 에 동일 분기 추가. |
| 위 어느 것도 아님 | H4 | 추가 검증 시나리오 필요. plan 재검토. |

## 7. 다음 단계

1. 사용자가 본 protocol 의 Scenario A~D 를 한컴에서 수행하고 `samples/tac-verify/` 에 8개 hwp (+ 선택 PDF) 공유.
2. rhwp 가 위 §5/§6 표에 따라 dump 비교하여 가설 확정.
3. 확정된 가설에 따라:
   - 1차 분석 `hancom_picture_tac_toggle.md` 갱신 (raw 데이터 + 결론).
   - plan `enchanted-painting-pascal.md` §3-2 fix 정책 (또는 §3-2 재작성) 확정.
   - 수행계획서 `task_m100_1151_v2.md` 분기 (B-i/B-ii) 동기.
   - 구현계획서 `task_m100_1151_v2_impl.md` 작성 → 작업지시자 승인 → Stage 1-c 코드 작업.

## 8. 시나리오 수행 부담을 줄이는 경량 옵션

위 4개 시나리오 (8개 hwp) 가 부담스러우면 **최소 2개 시나리오 (A + D)** 만 우선 수행해도 가설 확정에 충분할 수 있다. C 의 "중앙 셀" 분기 가능성은 낮으므로 A 가 H1 또는 H2 로 명확히 분류되고 D 도 같은 분류이면 그 결과로 plan 확정. 결과가 모호하거나 다르면 B/C 추가.
