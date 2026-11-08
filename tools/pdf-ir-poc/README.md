# PDF → rhwp IR 로컬 POC

브라우저 확장/WASM을 목표로 PDF frontend를 실험하는 독립 Rust crate다.
제품 workspace, 루트 Cargo 설정, Studio bundle에 통합하지 않는다.

## 현재 체크포인트

- `pdf_oxide 0.3.78`에서 span/image/path를 추출하여 기존 rhwp Document IR로 lowering한다.
- 괘선 격자 검출기와 계층형 표/문단 상태머신이 소유권·후보·보류 근거를 남긴다.
- 중첩 표는 셀 복합체의 F2 경계·호몰로지 검증과 셀 슬롯 대입 모델로 계층을 인식한다.
  확인된 가족은 실제 중첩 Table IR로 생성하며 실패한 가족은 배치형 출력 전체를 보존한다.
  구현과 수학적 적용 한계는 [중첩 표 POC](NESTED_TABLES.md)에 정리했다.
- 기본 변환은 배치형 글상자·그림·도형이다. `--tables`는 geometry-confirmed 표 후보를
  실제 Table/Cell/Paragraph IR로 생성하며 불확실한 후보는 배치형 경로에 보류한다.
- 글상자는 명시적 0 여백과 단일 줄 SQUEEZE로 저장한다. 제품 serializer가 생략한 0 여백은
  POC 전용 구조화 ZIP/XML adapter에서 명시한다.
- `--painted-rules`는 flat-cap 축 방향 획의 물리 면적을 채운 Rectangle으로 표현한다.
- 2026-10-14 메인테이너가 기존 B본의 한컴 열기·조판 배치를 확인했다.
  기준 rhwp SHA: `552f5228d2dbfd6046c838f14241704f1bcb8a05`.
  확인 HWPX SHA-256: `a691dbb3309fb7f45372720bf51e59b058395efcf680040278f5bbc2b1817225`.
  기존 파일은 `output/poc/pdf-ir-20261013/pdf-oxide-hancom-v2/table-painted-page/compiled.hwpx`다.
  이 판정을 다른 source SHA, A본, 편집 후 저장·재열기에 자동 적용하지 않는다.

## 재현

저장소 루트에서 실행한다. Rust 1.93.1, wasm32 target, wasm-bindgen-cli 0.2.127,
Node, Poppler, Python의 Visual Sweep 의존성과 Studio의 puppeteer-core가 필요하다.
Chrome 경로가 macOS 기본값과 다르면 `CHROME_PATH`로 지정한다.

```sh
cargo +1.93.1 run --manifest-path tools/pdf-ir-poc/Cargo.toml --release --locked \
  --target-dir target/pr-review -- pdf/issue5875/nested_table_text_caption-2020.pdf \
  output/poc/pdf-ir/table-painted-page --painted-rules

node tools/pdf-ir-poc/verify-local.mjs "$PWD" /path/to/wasm-bindgen output/poc/pdf-ir
node tools/pdf-ir-poc/verify-visual.mjs "$PWD" output/poc/pdf-ir
```

`--painted-rules`를 생략하면 기존 Line을 유지한 A본이다. `--tables`를 지정하면 실제 표
객체를 생성한다. 두 옵션은 개별 모드이며 함께 지정하지 않는다. CLI는 입력 첫 페이지만 변환한다.
검증기는 텍스트 대조군과 표 입력을 Native/fresh WASM에서 재열고, source/input/module 해시와
실행 로그를 output에 남긴다. 독립 텍스트·raster는 원본 PDF에서 새로 생성한다.
원본 전체 12쪽과 선택한 첫 쪽을 혼동하지 않으며, rhwp 검사를 한컴 통과로 대신하지 않는다.

## 표 객체화 구현

