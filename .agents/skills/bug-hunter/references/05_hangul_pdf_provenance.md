# 05 — 한컴 PDF provenance

한컴 출력은 도구·버전·출력 경로·폰트에 따라 달라진다. playbook:
"해당 환경과 provenance를 기록한 비교 기준으로 쓰고 보편적 절대
오라클로 간주하지 않는다."

기록 없이 비교 시트를 이슈에 붙이면 F03. 그 시트는 근거가 아니다.

## 필수 키

`fixtures/provenance_keys.json`:

| 키 | 예 (playbook 예시 6) |
| --- | --- |
| tool | Hwp 2022 |
| version | 12.0.0.4426 |
| outputPath | 인쇄 → PDF / 다른 이름으로 저장 |
| fonts | 문서가 요구하는 face + 설치된 family |
| sourcePath | `samples/21_언어_기출_편집가능본.hwp` |
| referencePdfPath | `pdf/21_언어_기출_편집가능본-2022.pdf` |
| sourceSha256 | `905454045c…` |
| referenceSha256 | `f2d858d797…` |
| creator | `Hwp 2022 12.0.0.4426` |
| producer | `Hancom PDF 1.3.0.550` |
| paper | A3 841×1190pt |
| recordedAt | ISO-8601 |

PDF 메타의 Creator/Producer 를 읽고 적는다. "한컴으로 뽑았다"는
문장은 키가 아니다.

## 출력 경로를 적는 이유

같은 버전이라도 "PDF로 저장"과 "인쇄 드라이버"와 "한컴 PDF
프린터"가 텍스트층·폰트 임베드를 다르게 만든다. 텍스트 멀티셋이
흔들리면 먼저 이 키를 의심한다.

## 폰트

- 문서 legacy face ≠ 설치 family 일 수 있다.
- fidelity_compare 는 `--font-style` 기본으로 `@font-face
  src: local(...)` 별칭을 쓴다. 글꼴 바이너리를 SVG 에 embed 하지
  않는다.
- `RHWP_FONT_PATH_DIR` 계약은 유지.
- 휴먼명조/휴먼고딕처럼 Chrome 이 `.notdef` 로 그리는 EBDT 는
  outline 명조·고딕을 먼저 고른다. HY신명조는 원 face 우선.

설치되지 않은 폰트로 난 픽셀 diff 를 rhwp 결함으로 쓰지 않는다.
provenance 의 fonts 칸에 "미설치"를 적고 후보에서 내린다.

## samples/ 동반 PDF

도구·버전·provenance 를 확인하기 전에는 참고 자료다. 예시 6처럼
키가 이미 playbook 에 박힌 쌍만 한컴 기준으로 승격한다.

## 관련

- 도구 사용: [12_fidelity_compare.md](12_fidelity_compare.md)
- 표본 TSV: `fixtures/tsv/provenance_sample.tsv`
- 예제: [06_float_margin_leet.md](../examples/06_float_margin_leet.md)
