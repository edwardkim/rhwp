# #6936 — 일반 face만 있는 글꼴의 PDF 굵게 복원

Issue: [#6936](https://github.com/edwardkim/rhwp/issues/6936)

- 확인일: 2026-09-13 KST
- 기준: `upstream/devel` `70bf40af2a2818e72bd58b4fa66e2d4c06de2b51`
- 작업 브랜치: `fix/6936-pdf-synthetic-bold-20260913`
- 초기 구현·시험·증적 커밋: `01cd447a4`
- PR 준비의 최종 코드 검증 커밋: `3ba6d6d3e`
- 판정: 기본 SVG→PDF 경로의 문제를 재현·수정했고 PR 준비의 로컬 필수 게이트를 완료했다.
  Windows 네이티브 실행 및 원격 CI는 미실행이다. 원격 push/PR 생성 전 로컬 준비 상태다.

## 재현과 원인

돋움·바탕·굴림처럼 Bold face가 없는 family는 SVG의 `font-weight="bold"`에도 usvg가
Regular face를 선택한다. 기존 svg2pdf는 이 글꼴을 채우기로만 기록하여 굵기가 사라진다.
SVG에 stroke만 붙이면 별도의 fill/stroke 텍스트 show 연산을 내보내 추출·검색 문자가 중복된다.

새굴림은 별도의 기존 오류도 있었다. layout-name 규칙이 `새굴림`을 `함초롬돋움`으로 바꾸어
원래 New Gulim 대신 HCR Dotum/실제 Bold를 사용했다. 이 경우 굵어 보이는 것만으로는
원래 글꼴의 굵게가 정상이라고 판정할 수 없다.

## 수정

1. PDF 전용 SVG 준비 단계에서 **실제 glyph에 선택된 face의 weight**를 확인한다.
   일반 face만 사용하는 굵게 text에 한컴 실측 비율 `stroke-width = font-size × 0.02`를 적용한다.
   원본 문자·face·glyph transform을 보존하며 실제 Bold face와 일반 텍스트는 건드리지 않는다.
2. vendored svg2pdf의 단색 fill+stroke를 PDF `Tr 2`로 결합한다.
   하나의 glyph를 한 번만 기록하며 색·불투명도·선 속성을 보존한다. 역순 paint는 같은 불투명 색일
   때만 결합한다. 서로 다른 역순 색·반투명 역순·gradient/pattern은 기존 paint 순서를 유지한다.
3. `새굴림 → New Gulim`의 layout-name 변경을 append-only change set으로 적용했다.
   기존 rule은 retired 이력과 successor를 남기고 새 rule을 추가했다. 기존 New Gulim metrics를
   사용하며 나머지 네 projection의 semantic hash는 동일하다. 봉인 v1/과거 기준선은 갱신하지 않았다.

rhwp가 내보내는 균일한 text cluster가 적용 대상이다. 임의의 혼합 굵기·실제 Bold와 Regular가
섞인 fallback·복잡한 paint·자식 stroke override는 잘못 합성하지 않고 unsupported로 보고한다.
PDF 합성 굵게를 브라우저용 SVG에 삽입하지 않는다. 원래 이슈의 부수적인 HML Bold 파싱 의문은
이번 HWP/HWPX 및 PDF 경로 검증으로 해결했다고 주장하지 않는다.

## 독립 기준과 visual sweep

새 합성 HWPX를 한컴으로 HWP 저장한 뒤 같은 HWP를 engine 2020으로 PDF 변환했다.
한컴 버전은 **12.0.0.4605**, 기준 PDF는 1페이지다. MCP `start → status → download`를 사용했다.
검증 입력·변환 job ID·SHA·로컬 Windows 글꼴 출처는 [fixture 설명](../../samples/issue6936/README.md)에 있다.
HWP/HWPX/기준 PDF와 세 대조 PDF 모두 Git에 포함했다. 기존 파일을 이름만 바꾸어 중복 추가하지 않았다.

[한컴/수정 전/수정 후 비교](../pr/assets/issue_6936/comparison.png)에서 12개 행 전체를 확인했다.

- 돋움·바탕·굴림의 보통과 굵게가 수정 전에는 같은 두께다. 수정 후 한컴 기준처럼 구분된다.
- 새굴림은 함초롬 치환이 없어지고 한컴 기준의 New Gulim 모양과 굵게를 사용한다.
- 함초롬돋움·함초롬바탕의 실제 Bold와 모든 일반 행에 불필요한 합성 획이 추가되지 않는다.
- 새굴림 두 행을 제외한 모든 글자 원점은 수정 전후 동일하다. 새굴림 두 행은 원래 face의
  advance를 사용하며 최대 원점 차이는 5.200012pt다. 이를 합성 획 자체의 위치 변경으로 숨기지 않는다.
- 전체 페이지 픽셀 동일 판정은 하지 않았다. 최종 비교 패널을 Git에 보존하고 전체 PNG는 임시 경로에서 재생성할 수 있다.

| 결과 | 공백 제외 추출 문자 | text show 연산 | 판정 |
| --- | ---: | ---: | --- |
| 한컴 기준 | 158 | 60 | 문자를 묶어 기록, 일반 face 굵게에 Tr 2 사용 |
| 수정 전 devel | 158 | 158 | Bold face 없는 돋움·바탕·굴림의 굵기 소실 |
| stroke만 추가한 대조군 | 207 | 207 | 49자 중복, 겉모양만으로 발견할 수 없음 |
| 최종 수정본 | 158 | 158 | 49개 합성 굵게 Tr 2, 추출 텍스트가 한컴 기준과 동일 |

[PDF 측정 기록](../pr/assets/issue_6936/README.md),
[재검증 스크립트](../../samples/issue6936/inspect_pdf.py),
[한컴 PDF](../../pdf/issue6936-bold-faces-2020.pdf), [최종 PDF](../../pdf/issue6936-after.pdf).

## 실행한 검증

전용 target: `/Users/tsjang/rhwp/target/issue6936-20260913`.
Mac에서 `win10-ted`에 설치된 해당 글꼴 파일을 읽기 전용으로 복사해 `--font-path`로 제공했다.
글꼴 파일은 배포하지 않고 해시와 원래 설치 경로만 기록했다.

| 검사 | 결과 |
| --- | --- |
| 기본 CLI build / 같은 HWP의 3개 코드 상태 export-pdf | 모두 exit 0 |
| #6936 focused nextest | 9/9 PASS |
| #5874 기울임 회귀 | 7/7 PASS |
| #3772 bold ExtraLight fallback 회귀 | 2/2 PASS |
| #7077 gradient/opacity mask 회귀 | 2/2 PASS |
| font rule v2 / projection / mutation rehearsal | 45/45 PASS |
| font projection generator check | PASS |
| native / WASM32 / workspace all-target Clippy (`-D warnings`) | 3종 모두 PASS |
| `cargo build --locked --workspace` | PASS |
| full release-test nextest | 9,749/9,749 PASS, 51 skipped |
| Native Skia lib | 4,112 PASS, 13 ignored |
| Native Skia 그림 placeholder / 직접 PDF export | 2/2, 4/4 PASS |
| #4966 기존 글꼴 projection 회귀 | 3/3 PASS |
| 실제 HWP public font trace schema/lifecycle | 새 규칙 참조 34개, traceSourceDrift/retired/replaced/dangling 0 |
| 최종 코드 PDF 재출력 / 검증 입력 6개 Git blob 대조 | 최종 PDF 바이트 동일, 입력 SHA 모두 일치 |
| `cargo fmt --all -- --check` | PASS |
| suite prepare 후 `rust-test-suite-manifest.mjs --check` | PASS |
| PDF 추출/Tr/단일 show/행별 원점/1페이지 검사 | PASS |
| fresh WASM `--no-opt` build | PASS (로컬 진단 경로) |
| native/WASM SVG 실제 실행 비교 | HWP 87,322 bytes, HWPX 87,430 bytes; 각각 1페이지, 바이트 동일 |

원본 실행 로그는 `/private/tmp/rhwp-6936-pr-prepare-20260913/archived-evidence/validation.log`에 보관한다.
파생 integration suite/manifest는 검증용이며 커밋하지 않았다.

추가로 `scripts/tests/font_rule*.test.mjs` 전체 97개를 실행하면 93개 통과, 다음 4개가 실패한다.
수정 전 primary checkout `70bf40af2`에서도 같은 4개 검사 실패를 재현했다.

- `all 1,352 candidates close to one row or an approved profile split`
- `all 830 current registry rules resolve as carried-forward active`
- `the complete W1 rule population closes as lifecycle or historical reference-only`
- `W7 pre-migration semantics remain equal after source ownership migration`

앞의 세 검사는 과거 population/lifecycle 가정, 마지막 검사는 기존 Studio webfont 공급/요청 목록과
W7 snapshot의 차이다. 기존 #7023 교체 이후 기준선도 실패하며 이번 추가 교체로 실제 개수는 달라지므로,
오류의 수치까지 동일하다고 주장하지 않는다. 봉인 v1 및 W7 snapshot을 덮어쓰거나 실패를 PASS로 기록하지 않았다.
최종 후보 로그는 `/private/tmp/rhwp-6936-pr-prepare-20260913/final/font-rules-all.log`,
기준 devel 로그는 `/private/tmp/rhwp-6936-pr-prepare-20260913/base-font-rules-all.log`에 보관한다.
이번 변경의 reducer·projection·mutation 검사 45개와 실제 HWP의 public trace 계약은 별도로 통과했다.

최초 전체 Rust 실행에서는 글꼴 규칙의 기존 retired 개수 가정과 public trace rule ID 계약 검사 2개가
실패했다. 새 replacement ID를 기존 candidate identity의 20자리 해시 계약에 맞추고, 기존 Rust 검사가
명시적인 두 change set의 교체만 허용하도록 보정했다. production trace API와 봉인 v1는 변경하지 않았다.
보정 후 필수 lint 묶음과 전체 9,749개를 다시 통과했다.

최종 실행 로그는 `/private/tmp/rhwp-6936-pr-prepare-20260913/final/`에 보관한다.
전체 nextest는 다음 명령을 사용했다. 10 logical CPU·32 GiB RAM 및 코퍼스 내부 worker를 고려해
동시 test process를 6개로 제한했으며 새로운 입력 두 개를 security sweep에 명시했다.

```sh
RHWP_SECURITY_SWEEP_SAMPLES_JSON='["samples/issue6936/bold-faces.hwp","samples/issue6936/bold-faces.hwpx"]' \
  cargo nextest run --locked --cargo-profile release-test \
  --target-dir /Users/tsjang/rhwp/target/issue6936-20260913 \
  --tests --test-threads 6 --no-fail-fast
```

native/WASM/workspace lint는 같은 전용 target에서 `local_validation.md` 4.3의 묶음을 순차 실행했다.
추가 소요를 줄이기 위해 코드가 바뀌지 않은 최종 문서 커밋 뒤에 전체 검증을 중복 실행하지 않는다.

### WASM 검사 이유와 범위

PDF export 자체는 네이티브 경로다. 이번 변경에는 공통 `새굴림 → New Gulim` 이름 규칙도 포함되어
WASM에서도 동일한 face 선택과 SVG 출력을 유지하는지 확인했다. PDF 전용 합성 stroke를 WASM에서
검사하거나 적용한 것이 아니다. 두 입력 모두 native/WASM SVG가 바이트 단위로 같으며 각 SVG의
New Gulim text node 26개와 PDF 전용 합성 태그 부재를 확인했다. Node에서 실제 WASM 모듈을 실행했으며,
Studio UI 또는 Windows 브라우저를 이번 검증에서 실행했다고 주장하지 않는다.

Docker 데몬 연결이 불가능해 개발 환경 안내에서 허용한 로컬 `--no-opt` 진단 경로를 사용했다.
Rust release 빌드는 성공했지만 wasm-opt까지 수행한 표준 Docker 배포 빌드의 성공 증거는 아니다.

```sh
CARGO_TARGET_DIR=/Users/tsjang/rhwp/target/issue6936-20260913 \
  scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt
node scripts/svg_native_wasm_diff.mjs \
  samples/issue6936/bold-faces.hwp samples/issue6936/bold-faces.hwpx \
  --rhwp /Users/tsjang/rhwp/target/issue6936-20260913/release-test/rhwp \
  --pkg pkg --out /private/tmp/rhwp-6936-pr-prepare-20260913/final/svg-parity --keep-match
```

WASM package와 비교 SVG/JSON는 검증용이며 커밋하지 않는다. 변경 Markdown 상대 링크 검사와
`git diff --check upstream/devel...HEAD` / `git diff --check`도 통과했다.
최신 `upstream/devel`은 기준 SHA와 같고 merge-tree 충돌은 없다.

### 남은 검증 경계

Windows 네이티브 CLI 실행, 비공개 사내 코퍼스, 대규모 PDF export 시간의 전후 비교 및 원격 CI는
미실행이다. 합성 굵게 후보 SVG에서는 usvg parse가 한 번 추가되므로 대용량 성능은 별도 관측이 필요하다.
기울임·실제 Bold·복잡 paint 회귀와 단일 text show 보존은 실행한 범위에서 통과했다.

## 공통 조판 원칙 사전 검토

| 검토 항목 | 근거 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 한컴의 `w/Tf=0.02`와 PDF Tr 2를 독립 측정했다. 실제 선택 face weight와 균일 paint를 조건으로 사용하며 파일명·문서별 분기는 없다. 실제 Bold, 혼합 span, fallback, paint 반례를 실행했다. | 충족 |
| 측정·배치 일관성 | PDF 준비 단계는 usvg의 실제 glyph face를 조회하고 원본 문자·transform을 유지한다. 새굴림은 공통 이름 규칙과 기존 New Gulim metrics를 함께 사용한다. 다른 10개 행의 glyph 원점 변화는 0이다. | 충족 |
| 줄 소속과 점유 높이 | 줄 소속·LineSeg·점유 높이 알고리즘을 변경하지 않는다. 새굴림 face 복원으로 해당 두 행의 advance만 달라지며 1페이지를 유지한다. | 비해당 |
| 사례와 증거의 독립성 | 합성 계약 테스트와 별개로 한컴 저장 HWP 및 한컴 PDF를 생성하고 12개 행을 직접 대조했다. 비공개 사내 원본 코퍼스를 재현했다고 주장하지 않는다. | 충족 |
| 기준값 변경 | 한컴 PDF와 직접 대응하는 새 HWP의 쪽수 원장 1행만 추가했다. 글꼴 교체는 명시적 change set으로 기록하며 봉인 v1와 과거 W7 baseline은 보존했다. | 충족 |
| 주장과 검증 범위 | 코드 SHA·명령·결과 및 입력/산출 SHA를 기록한다. Windows 네이티브 실행, 사내 코퍼스 및 대규모 PDF 성능은 미검증으로 남긴다. | 충족 |
