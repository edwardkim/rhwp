---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1773/README.md
last_verified: 2026-07-16
---

# #1773 원 인코딩 보존 설계 — 레코드 전용(무 컨트롤문자) 구역 컨트롤 왕복 계약

## ⚠ 재판별 (2026-07-03) — 관측 케이스의 실체는 HWP3

구현 착수 후 파일 매직 확인 결과, 알려진 TextRun ±1 케이스 3건(2912837=admrul_072,
big_hwp admrul_0644/0645)은 전부 **HWP3 (`HWP Document File V3.00`)** 이다.
즉 "HWP5 레거시 무-문자 인코딩"이 아니라 **HWP3 파스 IR → HWP5 직렬화 → HWP5 재파스**
경로다. 이때:

- A(HWP3 파스): 문단 0 controls=0, cc=9 — HWP3 에는 컨트롤 문자 개념 자체가 없음
- 변환: 저장 어댑터가 SectionDef 컨트롤을 삽입(insert_section_def_control — PAGE_DEF
  누락 방지용 **정당한 동작**) + 직렬화기가 제어 문자 합성 → cc 9→18
- B(재파스): text 는 동일하나 컨트롤·offset 이 붙음 → 조성(line/run 분할) 차이

따라서 이 3건의 근본 해소는 본 계약이 아니라 **"HWP3 파스 IR 과 HWP5 재파스 IR 의
컴포저 조성 불변성"** (같은 text 면 secd 컨트롤/offset 시프트와 무관하게 같은 분할)
이라는 별개 과제다. 아래 계약은 (코퍼스에서 아직 확인되지 않은) 진짜 HWP5
레코드-전용 인코딩 클래스에 대한 왕복 충실도 보강으로 유효하며 구현·테스트를 유지한다.

## 문제 (원 설계 — HWP5 레코드-전용 클래스)

HWP5 파일이 문단의 구역정의(secd)/단정의(cold) CTRL_HEADER 레코드를 가지면서
PARA_TEXT 에 대응 **제어 문자가 없는** 경우, 현 직렬화기 `serialize_para_text` 는
배치되지 않은 모든 컨트롤을 tail 루프에서 표준 확장 제어 문자(8 code unit)로
**무조건 합성**한다 → cc 변화 → 재파스 문자 스트림 변화 → 조성 차이.

## 계약 설계

**검출 (파서, `parse_paragraph`)**: `parse_para_text` 가 텍스트에서 만난 extended
컨트롤 문자 수(`anchored_ctrl_chars` — 0x0003 FIELD_BEGIN + is_extended_only_ctrl_char
대상)를 반환. 아래 전부 성립 시 `Paragraph.controls_without_ctrl_chars = true`:

- `has_para_text` (PARA_TEXT 레코드 존재)
- `!controls.is_empty()` (CTRL_HEADER 존재)
- `anchored_ctrl_chars == 0` (제어 문자 전무 — **all-or-nothing**)

부분 불일치(일부만 앵커)는 문자↔레코드 정렬이 모호하므로 **보존하지 않고** 종전
표준 합성에 맡긴다 (보수적 폴백, 동작 무변경).

**보존 (직렬화기, `serialize_para_text`)**: 플래그가 참이면 갭 루프·tail 루프의
컨트롤 문자 방출을 생략 (`emit_ctrl_chars=false`). CTRL_HEADER 레코드 자체는
`serialize_paragraph` 가 별도로 기록하므로 컨트롤은 보존된다. PARA_HEADER cc 는
직렬화된 PARA_TEXT code unit 수에서 재계산되므로(한컴 손상 판정 회피 로직 기존재)
자동으로 원본 값(9)에 수렴한다. control_mask 는 파스 원본 보존값 그대로.

**편집 정규화**: 문단 편집으로 컨트롤 구성이 변하면 표준 인코딩으로 복귀 —
`split_at`(자신·신규 문단)과 `merge_from` 에서 플래그를 false 로 리셋. 이 경로들은
char_count 를 "컨트롤당 8 cu" 기준으로 재계산하므로 보존 유지가 오히려 모순을 만든다.

## 영향 범위 분석

| 경로 | 영향 |
|------|------|
| HWPX 파스 → HWP 저장 | 무영향 — HWPX 파서는 플래그를 세우지 않음 (default false) |
| 네이티브 HWP5 표준 인코딩 파일 | 무영향 — anchored==controls 라 플래그 false |
| 레거시 무-문자 파일 재저장 | cc·문자 스트림 원본 보존 (본 수정의 목적) |
| FIELD_BEGIN/END | 플래그 문단은 정의상 field_ranges 가 비어 있음 (0x0003 문자가 없으므로) — 간섭 없음 |
| 편집(document_core) | split/merge 시 정규화. 그 외 컨트롤 삽입 커맨드는 HWPX/신규 문단 대상이라 플래그 false |
| 렌더러 | 무영향 — 렌더는 controls 벡터를 소비, 문자 스트림 인코딩과 무관 |

## 검증 결과 및 최종 판정 — **구현 철회, 설계 기록만 보존**

프로토타입 구현(모델 플래그 + 파서 검출 + 직렬화 분기 + 왕복 테스트 2종, lib 2070
통과)까지 완료 후 실증 검증에서 다음이 확정되어 **구현을 철회**한다:

1. **관측 케이스 전원 HWP3**: admrul_072/0644/0645 재검 — TextRun ±1 **잔존**.
   ir-diff 로 확인한 실제 변화는 "저장 어댑터의 SectionDef 컨트롤 삽입"(A controls=0
   → B controls=1, HWP3 파스 IR 에는 컨트롤이 아예 없음)이며, 이는 PAGE_DEF 보강을
   위한 정당한 변환 동작이다. 본 계약(파스된 컨트롤의 문자 합성 억제)의 적용 대상이
   아니다.
2. **참 클래스 코퍼스 부재**: big_hwp 2,487 + sample_hwp 298 = HWP5 전수 2,785 파일
   레코드 스캔(PARA_HEADER/PARA_TEXT/CTRL_HEADER 직접 판독) — "컨트롤 보유 + 제어
   문자 0" 문단 **0건**. HWP5 실파일에서 레코드-전용 인코딩은 확인되지 않았다.
3. 참 케이스가 없는 방어 코드는 유지비만 남기므로 철회하고, 본 문서로 계약 설계와
   기각 근거를 보존한다 (재발 시 이 설계를 그대로 착수 가능).

## 잔여 과제 (실제 근본 — 별개 트랙)

**HWP3 파스 IR ↔ HWP5 재파스 IR 의 컴포저 조성 불변성**: 같은 text 인데 컨트롤
유무/char_offsets 시프트(+8)에 따라 line/run 분할이 갈리는 지점을 특정해야 한다
(admrul_072 는 표 셀 내부 TextLine 에서 ±1 발현). CLAUDE.md 의 HWP3 규칙(HWP3 전용
로직은 parser/hwp3 안에서만)을 지키려면 컴포저 일반 규칙의 offset-불변성 쪽이 정공.
게이트 노이즈는 #1834(WARN_TEXTRUN 분리)가 이미 triage 를 정밀화했다.