- 괘선 격자의 행/열/병합을 Cell 주소, rowspan/colspan, 물리 크기 및 cell_grid에 연결한다.
- 상태머신의 셀별 문단 후보를 실제 Paragraph로 만들고, span 스타일의 UTF-16 위치를 보존한다.
  추가된 공백은 PDF glyph 간격으로 추정한 것이며 authored 공백 복원을 보장하지 않는다.
- 그림은 소유 셀의 inline Picture 문단에 넣는다. 셀의 첫 텍스트 앞 간격은 문단 속성과
  첫 LineSeg frame에 함께 기록한다. 그림 앞 간격은 앞 문단의 후행 간격 또는 첫 셀 패딩으로
  표현하여 그림 원점 이전에 소비한다. 측정/배치가 다른 간격을 중복 추정하지 않게 한다.
- 격자 선과 셀 안 글상자는 실제 표로 대체하므로 중복 출력하지 않는다. 셀 밖의 내용은 보존한다.
- 단일 표 기존 경로는 균일 실선을 BorderFill에 연결한다. 중첩 가족은 셀 변별 색/굵기를
  연결한다. HWP 굵기 단계로 양자화하며 한 변 안의 혼합 스타일은 보류한다.
- colspan/rowspan, 빈 셀, 보류 후보, 실패 시 내용/스타일 보존 및 HWPX 왕복을 검사한다.
- 검증 표 입력: 실제 표 1개, 5행 3열 15셀, 세 번째 행/세 번째 셀 내부 그림 2개.
  원본 338자의 내용은 공백을 제외해 보존한다. 표 밖 제목 글상자 1개를 유지한다.
- 47개 POC 검사, Native/fresh WASM 보고서/PNG 일치와 독립 PDF 첫 페이지 비교를 실행한다.
  표 객체 출력의 실루엣은 Native/WASM 각각 92.37282%다. 글꼴/간격 차이는 남아 있다.
- **새 표 객체 출력은 한컴 미검증**이다. 기존 B본의 확인을 새 모드로 승격하지 않는다.
  재열기 조판/편집 후 저장 및 전쪽 변환도 별도로 확인해야 한다.

## 중첩 인식 단계 검증 (이전 체크포인트)

`c0dd49e28`, 2026-10-14 당시의 기록이다. 아래 객체화 단계와 구분한다.

- 기존 47개와 새 중첩 계약 12개, 총 59개 POC 검사가 통과했다.
  Native/WASM Clippy와 fresh WASM Worker 실행을 확인했다.
- 독립 합성 PDF의 3단계 표 계층·단일 내용 소유권은 Native/WASM 보고서가 일치했다.
  중첩 객체화는 보류하며 배치형 HWPX를 그대로 유지한다. 이 출력의 PDF 실루엣은 양쪽 100%다.
- 기존 B본과 단일 표 객체 HWPX는 변경 전 파일과 byte-identical이다.
  단일 표 출력은 Native/WASM 각각 92.37282%, B본은 98.35443%를 유지했다.
- 실제 `nested_table_text_caption-2020.pdf` 전체 12쪽을 Native에서 인식 진단했다.
  7쪽은 부모 1개/자식 3개, 8쪽은 부모 1개/자식 2개를 인식했다. 원본 PDF도 직접 판독했다.
- **실물 전수 인식은 미완료**다. 7쪽의 `<표 2>`는 부분 내부 가로 경계로 보류됐고,
  8쪽의 `<표 6>`·`<표 7>`은 한 행짜리 격자로 현재 기본 검출 범위 밖이다.
  인식되지 않은 내부 표의 내용은 부모 영역에 남을 수 있으므로 의미 복원 완료가 아니다.
  3쪽은 모호한 내용 소유권으로 보류됐다. 이 진단은 전쪽 HWPX 변환·시각 통과를 뜻하지 않는다.
- 재현 output: `output/poc/pdf-ir-20261014/nested-recognition/`.
  `nested-probe.json`은 실물 전쪽 진단, `verification/`은 검사·lint·해시·TSV 요약,
  `browser/evidence/`는 Native/fresh WASM 결과다. 한컴 중첩 조판·편집은 미검증이다.

