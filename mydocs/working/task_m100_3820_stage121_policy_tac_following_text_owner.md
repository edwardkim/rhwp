---
kind: investigation
status: in_progress
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 121 — TAC 표 뒤 첫 문자 줄 소유권

## 범위

정책연구 p33의 `pi=428`은 선행 TAC 표와 Bottom caption 뒤에 URL 본문을 가진다.
한컴 PDF는 URL 전체 `statistics.eurotransplant.org)`을 표·caption 아래 한 줄에 두지만,
현재 rhwp는 앞부분 `statistics.eurotr`을 표 오른쪽의 좁은 잔여 폭에 넣고
`ansplant.org)`만 아래로 보낸다. Stage 119의 상단선과 Stage 120의 stored-reset
fragment geometry는 유지하고, 이 저장 줄 소유권만 별도 처리한다.

## 저장 증거

- host paragraph: section 0, `pi=428`
- leading control: TAC table `ci=0`, `38416×13956HU`
- caption: Bottom, width `8504HU`, max width `38416HU`, spacing `850HU`
- caption LIST_HEADER text: `표 12... (출처:`
- host PARA_TEXT: `statistics.eurotransplant.org)`
- stored LINE_SEG 0: `text_start=0`, `vpos=0`, `line_height=16086`
- stored LINE_SEG 1: `text_start=8`, `vpos=17086`, `line_height=1000`
- 첫 visible host 문자의 HWP character offset: `8`

따라서 URL은 caption의 잘못된 폭이나 parser의 caption text가 아니다. 원본 저장 줄은
첫 visible host 문자를 두 번째 줄의 시작으로 명시한다.

## 직접 원인

`src/renderer/layout/paragraph_layout.rs`의 stored line break 변환은 두 번째
`LINE_SEG.text_start=8`을 visible string의 `char_idx=0`으로 정확히 환산한다. 그러나
현재 `char_idx > 0` 조건이 이 권위적인 첫 문자 break를 버린다. 이후 일반 right-margin
fallback이 표 오른쪽 약 92.5px를 채워 URL 앞부분을 잘못 소유한다.

현재 render tree의 잘못된 배치는 다음과 같다.

- `statistics.eurotr`: `x≈608.6px`, 표 오른쪽 잔여 폭
- `ansplant.org)`: `x≈94.5px`, `y≈293.9px`, 표 아래

## 최소 판정 계약

`char_idx=0`을 전역 허용하지 않는다. 다음 저장 증거를 모두 만족할 때만 첫 visible
문자 앞 break를 보존한다.

1. 문단 첫 control이 inline/TAC table이다.
2. 표에 Bottom caption이 있다.
3. control 뒤 visible host text가 존재한다.
4. 두 번째 stored LINE_SEG의 `text_start`가 첫 visible host 문자 offset과 정확히 같다.
5. 첫 stored LINE_SEG가 선행 control을 소유한다.

저장 LINE_SEG 증거가 없는 leading TAC, middle TAC, multiple/empty control 문단에는 기존
same-line fallback을 유지한다.

## 수정·회귀 계약

- p33 `pi=428`의 URL은 표 아래 `x≈line_start`에서 하나의 TextRun으로 시작한다.
- 해당 문단의 TextRun이 table right 이후 잔여 폭에 나타나지 않는다.
- caption은 `(출처:`까지 table width 안에 유지한다.
- table bbox, p33 page owner, 문서 215쪽, 후속 `6. 프랑스` 위치를 바꾸지 않는다.
- stored 증거 없는 leading/middle TAC의 기존 inline 배치를 음성 회귀로 고정한다.

## 증적

- [p33 보정 전 직접 비교](../pr/assets/task_m100_3820_stage121_policy_tac_following_text_owner/compare_p033_before.png)
- [보정 전 픽셀 보고서](../pr/assets/task_m100_3820_stage121_policy_tac_following_text_owner/report-before.tsv)
- [보정 전 provenance](../pr/assets/task_m100_3820_stage121_policy_tac_following_text_owner/provenance-before.tsv)

구현과 focused/실물 회귀, 최신 CLI의 한컴 PDF 직접 비교를 이어서 수행한다.
