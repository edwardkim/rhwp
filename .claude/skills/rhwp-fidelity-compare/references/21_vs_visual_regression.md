# 21 — `rhwp-visual-regression` 과 다른 축

두 스킬은 둘 다 "화면이 깨졌나"를 묻지만 **기준이 다르다.**

| | rhwp-fidelity-compare (여기) | rhwp-visual-regression |
| --- | --- | --- |
| 기준 | 한컴이 내보낸 공식 PDF | 같은 엔진의 다른 HWP/왕복 |
| 도구 | `tools/fidelity_compare` + `export-svg` | `render-diff` / `ir-diff` / `thumbnail` / `export-png` |
| 숫자 | 픽셀 diff% 랭킹, 문자 멀티셋 | px 변위, STRUCT/OVER/PASS |
| 결정성 A==A | 해당 없음 | 필수 (F02 there) |
| 공식 PDF | **필수** | 없어도 정직 |
| 최종 판정 | 유지자 + 거버넌스 | 상태 코드가 더 기계적. 그래도 STRUCT 는 경로로 읽음 |
| gym | 금지 | 금지 |

자기 일관성 PASS 는 한컴 호환이 아니다. 한컴 PDF 와 4% 차이나는
문서도 A==A 는 PASS 일 수 있다.

## 언제 이쪽 / 언제 저쪽

- "이 편집이 레이아웃을 흔들었나" → visual-regression
- "한글 2022 PDF 와 같은가" → 여기
- 둘 다 있으면 **따로** 돌리고 문장을 섞지 않는다

잘못된 합성:

> render-diff PASS 이므로 한컴과 같습니다.

정직한 병렬:

> render-diff A B 는 PASS (의도 편집만 변위). 한컴 PDF 대조는
> p12 가 랭킹 1위라 시트를 유지자에게 넘깁니다.

## 이 폴더에서 하지 말 것 (F07)

- `.claude/skills/rhwp-visual-regression/` 를 수정
- render-diff 상태 코드를 여기 정지 표에 재작성
- `geom_inventory.tsv` 헤더를 여기 fixtures 에 복사해 "같은 도구"처럼 위장
- visual-regression 의 examples 를 이 스킬 examples 로 복제

이웃 `SKILL.md` 는 인계를 위해 **존재만** 확인한다. 내용은 그쪽
스킬이 권위다.

## 명령 혼동

| 사용자가 말한 것 | 실제 |
| --- | --- |
| render-diff 로 한컴 PDF 를 비교 | 불가. render-diff 는 HWP/HWPX |
| fidelity 로 편집 전후 HWP | 오라클이 한컴 PDF 가 아니면 F01 |
| thumbnail 을 한컴 기준으로 | thumbnail 은 PrvImage. 여기 도구 아님 |
| ir-diff 로 PDF | ir-diff 는 IR. PDF 텍스트층이 아님 |

## 에이전트 한 줄

"시각 회귀 스킬은 자기 일관성입니다. 한컴 공식 PDF 가 있으니
fidelity_compare 축으로 갑니다. 그쪽 스킬 파일은 건드리지 않습니다."