## 중첩 객체화 단계 검증 (2026-10-14)

- 인식된 부모 셀에 자식/손자 Table을 실제 컨트롤로 생성한다. 가로 인셋은 TAC 바깥 여백,
  세로 간격은 호스트 줄과 직전 문단/첫 셀 padding으로 보존한다.
- 미확정 문단은 원본 span 글상자를 셀의 줄 프레임에 상대 배치하며 작성 당시의 문단 의미를
  임의로 확정하지 않는다. 미인식 내부 표의 괘선은 도형으로 남는다.
- 63개 POC 검사, Native/WASM Clippy, fresh WASM Worker와 HWPX 왕복이 통과했다.
  실제 셀 포함·내용 보존·중복 출력 방지·실패 복구와 손자 셀 텍스트의 공개 API 편집/재저장을 검사했다.
- 합성 3단계는 실제 표 3개/99.67123%, 원본 7쪽은 실제 표 4개/97.12543%,
  원본 8쪽은 실제 표 3개/96.60277%다. 각 선택 1쪽의 Native/WASM 점수이며 양쪽 PNG는 동일하다.
  글꼴/간격 차이는 남는다. 원본 12쪽 전체 변환은 하지 않았다.
- 기존 B본과 단일 표 대조군 HWPX/PNG는 그대로 유지됐다.
- **이 중첩 객체 HWPX는 한컴 조판 실패가 보고되었다.** 이전 배치형 샘플의 확인을 새 출력에 적용하지 않는다.
- 새 샘플: `output/poc/pdf-ir-20261014/nested-tables/real7-structured-page/compiled.hwpx`,
  `real8-structured-page/compiled.hwpx`, `nested-structured-page/compiled.hwpx`.
  같은 output의 `verification/` 및 `browser/evidence/`에 재현 기록이 있다.

