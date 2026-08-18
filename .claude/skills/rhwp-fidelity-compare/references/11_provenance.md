# 11 — provenance 기록

한컴 출력은 도구·버전·출력 경로·폰트에 따라 달라진다. 같은
`samples/foo.hwp` 라도 한글 2020 인쇄-PDF 와 한글 2024 파일-PDF 는
쪽수와 자간이 다를 수 있다. 비교 숫자는 그 환경에 묶인다.

정본 README: "재현 기록에는 해당 환경과 원본/기준 PDF 의 provenance 를
남기며, diff% 는 보편적 절대 판정이 아니라 후보 검출 근거로 쓴다."

## 하네스가 쓰는 파일

`provenance.tsv` 헤더: `role`, `path`, `grade`

```
role	path	grade
source	/repo/samples/2022-plan.hwp	원본 입력
reference_pdf	/repo/pdf/2022-plan-2022.pdf	기준 PDF: pdf/ 보존 한컴 2022 출력
```

direct pair 는 `--reference-grade` 가 두 번째 행 grade 가 된다.
등록 키는 `REG[].reference_grade` 가 들어간다. `--reference-grade` 를
등록 키 호출에 붙이면 하네스가 사용법 오류로 거절한다.

이 세 열은 **최소** 다. 한컴 버전과 메뉴는 에이전트가 작업 기록에
보탠다.

## 에이전트가 남기는 필드

| 필드 | 예 | 없으면 |
| --- | --- | --- |
| hangulTool | 한글 2022 | 비교를 참고 등급으로 강등 (F17) |
| hangulVersion | 12.0.0.5338 | "버전 미확인" 이라고 명시 |
| exportPath | 파일 → PDF로 저장 | 인쇄-PDF 가능성과 배율 위험을 적음 |
| fonts | `RHWP_FONT_PATH_DIR=...` + 설치 목록 요약 | F14 재실행 여지를 적음 |
| originalPath | 절대 경로 + SHA-256 가능하면 | 상대 경로만이라도 |
| oraclePath | 절대 경로 | 비교 금지 |
| referenceGrade | 한컴 2022 기준 PDF | 참고 PDF |
| rhwpBinary | `RHWP_BIN` + `git rev-parse HEAD` | stale 바이너리 위험 |
| python | `venv\Scripts\python.exe` | F09 |
| chrome | `CHROME_BIN` 또는 "text-only" | F10 |

`mydocs/working/agent_fidelity_compare.md` 와 각 런 노트에 표를 복사한다.
하네스 TSV 를 고쳐서 열을 늘리지 않는다. 새 스키마는 도구 PR 이다.

## 등급 문장

허용되는 등급 문장 (픽스처 `provenance_schema.json` 과 맞춤):

- `한컴 2022 기준 PDF`
- `한컴 2024 기준 PDF`
- `한컴 2020 기준 PDF`
- `참고 PDF — 버전·provenance 별도 확인`
- `사용자 지정 기준 PDF (provenance는 출력 파일 참조)`

"사실상 한컴과 동일" 같은 문장은 등급이 아니다. F05.

## 맞춰찍기 PDF

`save_as(PDF)` 가 PageCount 를 줄인 파일은 오라클이 아니다.
provenance 에 `exportPath=save_as(PDF), pageCountMismatch=yes` 를 적고
비교를 참고로 내린다. 15장.

## 레시피 — 런 노트 템플릿

```
# fidelity run 2026-08-18T12:00Z
hangulTool: 한글 2022
hangulVersion: unknown-asked
exportPath: pdf/ 보존본 (*-2022.pdf)
fonts: RHWP_FONT_PATH_DIR unset, Windows installed
originalPath: samples/2022-plan.hwp
oraclePath: pdf/2022-plan-2022.pdf
referenceGrade: 한컴 2022 기준 PDF
rhwpBinary: target/pr-review/release-test/rhwp @ <sha>
python: venv/bin/python
chrome: /usr/bin/google-chrome
out-dir: /tmp/rhwp-fidelity-plan
mode: pixel-and-text
range: 0-34
```

이 블록 없이 "한컴과 비교했다"고 쓰지 않는다.

## 에이전트 금지

- 등급 없이 시트를 PR 에 붙이기
- bunjang 동반 PDF 를 `한컴 2022 기준 PDF` 로 고쳐 쓰기
- provenance.tsv 스키마를 이 스킬이 확장
- 원본 경로를 상대 경로로만 남겨 다른 worktree 에서 재현 불가하게 하기
  (가능하면 절대 경로 + 저장소 상대 경로 둘 다)
