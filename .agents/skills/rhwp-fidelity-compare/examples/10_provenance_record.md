# 예제 — provenance 를 남긴다

이슈 #5329. 실 에이전트 경로. gym 아님.

## 하네스 TSV

```
role	path	grade
source	/repo/samples/2022-plan.hwp	원본 입력
reference_pdf	/repo/pdf/2022-plan-2022.pdf	기준 PDF: pdf/ 보존 한컴 2022 출력
```

## 에이전트 보강

```
hangulTool: 한글 2022
exportPath: 파일 → PDF로 저장 (pdf/ 보존본)
fonts: Windows installed, RHWP_FONT_PATH_DIR unset
rhwpBinary: target/pr-review/release-test/rhwp @ <sha>
```

등급 없이 시트를 PR 에 붙이지 않는다.

관련: `references/11_provenance.md`.
픽스처: `fixtures/tsv/provenance_plan.tsv`.
정지 F05.
