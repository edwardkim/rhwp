# CGMP 평가표 최신 fresh WASM 전쪽 증거

생산 코드: `b4ff8c81b5d986baf47bad9a3eee83cbd9d8367b`.
WASM SHA-256: `94fe70f13609c5a006b8e9044463fac7ae96fe5fe03ae33bc0c0153b19f7867c`.

fresh 전48쪽은 exit0·1010.918초, 최저47쪽93.64014%, 미달·누락0,
원본/Print 쪽수48/48, source guard=true다. 현재48쪽의 decoded RGBA는
직접 판독했던 이전7f04c fresh와 모두 동일하다. 새47쪽 review 및 제목의
같은 페이지 좌표 `(95,180)-(710,295)` 2배 확대를 직접 열었다.
외곽·문단 시작·줄바꿈·내용 순서는 보존된다. 제목/기관명의 글자 획·굵기·
일부 자간과 래스터 차이가 남으며 완전 일치를 주장하지 않는다.
이번 source의 CGMP Native 전쪽 재실행은 대기 중이며 양쪽 완료나 PR 승인으로
확대하지 않는다. TSV의 자동 gate `not_evaluated`와 직접 판독을 구분한다.

## 입력과 Print

- `pdf/semanticist21-20261008/cgmp-original-v97/cgmp_evaluation_table-0f03119d6ac3-2020.pdf` — SHA-256 `08ae67334ed2cb690dc25f5447d9c7204044a5b313e026fb20f4147de0182d4a`; 실제 파일과 HEAD blob 일치.
- `samples/issue6035/cgmp_evaluation_table.hwpx` — SHA-256 `0f03119d6ac3e19f74fc9fcec5315ac6ed436bab5abffd0804e9cf3350dd747a`; 실제 파일과 HEAD blob 일치.

## 재현 및 판정

원문 명령·입력/소스/폰트 해시·TSV·로그는 ignored
`output/pr-review/semanticist21-20261008/full-width-visible-alignment-v452/`의
`whole-fresh-execution-plan.json`, `whole-fresh-results.json`,
`cgmp-fresh/cgmp6035/silhouette_manifest.json`,
`cgmp-fresh-prior-pixel-comparison.json`, `whole-original-input-head-proof.json`,
`cgmp-fresh/cgmp6035/representative-review-provenance.json`에 보존한다.
원본과 독립 한컴2020 Print, `--compat 2022 --silhouette-only --wasm-pkg pkg`,
검증한 원문 공급 글꼴과 `--embed-fonts full`을 사용하며 페이지 범위는 전체다.
현재 상태는 [누적 검토](../../../archives/semanticist21_20261008_integrated_review.md)에 연결한다.

## PNG 해시

- `fresh-p047-heading-zoom.png` — SHA-256 `a49ea8596846992353a39395867bdd0674b61175ebdc2fa5bb10d848336e767d`
- `fresh-p047-overlay.png` — SHA-256 `26ee482182510dd07b5cf48b94132b7d9536c746dc7e4b4169b8d9809bc38d0e`
- `fresh-p047-review.png` — SHA-256 `4929506eb7b352a03829047c3b84aa874a05fcd1a1ecaffa3af5fe1c244f4204`
