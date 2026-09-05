# 01 — 언제 쓰는가: 독립 한컴 PDF 가 있을 때만

이 하네스는 **한컴이 내보낸 공식 PDF** 를 기준으로 rhwp 의 SVG 를 쪽별로
겹친다. 기준이 없으면 정직한 도구가 아니다. 그 경우의 정직한 도구는
`rhwp render-diff` 이며, 스킬은 `rhwp-visual-regression` 이다.

## 독립 한컴 PDF 란

다음을 **모두** 말할 수 있어야 한다. 하나라도 모르면 등급을 "참고 PDF" 로
내리고 최종 기준으로 쓰지 않는다 (F17).

1. 한컴 도구 이름 (한글 2020 / 2022 / 2024, 한컴오피스)
2. 버전 문자열 (가능하면 `12.0.0.xxxx`)
3. 내보내기 경로 (파일→PDF로 저장 / 인쇄→Microsoft Print to PDF / `pdf/` 보존본)
4. 그때 쓰인 글꼴 환경 (설치 목록 또는 `RHWP_FONT_PATH_DIR`)
5. 원본 HWP/HWPX 경로
6. 오라클 PDF 경로

이 여섯이 `provenance.tsv` 와 `mydocs/working/` 작업 기록에 남는다.
하네스가 자동으로 한컴 버전을 읽지는 않는다. 에이전트가 사용자·작업지시자에게
물어 적는다.

## 정직한 경우

- `pdf/` 아래 버전 접미사가 있는 장기 기준 자료 (`*-2022.pdf`, `*-2024.pdf`)
- 작업지시자가 "이 PDF 는 한글 2022 에서 파일→PDF 로 뽑았다"고 준 파일
- REG 키 `plan` `manual` `korexam` `math` `eng` 의 등록 쌍 (bunjang 제외)

`plan` 실측(2026-07-26)은 이 경로로 #3385 PUA 원문자 tofu 를 찾았다.
그 발견은 "diff% 가 커서"가 아니라 **시트를 사람이 본 뒤** 이슈가 됐다.

## 정직하지 않은 경우 (F01)

| 손에 있는 것 | 하면 안 되는 일 | 정직한 다음 수 |
| --- | --- | --- |
| 편집 전·후 HWP 만 | fidelity_compare 에 후 PDF 를 가짜로 넣기 | render-diff A B |
| 같은 파일 두 번 | 한컴 기준이라고 부르기 | render-diff A A |
| 사용자 말만, PDF 없음 | 빈 PDF 를 만들거나 export-pdf 를 오라클로 | PDF 를 요청하거나 인계 |
| `rhwp export-pdf` 산출 | 한컴 기준으로 바꿔 치기 | 자기 자신과 비교가 됨 |
| 스크린샷 JPEG | 해상도·압축을 오라클로 | 공식 PDF 를 요청 |

`rhwp export-pdf` 는 유용한 미리보기일 수 있으나 **한컴 공식 출력이 아니다.**
이 스킬의 기준 열은 항상 한컴이 뽑은 PDF 다.

## 동반 PDF (`samples/*.pdf`)

`bunjang` 키처럼 입력 옆에 놓인 PDF 는 REG 등급이
"참고 PDF — 버전·provenance 별도 확인" 이다. 도구·버전·출처를 확인하기
전에는 최종 기준으로 승격하지 않는다. `pdf/` 보존본과 혼동하지 말 것.

사용자가 "샘플 폴더에 PDF 가 있는데" 라고 하면 등급 표를 먼저 연다.
[18_registered_keys.md](18_registered_keys.md).

## 맞춰찍기·배율 함정

`hangul_pdf_baseline.md` 가 이미 적시한다. 한글 `save_as(PDF)` 는 인쇄
배율을 타서 편집기 25쪽이 생성 PDF 13쪽이 될 수 있다. PageCount 가
편집기와 다르면 그 PDF 는 오라클이 아니다. 이 스킬은 그 가드를
재발명하지 않고, 쪽수 불일치를 `page-count-ledger.tsv` 후보로만
기록한다 (F11). 전역 page-break 패치를 열지 않는다.

## 레시피

```bash
# 1) 사용자에게 오라클을 확인
#    "이 PDF 는 한글 몇에서, 어떤 메뉴로 뽑았습니까?"
# 2) 등록 키면
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 9 \
  --out-dir /tmp/rhwp-fidelity-plan
# 3) 아니면 direct pair + 등급
venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 9 \
  --source samples/doc.hwp \
  --reference-pdf pdf/doc-2022.pdf \
  --label doc-2022 \
  --reference-grade '한컴 2022 기준 PDF' \
  --out-dir /tmp/rhwp-fidelity-doc-2022
```

오라클이 없으면 여기서 끝 (F01). `rhwp-visual-regression` 스킬을 연다.
그 스킬 파일을 이 PR 에서 고치지 않는다 (F07).

## 에이전트 질문 스크립트

1. "한컴이 내보낸 PDF 가 있습니까?"
2. 예 → "도구·버전·메뉴·글꼴을 압니까?" → provenance 기록 → 이 스킬
3. 아니오 → "편집 전후를 숫자로 보시겠습니까?" → visual-regression
4. "버그를 실사용 여정으로 찾겠습니까?" → bug-hunter (F08, 재작성 금지)
