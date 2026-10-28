# 점 기호의 글꼴 공급자·자연 전진 회귀 증적

## 현재 전쪽 검증

구현 source는 `8dc75860cec88ddf7c3f8aeee230b2ca7e2e5537`이다. 명시적인 굴림·Arial·
함초롬바탕의 정상 HWPX 3개를 각각의 독립 한컴 Print와 비교했다. Native224와 최적화한
fresh WASM063에서 **전 3쪽 raw 실루엣 100%**, 독립 PDF와 전체 쪽수 1쪽이 일치한다.
`tsv/{native,wasm}/`는 TSV 원문이며 `independent_coordinates.json`에 입력/PDF/TSV 해시와
파일별 실제 명령·독립 glyph 좌표를 보존한다. 중간 실행 로그와 MCP 응답은 output에 둔다.

대표 정식 review·overlay는 `png/{native,wasm}/<글꼴>/`에 있다. 같은 좌표를 4배로 확대한
`details/`에는 Native / Print / Native overlay 및 fresh WASM / Print / fresh overlay를
같이 표시했다. 확대는 진단용이며 전쪽 점수를 다시 산출하거나 개별 이미지를 이동·정렬하지 않았다.
점의 높이·모양, 기준선과 뒤 글자의 자연 간격을 직접 대조했다. AA와 획의 raster 농도 차이는
남으며 RGB 완전 일치라고 보고하지 않는다.

## 원 PR과 보정 근거