한컴 재판정용 보정본은 `output/poc/pdf-ir-20261014/nested-tables-hancom-v1/`에 별도 생성했다.
셀 내부 개체의 본문 연동, PDF 줄 높이의 고정 간격 표현, 첫 줄 인셋의 단일 소유를 보정했다.
63개 검사와 Native/fresh WASM 검증을 통과했으며 선택한 7쪽/8쪽 점수는
97.12586%/96.60277%다. **실제 한컴 Print에서는 7쪽 57.69848%, 8쪽 40.22202%로 불합격**이다.
7쪽 상단 겹침은 해소됐지만 표/잔류 괘선 배치와 셀 재조판 결함이 남는다.
원본의 중복 stroke와 객체화 뒤 남는 괘선도 독립 Poppler/qpdf로 확인했다. 상세 근거와 제한은
[한컴 재조판 비교본](NESTED_TABLES.md#한컴-재조판-비교본-2026-10-14)에 기록했다.

현재 객체화 실험과 한컴 실패 근거는 `3b879f610`에 체크포인트로 커밋했다.
다음 단계는 경계 정규화/소유권 → 부모 셀별 내부 격자 → 단일 자식 한컴 재조판 → 가족 확대
순으로 재도전한다. [재도전 전략](NESTED_TABLES.md#재도전-전략-2026-10-14-승인됨)은
2026-10-14 메인테이너가 승인했으며 R1부터 단계별 구현·검증을 진행한다.

R1에서 원본 기여 ID로 중첩 표 경계의 괘선을 소비하도록 보정했다. 선택한 7쪽/8쪽의
인식 축과 겹치는 잔류 괘선은 17개/10개에서 양쪽 0개로 줄었고 Table 수와 전체 글자는 유지됐다.
66개 POC 검사와 fresh WASM을 통과했으며 Native/WASM 점수는 97.12563%/96.60263%다.
실제 한컴 Print는 54.66379%/36.81599%로 계속 불합격이다. R1 경계 소유권과 R3 재조판
판정을 구분하며, 증적은 `output/poc/pdf-ir-20261014/nested-tables-retry-r1/`에 있다.

R2는 검증된 부모 셀 안의 1행/1열 표 탐색과 잉크로 지지되는 논리 접합을 추가했다.
7쪽 `<표 2>`와 8쪽 `<표 6>`·`<표 7>`까지 검출되어 양쪽 실제 Table 수는 5개(부모 1, 자식 4)다.
잔류 원본 괘선은 0개, 69개 POC 검사는 통과했고 Native/fresh WASM 점수는 96.83768%/96.22903%다.
실제 한컴 Print는 67.01952%/39.91311%로 불합격이며 R3 재조판 보정을 진행한다.
증적은 `output/poc/pdf-ir-20261014/nested-tables-retry-r2/`에 보존했다.

R3는 중첩 셀의 모든 PDF 줄을 실제 Paragraph/Run으로 내보내고, 물리 좌표가 CHAR 단위로
직렬화되지 않도록 `2 × hu(points)` 변환을 사용한다. span 간격에는 실제 저장 탭 폭을 넣는다.
선택 7쪽/8쪽 Native/fresh WASM은 93.76171%/94.12229%, 실제 한컴 Print는
94.05924%/92.76461%이며 글자 614/714개가 각각 전부 보존됐다.
한컴 HWP 저장 진단에서도 `<표 5>` 호스트의 잘못된 추가 높이 27.8pt가 해소됐음을 확인했다.
범위는 선택한 두 1쪽 문서이며 전체 12쪽 변환과 한컴 UI 편집 통과를 뜻하지 않는다.
Native/WASM 증적은 `output/poc/pdf-ir-20261014/nested-tables-retry-r3-verified/`,
한컴 증적은 `output/poc/pdf-ir-20261014/nested-tables-retry-r3/`에 있다.
상세 실험과 독립 근거는 [R3 결과](NESTED_TABLES.md#r3-결과-2026-10-14)를 참조한다.

R3 커밋 `5a7e4bbf8`에서 R4 재검증도 통과했다. 한컴 무편집 저장 HWP의 재열기 Print는
8쪽 92.76461%를 유지한다. 원본 12쪽의 인식 진단도 실행했고 3쪽의 모호한 소유권은
보류했다. 새 샘플 UI 판정과 원본 12쪽 전체 변환은 미검증이다.
최종 샘플은 `output/poc/pdf-ir-20261014/nested-tables-retry-r4/` 아래의
`real7-structured-page/compiled.hwpx`, `real8-structured-page/compiled.hwpx`다.
원본과 한컴 Print PDF·TSV·job·해시는 같은 루트에 있다.
[최종 확인 파일](NESTED_TABLES.md#r4-결과와-확인-파일-2026-10-14)을 참조한다.

```sh
cargo +1.93.1 run --manifest-path tools/pdf-ir-poc/Cargo.toml --release --locked \
  --target-dir target/pr-review -- pdf/issue5875/nested_table_text_caption-2020.pdf \
  output/poc/pdf-ir/table-structured-page --tables
```

## 추적 범위와 후속 과제

Git에는 Rust/JS 소스, Cargo.lock, 이 문서만 보존한다. WASM/HWPX/SVG/PNG와 실행 manifest는
ignored output에 둔다. 검증·빌드 target은 저장소 공용 `target/pr-review`다.

후속 과제는 선택 범위 밖 미인식 내부 표의 원본 획 추적, 한 행짜리 표의 추가 오검출 검증,
한컴 UI 편집 검증, 글꼴과 세부 간격 복원, 한 변 안의 혼합 테두리,
borderless 표 및 원본 전쪽 변환이다. 기존 B본은 비교 기준으로 보존한다.

보안 한계: 입력/추출 결과 예산만 있으며 parser 내부 선행 할당·압축 폭탄·CPU 상한은
보장하지 않는다. 제품 통합 전에 Worker 종료 정책과 손상/공격성 PDF 검증이 필요하다.
