# 13 — 이슈 템플릿

playbook: 격차마다 재현 명령, 관련 코드 경로(파일:라인), 정답지
대비 근거를 갖춘 이슈를 작성한다. 증상만 기록하지 않는다.

필수 세 칸이 비면 F11. 올리지 않는다.

## 필수 필드

`fixtures/issue_templates.json` · `fixtures/issue_template.md`

| id | 제목 | 합격 조건 |
| --- | --- | --- |
| repro | 재현 명령 | 복붙하면 같은 산출 |
| codePath | 코드 경로 | `파일:라인`, devel HEAD SHA |
| groundTruth | 정답지 대비 근거 | 종류 + provenance 또는 재독 표 또는 매뉴얼 문장 |

추가로 적는 것 (필수에 가깝다):

- classification — 소실/과잉/치환/재독/계약/픽셀 후보
- limitations — 오라클이 안 보는 축
- notAFix — 이 이슈는 헌팅 산출. 패치는 별도 PR

## 본문 골격 (IT01)

```markdown
## 재현 명령
```bash
<명령>
```

## 코드 경로
`crates/…/file.rs:LINE` (devel HEAD `<sha>` 에서 확인)

## 정답지 대비 근거
- 종류:
- provenance:
- 분류:
- 실측 표 (N중 M, 반례 수)

## 한계
- 이 오라클이 안 보는 축

## 수정
이 이슈는 헌팅 산출이다. 패치는 별도 PR.
```

## 값 손실이 아니면 아니라고 쓴다

구조 소실·정규화·검출 후보를 "데이터가 사라졌다"고 과장하지 않는다.
#3551 은 `default == case × 2` 가 성립해 구조 소실이지 데이터
손실이 아니었다. 그 문장이 판정의 일부다.

## 열지 않는 이슈

- 콘솔 깨짐 (F10)
- 정답지 없는 충실도 주장 (F04)
- devel 에 이미 없는 결함 (F14)
- 표본 1건 계약 단정 (F15)
- 실제 접수 실패 (우리는 접수하지 않음)

## 음성 결과 (IT03)

가설을 기각했으면 그것도 이슈/코멘트로 남긴다 (F16). 다음 사람이
같은 길을 다시 파지 않는다.

예제: [20_issue_from_finding.md](../examples/20_issue_from_finding.md)