원 [PR #8057](https://github.com/edwardkim/rhwp/pull/8057)의 source는
`76271c14e4823a10a69747930f1c128f7bc7380a`, 이 브랜치의 반영은
`ec6c9c8f5ba4b0183bdf06e6a924379e66af6883`이다. 원 검사는 font/DocInfo가 없는
메모리 Document를 수동으로 만들고, U+2024가 항상 투명한 원문과 벡터 원으로 출력돼야
한다고 검사했다. `주거․수도․광열`이라는 사용 문맥만으로 모든 글꼴의 Unicode 글리프를
다른 모양으로 대체할 수는 없다.

독립 Print는 글꼴마다 다른 실제 프로그램과 자연 전진을 쓴다. 글꼴이 자신의 U+2024·U+2027을
가지고 있으면 그 프로그램을 보존하며, 없을 때만 Print에서 확인한 공급자를 사용한다.
측정과 paint가 같은 글리프를 소비하도록 보정한 source는
`b12357d79a7ae8d2f09847ad8fe7ac22014c86d0`이고, 현재 runtime에도 포함돼 있다.

| 입력 | 본문 | 독립 Print의 공급자 계약 |
| --- | --- | --- |
| `gulim.hwpx` | `주거․수도․광열` 및 `A․B · C.D ‧ E` | 원 Gulim 프로그램에 없는 점은 Haansoft Dotum 공급자를 사용한다. |
| `arial.hwpx` | `A․B · C.D ‧ E` | 원 Arial 프로그램에 없는 점은 Haansoft Batang 공급자를 사용한다. |
| `hcr-batang.hwpx` | `A․B · C.D ‧ E` | 원 HCR Batang 프로그램의 실제 점 글리프를 보존한다. |

이 세 입력은 `rhwp scaffold`의 명시적 font/paragraph 설정으로 생성했다. 저장 LineSeg나
좌표를 수동으로 주입하지 않았다. 원 PR의 미첨부 가계동향 문서 11쪽을 이 자료의 비교
페이지로 표기하지 않으며, 메모리의 글꼴 없는 helper와 정상 파일 입력도 구분한다.

한컴 2020 **11.0.0.9136**, `PrintMethod=0`, `hancom2020_pdf_driver_one_up`으로
각 입력 자체를 실제 인쇄했다. PDF는 `pdf/verified-point-suppliers/`에 보존한다.

| 입력 | Print job |
| --- | --- |
| `gulim.hwpx` | `582b3dbd-d462-4ba9-b30c-3e672c246dcb` |
| `arial.hwpx` | `4e5e527b-31af-4e34-9e97-4da7ff2745a1` |
| `hcr-batang.hwpx` | `125ea0ba-9b98-48b2-b016-d3176709504a` |

## 새 정식 검사

[verified_point_glyph_supplier.rs](../../../../tests/cases/verified_point_glyph_supplier.rs)의
새 함수 3개는 다음을 검사한다.

- 같은 입력과 독립 PDF의 SHA-256 및 전체 쪽수.
- 원문 기호·뒤 글자의 순서, 누락·중복과 같은 줄의 공유 기준선.
- 점과 B·C·E의 독립 Print 원점. 양쪽 raw 100%·직접 판독을 마친 1쪽에만
  **96 DPI, 종이 왼쪽 위 원점의 CSS px ±2px**를 적용한다.
- SVG font-family에 Print의 공급자가 포함되는지와 HCR의 원 프로그램 우선 여부.
- U+00B7의 기존 벡터 점 하나를 보존하고, 별개의 두 Unicode 기호를 무조건 원·타원으로
  대체하지 않는지 확인한다. 전체 SVG 해시는 사용하지 않는다.

같은 정식 소스로 보정 전 Native215는 **1 PASS / 2 FAIL**, 현재 Native224는
**3 PASS / 0 FAIL**, ignore·실행 누락은 0건이다. Arial·굴림의 공급자와 뒤 글자 전진
검사가 실제로 수정 전 실패하며 HCR의 원 프로그램은 양쪽에서 통과하는 정상 대조군이다.
동결 라이브러리 실행과 focused native Clippy·rustfmt 검사를 수행했다. 전체 nextest와
전체 Cargo lint의 완료를 뜻하지 않는다.

현재 수집기의 `--prepare`로 새 모듈을 수집했다. 원 1개 검사를 새 3개 검사로 교체해
전체 static test 수는 7854→7856개, source 수는 1810개로 같다. 생성 harness·manifest는
커밋하지 않는다. 전쪽 Native/fresh 90% 최소 조건과 완료 페이지 양쪽 98% 유지 조건도
계속 적용한다.

## 수용하지 않은 원 검사 보존

고정 base `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`에는 이 원 검사 파일이 없다.
이번 통합 후보에 들어온 신규 검사의 잘못된 보편 가정을 보정하는 절차다. 기존 devel의
회귀·fixture를 90% 미달 이슈로 이관하는 절차와 구분한다.

- 이전 활성 경로: `tests/cases/one_dot_leader_middle_dot_paint.rs`.
- 제거 직전 source: `522a9bc41bc761d2c83be2596121a610edc94913` 및 원 반영 `ec6c9c8f5ba4b0183bdf06e6a924379e66af6883`.
- Git blob: `a54d38c2e56b8ee8c3c8f36c3a2d55ee291f4f13`. 두 commit의 blob이 같다.
- 전체 모듈과 helper 보존: [원 검사 .rs.txt](incoming-tests/one_dot_leader_middle_dot_paint.rs.txt).
- 이전·보존 사본 SHA-256: `092514d6a15f8743b877cbeff2d82b19e6bd8fd83c08a707a0ca960fc5842167`.
- 활성 제거: 모든 글꼴의 U+2024를 원으로 강제하는 함수 1개와 전용 helper.
- 활성 유지: U+00B7의 기존 벡터 경로와 정상 글꼴의 기호·내용 보존.
- 활성 대체: 독립 Print로 검증한 위 세 함수.
- 복원 조건: 특정 글꼴·입력에서 별도 벡터 계약을 주장하려면 독립 Print와 전체 페이지
  Native/fresh 90% 이상, 직접 확대 판독 및 정상 글꼴 대조군을 먼저 입증한다. Unicode
  값만으로 원을 강제하는 보편 검사를 그대로 복원하지 않는다.

보존 사본은 `mydocs/pr/assets/`의 `.rs.txt`라 자동 수집되지 않는다. 원본을 폐기하거나
`#[ignore]`·조용한 return으로 실패를 숨기지 않았다.

## 재현

파일별 실제 명령은 좌표 JSON의 `qualification.{native,wasm}.command`에 있다. 같은
입력·PDF·96 DPI·글꼴 공급을 유지하고 현재 producer 경로로 실행한다.

```bash
venv/bin/python scripts/visual_sweep.py --silhouette-only \
  --file-target <이름> <같은 HWPX> <같은 Print PDF> \
  --rhwp-bin <Native 실행 파일> --compat 2022 --dpi 96 --page-workers 1 \
  --font-path /usr/share/fonts --font-path /usr/local/share/fonts \
  --font-path /home/tsjang/.local/share/fonts --font-path /home/tsjang/rhwp/assets/fonts \
  --out output/pr-review/<검토명>/<이름>
# fresh WASM은 같은 명령에 --wasm-pkg <최적화한 web package>를 추가한다.
```

`--compat 2022`는 RHWP 조판 프로필이고 실제 한컴 Print engine은 2020이다.
Native producer의 SHA-256은
`e333c70587ecf48e24632de8e6f3b843f33eb328051580442af4f46214743ca4`다.
현재 자료는 세 글꼴 계약의 증적이며 전체 PR의 나머지 Rust·Studio·시각 검증은 계속 진행한다.
