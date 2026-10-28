---
kind: guide
status: active
canonical: mydocs/manual/cli_commands.md
last_verified: 2026-10-17
---

# rhwp-q — HWP/HWPX 문서 조회·진단 CLI

`rhwp-q`는 HWP/HWPX의 텍스트·서식·개체·글꼴·페이지 배치를 조회하는 실행 파일이다.
기존 `rhwp-q-*` 실행 파일 23개의 기능을 `rhwp-q <명령>`으로 묶었다.
문서 편집·저장을 수행하지 않으며, 출력은 터미널용 텍스트 또는 스크립트용 JSON이다.

**release와 debug 빌드를 모두 지원한다.** 평소 조회에는 아래 release 예제를 사용한다.
`target/pr-review`는 이 저장소가 재사용하는 빌드 경로이고, 그 아래 `release`와 `debug`는
서로 다른 Cargo profile의 결과다. `debug/rhwp-q`만 실행할 수 있다는 제한은 없다.

## 1. 처음 실행하기

### Linux: release 빌드와 첫 조회

Rust/Cargo가 설치된 환경에서 저장소 루트로 이동한다. 아래 예제는 Ubuntu 기본 작업공간이다.
다른 Cargo 작업이 공용 target을 사용하는 중이면 끝난 뒤 빌드한다.

```bash
cd /home/tsjang/rhwp
cargo build --locked --release --target-dir /home/tsjang/rhwp/target/pr-review --bin rhwp-q

# 이 셸에서 사용할 실행 파일 경로
rhwp_q=/home/tsjang/rhwp/target/pr-review/release/rhwp-q

"$rhwp_q" --version
"$rhwp_q" --help
"$rhwp_q" text-file samples/form-01.hwp
"$rhwp_q" page-images samples/form-01.hwp --page 0 --json
```

첫 빌드는 의존성 컴파일 때문에 오래 걸릴 수 있다. 같은 profile과 target을 재사용하면
다음 빌드에서 캐시를 사용한다. 조회할 때마다 빌드할 필요는 없다. 소스를 바꾼 뒤에만 다시 빌드한다.
위 예제의 `rhwp_q`는 현재 셸의 편의 변수이며 설치 절차나 프로그램의 필수 환경 변수는 아니다.

### Windows PowerShell: release 빌드와 첫 조회

기본 작업공간이 `C:\Users\admin\rhwp`인 경우다. 경로가 다르면 두 경로를 실제 위치로 바꾼다.

```powershell
Set-Location C:\Users\admin\rhwp
$rhwpReviewTarget = 'C:\Users\admin\rhwp\target\pr-review'
cargo build --locked --release --target-dir $rhwpReviewTarget --bin rhwp-q
if ($LASTEXITCODE -ne 0) { throw 'rhwp-q 빌드 실패' }
$rhwpQ = Join-Path $rhwpReviewTarget 'release\rhwp-q.exe'
& $rhwpQ --help
& $rhwpQ page-images 'samples\form-01.hwp' --page 0 --json
```

macOS도 Linux와 같은 Cargo 옵션을 사용한다. 공용 target은 **그 host의 기본 rhwp 작업공간**
아래 `target/pr-review`로 지정한다. worktree에서 빌드할 때도 기본 작업공간의 동일한 절대
target 경로를 전달한다. 자세한 환경 준비는 [개발 환경 안내](dev_environment_guide.md)를 따른다.

### release와 debug 선택

| 용도 | 빌드 옵션 | Linux/macOS 실행 파일 | Windows 실행 파일 |
| --- | --- | --- | --- |
| 반복 조회·대규모 문서 분석 | `--release --bin rhwp-q` | `target/pr-review/release/rhwp-q` | `target\pr-review\release\rhwp-q.exe` |
| 개발·디버깅 | `--bin rhwp-q` | `target/pr-review/debug/rhwp-q` | `target\pr-review\debug\rhwp-q.exe` |

두 profile은 같은 명령·옵션·JSON 계약을 제공한다. release는 최적화된 실행 파일을 만든다.
debug는 개발 반복 빌드용이며, 이 변경에서 dev profile은 `debug=1`로 함수·파일·줄 정보 위주
역추적을 남긴다. 로컬 변수 등 상세 debugger 정보가 필요하면 다음처럼 빌드한다.

```bash
CARGO_PROFILE_DEV_DEBUG=2 cargo build --locked --target-dir /home/tsjang/rhwp/target/pr-review --bin rhwp-q
```

debug 정보 설정을 바꾸면 재컴파일이 필요할 수 있다. release/release-test 설정에는 이 dev
설정이 적용되지 않는다. 이하 Bash 예제는 첫 절에서 설정한 **release의 `$rhwp_q`**를 사용한다.

## 2. 무엇을 조회할지 고르기

| 알고 싶은 내용 | 먼저 실행할 명령 |
| --- | --- |
| 문서 전체 텍스트 | `text-file FILE` |
| 특정 쪽의 텍스트·마크다운 | `kit page-text FILE --page 0`, `markdown FILE --page 0` |
| 특정 쪽 글자 위치·글꼴·크기 | `text-layout FILE --page 0 --json` |
| 글꼴 선택 과정과 줄 프레임 | `font-layout-evidence FILE --page 0 --json` |
| 표·그림의 쪽 위 배치 | `control-layout FILE --page 0 --json` |
| 문서의 컨트롤과 개체 목록 | `objects FILE --json` |
| 쪽에 사용된 원본 그림 키 | `page-images FILE --page 0 --json` |
| 그림·수식·차트 목록 | `kit pictures FILE --json`, `kit equations FILE --json`, `kit chart-list FILE --json` |
| 글자·문단 모양 | `cursor-model FILE --json`으로 주소 확인 후 `char-shape` / `para-shape` |
| 용지 크기·여백 | `page-def FILE --section 0 --json` |
| 특정 좌표의 문서 위치 | `hit-test FILE --page 0 --x 120 --y 150 --json` |
| 표 행·열 등 원본 모델 속성 | `pack tables-model FILE --json`, `more table-spacing FILE --json` |

이 표의 `FILE`은 실제 HWP/HWPX 파일 경로로 바꾼다. 공백이 있는 경로는 인용한다.

```bash
"$rhwp_q" text-file "/자료/2025 행정업무운영 편람.hwpx" --json
```

일반 문서 변환·렌더링은 [`rhwp` CLI](cli_commands.md), 별도 에이전트 도구는
[에이전트 toolkit 안내](agent_toolkit_cli.md)를 사용한다. 이 도구의 JSON 좌표만으로 한컴과의
시각 일치 여부를 판정하지 않는다. 출력 비교는 [Visual Sweep](verification/visual_sweep_guide.md)을 따른다.

## 3. 공통 사용 규칙과 주소 단위

```text
rhwp-q <단건 명령> <파일> [명령 옵션]
rhwp-q kit <명령> <파일> [명령 옵션]
rhwp-q more <명령> <파일> [명령 옵션]
rhwp-q pack <명령> <파일> [명령 옵션]
```

- `<...>`는 필수 값, `[...]`는 생략 가능한 옵션이라는 **설명 표기**다. 꺾쇠·대괄호를 입력하지 않는다.
- `--json`을 붙이면 stdout에 JSON 객체를 출력한다. 기본 텍스트 출력의 형식은 명령마다 다르다.
- 루트 `--help`는 명령 목록, `--version`은 버전이다. 세부 옵션은 명령 뒤 `--help`로 확인한다.
- 명령별 parser가 독립적이므로 아래 예제처럼 `--page 0` 형태를 사용한다.
  모든 명령에서 `--page=0`·옵션 축약·루트 `--json`을 지원한다고 가정하지 않는다.
- 파일 경로는 현재 작업 디렉터리 기준이다. 실행 파일의 디렉터리를 기준으로 찾지 않는다.
- HWP/HWPX는 같은 명령으로 조회한다. 암호화·손상·미지원 입력은 실행 오류가 날 수 있다.

```bash
"$rhwp_q" --help
"$rhwp_q" text-layout --help
"$rhwp_q" kit --help
"$rhwp_q" kit chart-csv --help
"$rhwp_q" more --help
"$rhwp_q" pack --help
```

### 번호·좌표를 혼동하지 않기

| 인자/값 | 의미 |
| --- | --- |
| `--page` | 조판된 쪽 번호. **0 = 첫 쪽**, 1 = 둘째 쪽 |
| `--section` | 원본 구역 번호. 0부터 시작하며 쪽 번호와 다름 |
| `--list` | 한글 커서 좌표계의 리스트 ID. `cursor-model`의 root/lists로 확인하며 section 번호로 대체하지 않음 |
| `--para` | 해당 명령이 사용하는 구역 또는 리스트 안의 문단 번호. 0부터 시작 |
| `--pos` | 한글 커서 좌표계의 스트림 위치. 컨트롤도 위치를 차지하므로 표시 문자열의 문자 수와 다를 수 있음 |
| `--offset` | `cursor-rect` 등의 문자 오프셋. `--pos`와 같은 단위라고 가정하지 않음 |
| `--ci` | 해당 문단의 컨트롤 인덱스. `controlIndex` 등으로 확인 |
| `--cell`, `--cell-para` | 셀 인덱스와 셀 내부 문단 인덱스. 0부터 시작 |
| `--chart` | `kit chart-csv`의 차트 번호. **1 = 첫 차트**. `chart-list`로 먼저 확인 |
| `--slot` | more/pack의 진단 슬롯. 0부터 49까지 |
| `--x`, `--y` | 쪽 내부의 배치 좌표. 기본 96 DPI의 px 좌표이며, 스크린샷 배율·브라우저 스크롤을 더한 좌표와 다름 |
| `page-def`의 폭·높이·여백 | JSON `units: "HWPUNIT"`. px가 아님 |

본문 시작점은 `cursor-model`의 `root.topPos`로 확인한다. 셀 등 하위 리스트의 주소는
`lists`의 `listId`, `hostListId`, `sectionIndex`, `hostPara`, `controlIndex`, `cellIndex`를
함께 확인한다. `listCount`가 `lists` 배열 길이와 같다고 가정하지 않는다. 본문 정보는 별도 `root`에도 있다.

### 개별 명령의 상세 도움말

`rhwp-q <명령> --help`와 `rhwp-q kit|more|pack <명령> --help`는 명령·그룹·사용법·설명·
필수 파일 인자·옵션의 필수/선택 여부·실행 예시·출력·종료 코드를 표시한다.
도움말은 입력 파일 없이 볼 수 있다. 예를 들어:

```bash
"$rhwp_q" page-images --help
"$rhwp_q" kit cursor-in-hf --help
"$rhwp_q" more table-spacing --help
```

`page-images --help`의 주요 항목은 다음과 같다. 실제 출력에는 JSON 식별자·종료 코드도 있다.

```text
명령: page-images
그룹: query

사용법:
  rhwp-q page-images <파일> --page <N> [--json]

설명:
  쪽에서 사용하는 원본 그림 키 조회

인자:
  <파일>  조회할 HWP/HWPX 파일 (필수; 공백이 있으면 인용)

옵션:
  -h, --help  도움말 표시
  --page <N>  0부터 세는 쪽 번호; 0은 첫 쪽 (필수)
  --json  stdout에 JSON 객체 출력; 기본은 명령별 텍스트 (선택)

예시:
  rhwp-q page-images --help
  rhwp-q page-images "입력.hwp" --page 0 --json
```

예시의 `입력.hwp`와 개체 주소·키·ID는 실제 문서의 값으로 바꾼다. 도움말의 파일명은 자동 생성되는 fixture가 아니다.

## 4. 단건 조회 명령 20개

모든 행은 `rhwp-q` 다음에 붙이는 구문이다. `<파일>`은 필수이며 모든 명령에 `[--json]`을 붙일 수 있다.

| 구문 | 조회 결과와 주의점 |
| --- | --- |
| `text-file <파일> [--cp949]` | 스캔 순서의 전체 텍스트. 기본 UNICODE, `--cp949`는 CP949 밖 문자를 `&#N;` 수치 참조로 표시 |
| `markdown <파일> --page <N>` | 해당 쪽 마크다운. `--page` 필수, JSON에서는 `markdown` 필드 |
| `scan-items <파일> [--limit <N>]` | 스캔 차례의 항목. limit은 1 이상, 생략하면 제한 없이 조회 |
| `cursor-model <파일>` | 한글 커서의 root와 하위 리스트 지도 |
| `section-starts <파일>` | 구역 시작 본문 문단 번호 |
| `page-caret <파일>` | 각 쪽에서 캐럿이 설 수 있는 첫 자리 |
| `char-shape <파일> --list <N> --para <N> --pos <N>` | 커서 자리의 `charShape`. 없는 자리는 빈 객체일 수 있음 |
| `para-shape <파일> --list <N> --para <N>` | 해당 문단의 `paraShape`. 없는 자리는 빈 객체일 수 있음 |
| `cursor-rect <파일> --section <N> --para <N> --offset <N>` | 지정 문서 위치의 캐럿 사각형 |
| `hit-test <파일> --page <N> --x <F> --y <F>` | 쪽 좌표의 문서 위치와 캐럿 정보 |
| `text-layout <파일> --page <N>` | `runs`의 텍스트·좌표·크기·서식. 빈 run도 줄 상자 정보를 가질 수 있음 |
| `font-trace <파일> --page <N> [--max-characters <N>]` | 글꼴 결정 추적. 문자 제한 기본 1024, 허용 범위 1~4096 |
| `font-layout-evidence <파일> --page <N> [--max-characters <N>]` | 같은 쪽 tree에서 글꼴 추적·줄 frame·stored-row 판정을 함께 조회. 문자 제한은 위와 같음 |
| `control-layout <파일> --page <N>` | 해당 쪽 표·그림 등의 `controls` 배치 |
| `form-info <파일> --section <N> --para <N> --ci <N>` | 양식 개체 정보. 해당 개체가 없으면 `found=false`로 성공 |
| `page-def <파일> --section <N>` | 구역 용지 설정·여백. 단위 HWPUNIT |
| `page-images <파일> --page <N>` | 원본 그림 참조 `keys`. 이미지 파일을 디스크에 저장하는 명령은 아님 |
| `page-items <파일> [--page <N>]` | 조판 항목. **page 생략 시 모든 쪽** |
| `objects <파일>` | 컨트롤 사슬 `controls`와 개체 목록 `objects` |
| `object-cycle <파일>` | 개체 선택 순환 순서 `cycle` |

### 텍스트·배치를 파일로 저장하기

실행 결과는 `--output` 대신 셸 리다이렉션으로 저장한다. 먼저 디렉터리를 만든다.

```bash
mkdir -p output/query-demo
"$rhwp_q" text-file samples/form-01.hwp > output/query-demo/text.txt
"$rhwp_q" markdown samples/form-01.hwp --page 0 --json > output/query-demo/markdown.json
"$rhwp_q" text-layout samples/form-01.hwp --page 0 --json > output/query-demo/layout.json
"$rhwp_q" page-items samples/form-01.hwp --json > output/query-demo/all-page-items.json
```

`text-layout`의 `runs`는 글자 하나씩의 배열이 아니다. 각 run에 `text`, `x`, `y`, `w`, `h`,
`fontFamily`, `fontSize`, `secIdx`, `paraIdx`, `charStart`, `charX` 등이 들어간다.
원본 서식은 `char-shape`, 실제 쪽 위 배치는 `text-layout`, 글꼴 결정 이유는 `font-trace`로 확인한다.

### 커서 주소에서 서식으로 이어 조회하기

```bash
"$rhwp_q" cursor-model samples/form-01.hwp --json
"$rhwp_q" objects samples/form-01.hwp --json
"$rhwp_q" char-shape samples/form-01.hwp --list 0 --para 0 --pos 0 --json
"$rhwp_q" para-shape samples/form-01.hwp --list 0 --para 0 --json
"$rhwp_q" cursor-rect samples/form-01.hwp --section 0 --para 0 --offset 0 --json
"$rhwp_q" hit-test samples/form-01.hwp --page 0 --x 100 --y 100 --json
```

이 샘플의 첫 문단에는 구역·단 정의 컨트롤이 있다. `pos=0`의 모양 조회 예제와 실제 본문
문자 시작 위치를 구분한다. 다른 문서에는 위 주소를 그대로 적용하지 말고 먼저 주소를 조회한다.

### 글꼴 진단

```bash
"$rhwp_q" font-trace samples/form-01.hwp --page 0 --max-characters 128 --json
"$rhwp_q" font-layout-evidence samples/form-01.hwp --page 0 --max-characters 128 --json
```

실행 host의 설치 글꼴과 공급 경로는 실제 글꼴 선택·배치 결과에 영향을 줄 수 있다.
`--max-characters`는 진단 문자 범위를 제한하는 옵션이다. 페이지 수나 글꼴 설치를 지정하지 않는다.
같은 스냅샷의 추적·프레임을 함께 볼 때는 `font-layout-evidence`를 사용한다.

## 5. kit — 검색·개체·커서·차트 조회 50개

### 자주 쓰는 조합

```bash
"$rhwp_q" kit pictures samples/form-01.hwp --json
"$rhwp_q" kit equations samples/form-01.hwp --json
"$rhwp_q" kit chart-list samples/form-01.hwp --json
"$rhwp_q" kit search-all samples/form-01.hwp --q '명령' --json
"$rhwp_q" kit layer-tree samples/form-01.hwp --page 0 --json
"$rhwp_q" kit overflow-cells samples/form-01.hwp --json
```

차트 CSV는 `chart-list`에 실제 차트가 있는 문서에서 실행한다. 첫 차트는 `--chart 1`이다.
차트가 없는 `form-01.hwp`에서 CSV를 요청하면 성공 예제가 아니라 실행 오류가 된다.

```bash
# 문서.hwpx는 차트가 들어 있는 실제 파일 경로로 바꾼다.
"$rhwp_q" kit chart-list "문서.hwpx" --json
"$rhwp_q" kit chart-csv "문서.hwpx" --chart 1 > output/query-demo/chart.csv
"$rhwp_q" kit chart-csv "문서.hwpx" --chart 1 --json > output/query-demo/chart.json
```

계열마다 라벨 또는 X 값이 달라 하나의 CSV로 안전하게 표현할 수 없는 차트는 오류를 반환한다.

### 그림 키와 원본 바이트

`page-images`는 키 목록, `source-image-bytes`는 그 키의 원본 데이터 조회다. 키는 추측하지 않고
조회 결과를 사용한다. 다음 예제는 실제 첫 쪽 그림이 있을 때만 바이트 JSON을 만든다.

```bash
"$rhwp_q" page-images "그림이 있는 문서.hwp" --page 0 --json
# 위 결과 keys의 실제 값으로 그림키를 바꾼다.
"$rhwp_q" kit source-image-bytes "그림이 있는 문서.hwp" --key '그림키' --json
"$rhwp_q" kit source-image-bytes "그림이 있는 문서.hwp" --key '그림키' --include-bytes --json
```

기본 출력은 `present`, `mime`, `len` 메타데이터다. `--include-bytes`를 추가하면 데이터가
존재할 때 `bytesBase64`도 담는다. `kit bin-data --index N`도 같은 옵션을 제공한다.
두 옵션은 문서 안의 원본 바이트를 조회하며 페이지를 PNG로 렌더링하지 않는다.

검색은 `--ignore-case`로 대소문자 구분을 끄고, `--include-cells`로 셀 내용을 포함할 수 있다.
`cursor-in-hf`는 `--header`/`--footer` 중 하나와 `--apply-to`를 필수로 받는다.
apply-to는 0(양쪽)·1(짝수)·2(홀수)이고, 선택 `--page`는 대표 쪽 힌트다.
`hit-in-hf` / `hf-edit-target`의 header/footer 선택은 생략하면 머리말이다.
`fn-selection-rects`의 index와 선택 시작/끝 인자는 생략 시 각각 0이다.
`canvaskit-preflight`의 mode는 default/compat(기본 default), profile은
screen/print/high-quality/fast-preview(기본 screen)이다.

### kit 전체 명령

아래 표의 각 행은 `rhwp-q kit` 다음에 붙인다. 모든 행에 `[--json]`을 추가할 수 있다.
`N`은 정수, `F`는 실수다. `source-image-bytes`와 `bin-data`의 `--include-bytes`는 선택 옵션이다.

| 구문 | 조회 내용 |
| --- | --- |
| `empty-doc <파일>` | 빈 문서 여부 (IsEmpty) |
| `cell-shape <파일> --list <N>` | 셀 CellShape 파라미터셋 |
| `overlay-images <파일> --page <N>` | 쪽 overlay 그림 |
| `flow-images <파일> --page <N>` | 쪽 본문 흐름 그림 ops |
| `footnote-info <파일> --page <N> --index <N>` | 쪽 각주 참조 좌표 |
| `footnote-hit <파일> --page <N> --x <F> --y <F>` | 좌표가 각주 영역인지 |
| `footnote-footholds <파일> --page <N>` | 쪽에 각주 발판이 있는지 |
| `note-edit-info <파일> --section <N> --para <N> --ci <N>` | 각주·미주 편집 대상 정보 |
| `cursor-in-cell <파일> --section <N> --para <N> --ci <N> --cell <N> --cell-para <N> --offset <N>` | 셀 안 캐럿 사각형 |
| `cursor-in-hf <파일> --section <N> --header\|--footer --apply-to <N> --para <N> --offset <N> [--page <N>]` | 머리말·꼬리말 캐럿 사각형 |
| `cursor-in-footnote <파일> --page <N> --index <N> --para <N> --offset <N>` | 각주 안 캐럿 사각형 |
| `cursor-in-note <파일> --section <N> --para <N> --ci <N> --note-para <N> --offset <N>` | 노트 안 캐럿 사각형 |
| `hit-header <파일> --page <N> --x <F> --y <F>` | 머리말·꼬리말 히트테스트 |
| `hit-in-hf <파일> --page <N> --x <F> --y <F> [--header\|--footer]` | 머리말·꼬리말 내부 히트 |
| `hit-in-footnote <파일> --page <N> --x <F> --y <F>` | 각주 내부 히트 |
| `hit-fn-marker <파일> --page <N> --x <F> --y <F>` | 본문 각주 표식 히트 |
| `hf-edit-target <파일> --page <N> [--header\|--footer]` | 머리말·꼬리말 편집 대상 |
| `fn-selection-rects <파일> --page <N> [--index <N>] [--start-para <N>] [--start-pos <N>] [--end-para <N>] [--end-pos <N>]` | 각주 선택 사각형 |
| `field-info-at <파일> --list <N> --para <N> --pos <N>` | 커서 자리 필드 정보 |
| `field-by-id <파일> --id <N>` | 필드 id로 현재 값 |
| `field-list-json <파일>` | 한글 필드 리스트 JSON |
| `cur-field-state <파일> --list <N> --para <N> --pos <N>` | 커서 자리 필드 상태 비트 |
| `form-at <파일> --page <N> --x <F> --y <F>` | 좌표의 양식 개체 |
| `form-value <파일> --section <N> --para <N> --ci <N>` | 양식 개체 값 |
| `para-bounds <파일> --list <N> --para <N>` | 문단 경계 |
| `line-starts <파일> --list <N> --para <N>` | 문단 줄 시작 오프셋 |
| `word-starts <파일> --list <N> --para <N>` | 문단 단어 시작 |
| `word-end <파일> --list <N> --para <N> --pos <N>` | 단어 끝 오프셋 |
| `caret-stops <파일> --list <N> --para <N>` | 캐럿 정지 위치 |
| `char-index <파일> --list <N> --para <N> --pos <N>` | 스트림 위치의 글자 인덱스 |
| `page-text <파일> --page <N>` | 한글 스캔 쪽 텍스트 |
| `bin-data <파일> --index <N> [--include-bytes]` | 임베드 바이너리 길이·종류 |
| `source-image-bytes <파일> --key <키> [--include-bytes]` | 원본 그림 바이트 메타 |
| `overflow-cells <파일>` | 넘친 셀 줄 수 |
| `table-overlaps <파일>` | 표 겹침 목록 |
| `page-border-fill <파일> --section <N>` | 구역 쪽 테두리·배경 |
| `covering-pages <파일> --section <N> --para <N>` | 문단이 걸친 쪽 |
| `chart-csv <파일> --chart <N>` | 차트 데이터 CSV |
| `chart-list <파일>` | 문서 안 차트 목록 |
| `search-all <파일> --q <문자열> [--ignore-case] [--include-cells]` | 전 문단 검색 좌표 |
| `canvaskit-preflight <파일> [--mode default\|compat] [--profile screen\|print\|high-quality\|fast-preview]` | CanvasKit 문서 사전점검 |
| `layer-tree <파일> --page <N>` | 쪽 레이어 트리 |
| `hyperlinks <파일>` | 하이퍼링크 목록 (읽기) |
| `equations <파일>` | 수식 목록 (읽기) |
| `pictures <파일>` | 그림 목록 (읽기) |
| `hidden-comments <파일>` | 숨은 설명 목록 (읽기) |
| `index-marks <파일>` | 찾아보기 표식 (읽기) |
| `auto-numbers <파일>` | 자동번호 (읽기) |
| `ruby <파일>` | 덧말 (읽기) |
| `page-hide <파일>` | 감추기 컨트롤 (읽기) |

## 6. more / pack — 원본 모델 속성 조회

`more`는 문단·컨트롤의 특정 속성/조건을 조회하고, `pack`은 개체 종류와 원본 모델 정보를
묶어서 조회한다. 각 그룹은 일반 조회 50개와 `volume-probe` 1개를 제공한다.
쪽 배치 좌표가 필요하면 `text-layout` / `control-layout`을 사용한다.

```bash
"$rhwp_q" more section-para-lens samples/form-01.hwp --json
"$rhwp_q" more para-empty samples/form-01.hwp --json
"$rhwp_q" pack tables-model samples/form-01.hwp --json
"$rhwp_q" pack forms-all samples/form-01.hwp --json
```

위 샘플의 `section-para-lens`는 `items: [{"paraCount": 13, "section": 0}]`를 반환한다.
`tables-model`은 표가 없으므로 `count: 0, items: []`로 성공한다. 양식 개체와 표를 혼동하지 않는다.

현재 more/pack 일반 조회는 구역의 본문 문단을 순회한다. 중첩 셀·머리말 내부까지 모두
재귀 순회한다고 가정하지 않는다. 기본 텍스트 출력은 항목 수이며 상세 값은 `--json`으로 본다.
이름에 `only`·`nonempty` 등이 있어도 현재 필터가 없는 명령은 아래 표와 개별 도움말에 제한을 적었다.

### more 전체 일반 조회

아래 명령은 모두 `rhwp-q more <명령> <파일> [--json]`으로 실행한다.

| 명령 | 조회 내용 |
| --- | --- |
| `para-empty` | 빈 문단 전수 |
| `para-has-ctrl` | 컨트롤 있는 문단 |
| `section-para-lens` | 구역별 문단 수 |
| `body-text-len` | 본문 글자 수 |
| `ctrl-per-para` | 문단당 컨트롤 수 |
| `table-border-fill` | 표 테두리/배경 id |
| `table-spacing` | 표 구조·속성 조회. 현재 셀 간격값은 반환하지 않는다. 반환 필드의 표 구조·속성만 판독한다. |
| `table-attr` | 표 속성 비트 |
| `picture-border-width` | 그림 테두리 두께 |
| `picture-opacity` | 그림 테두리 투명도 |
| `picture-href-set` | 그림 href 유무 |
| `equation-baseline` | 수식 기준선 |
| `equation-color` | 수식 색 |
| `equation-attr` | 수식 attr |
| `form-height` | 양식 높이 |
| `form-caption` | 양식 캡션 |
| `form-fore-color` | 양식 글자색 |
| `field-properties` | 필드 properties |
| `field-ctrl-id` | 필드 ctrl_id |
| `ruby-align` | 덧말 정렬 |
| `ruby-pos` | 덧말 위치 |
| `pagehide-border` | 테두리 감추기 |
| `pagehide-fill` | 배경 감추기 |
| `pagehide-master` | 바탕쪽 감추기 |
| `autonumber-super` | 자동번호 위첨자 |
| `char-overlap-border` | 겹침 테두리 |
| `hyperlink-text-len` | 링크 표시 길이 |
| `bookmark-empty-name` | 빈 책갈피 이름 |
| `header-nonempty` | 머리말 문단 수 조회. 현재 빈 항목도 포함한다. paraCount를 호출자가 확인한다. |
| `footer-nonempty` | 꼬리말 문단 수 조회. 현재 빈 항목도 포함한다. paraCount를 호출자가 확인한다. |
| `footnote-nonempty` | 각주 문단 수 조회. 현재 빈 항목도 포함한다. paraCount를 호출자가 확인한다. |
| `endnote-nonempty` | 미주 문단 수 조회. 현재 빈 항목도 포함한다. paraCount를 호출자가 확인한다. |
| `hidden-nonempty` | 숨은 설명 문단 수 조회. 현재 빈 항목도 포함한다. paraCount를 호출자가 확인한다. |
| `shape-height` | 그리기 높이 |
| `table-zones` | 표 영역 수 |
| `table-grid-len` | 표 그리드 길이 |
| `field-extra-props` | 필드 extra_properties |
| `form-back-color` | 양식 배경색 |
| `equation-version` | 수식 버전 정보 |
| `index-both-keys` | 첫째+둘째 키 길이 |
| `para-char-count` | 문단 문자 수 |
| `section-ctrl-total` | 구역 컨트롤 합 |
| `caption-para-count` | 표 구조·속성 조회. 현재 캡션 문단 수는 반환하지 않는다. 캡션 정보는 pack tables-model의 captionParagraphs로 확인한다. |
| `enabled-forms-only` | 양식 개체와 활성 상태 조회. 현재 enabled=true만 거르는 필터는 적용하지 않는다. items의 enabled를 호출자가 확인한다. |
| `nonempty-urls` | 하이퍼링크 URL·텍스트 조회. 현재 빈 URL을 제외하지 않는다. url/urlLen을 호출자가 확인한다. |
| `nonempty-scripts` | 수식 스크립트와 속성 조회. 현재 빈 스크립트를 제외하지 않는다. script를 호출자가 확인한다. |
| `lock-pictures-only` | 그림 잠금 상태·속성 조회. 현재 잠긴 그림만 거르지 않는다. lock을 호출자가 확인한다. |
| `picture-crop-left` | 그림 crop.left |
| `form-name-len` | 양식 이름·속성 조회. 현재 이름 길이 필드는 반환하지 않는다. name의 길이는 호출자가 계산한다. |
| `field-command-len` | 필드 명령·속성 조회. 현재 명령 길이 필드는 반환하지 않는다. command의 길이는 호출자가 계산한다. |

### pack 전체 일반 조회

아래 명령은 모두 `rhwp-q pack <명령> <파일> [--json]`으로 실행한다.

| 명령 | 조회 내용 |
| --- | --- |
| `forms-all` | 양식 개체 전수 |
| `shapes-all` | 그리기 개체 전수 |
| `char-overlaps` | 글자겹침 전수 |
| `headers-list` | 머리말 전수 |
| `footers-list` | 꼬리말 전수 |
| `footnotes-list` | 각주 전수 |
| `endnotes-list` | 미주 전수 |
| `new-numbers` | 새 번호 전수 |
| `page-num-ctrls` | 쪽번호 시작 전수 |
| `page-number-pos` | 쪽번호 위치 전수 |
| `column-defs` | 단 정의 전수 |
| `unknown-ctrls` | 미지 컨트롤 전수 |
| `tables-model` | 표 모델 행·열 |
| `field-ctrls` | 필드 컨트롤 전수 |
| `bookmark-names` | 책갈피 이름 |
| `treat-as-char` | 글자처럼 취급 개체 |
| `logical-inline` | 논리 인라인 컨트롤 |
| `picture-crops` | 그림 자르기 좌표 |
| `equation-scripts` | 수식 스크립트 |
| `form-types` | 양식 이름·텍스트·활성·너비 조회. 현재 별도 type 필드는 반환하지 않는다. |
| `hyperlink-hosts` | 하이퍼링크 URL·텍스트 조회. 현재 URL에서 host를 추출하지 않는다. url을 호출자가 파싱한다. |
| `ruby-mains` | 덧말 본문 |
| `pagehide-headers` | 머리말 감추기 |
| `autonumber-nums` | 자동번호 값 |
| `index-second-keys` | 찾아보기 둘째 키 |
| `hidden-comment-len` | 숨은 설명 문단 수 |
| `table-rows` | 표 행 수 |
| `table-cells` | 표 셀 수 |
| `shape-sizes` | 그리기 폭 |
| `header-paras` | 머리말 문단 수 |
| `footer-paras` | 꼬리말 문단 수 |
| `footnote-paras` | 각주 문단 수 |
| `endnote-paras` | 미주 문단 수 |
| `picture-locks` | 그림 잠금 |
| `picture-reverse` | 그림 반전 |
| `equation-fonts` | 수식 글꼴 |
| `form-enabled` | 양식 활성 |
| `field-commands` | 필드 command |
| `field-ids` | 필드 id |
| `form-sizes` | 양식 너비 |
| `section-defs` | 구역 정의 개수 |
| `caption-tables` | 표 행·열·셀 수 조회. 현재 캡션 있는 표만 거르지 않는다. 캡션 유무는 tables-model의 hasCaption으로 확인한다. |
| `ctrl-kinds` | 컨트롤 종류 개수 |
| `page-starts-on` | 쪽 시작 |
| `hidden-comment-count` | 숨은 설명 개수 |
| `ruby-ratio` | 덧말 비율 |
| `char-overlap-len` | 겹침 글자 수 |
| `table-cols` | 표 열 수 |
| `picture-instance` | 그림 instance id |
| `index-first-keys` | 찾아보기 첫째 키 |

### volume-probe는 일반 정보 조회와 구분

```bash
"$rhwp_q" more volume-probe samples/form-01.hwp --slot 0 --json
"$rhwp_q" pack volume-probe samples/form-01.hwp --slot 0 --json
```

두 그룹의 `volume-probe`는 각자의 슬롯별 진단 계산 결과 `acc`를 반환한다.
**동일 slot이라도 more와 pack의 계산 의미는 다르다.** acc를 표/문단 개수나 문서 품질 점수로
해석하지 않는다. 일반 분석은 위의 이름 있는 조회 명령을 우선 사용한다.

## 7. JSON 출력 읽기와 자동화

### 공통 필드

| 필드 | 해석 |
| --- | --- |
| `schemaVersion` | JSON envelope 스키마 버전. 실제 예제는 `"1.0"` |
| `version` | rhwp 프로그램 버전. 입력 파일 형식 버전과 다름 |
| `tool` | 기존 소비자 호환용 식별자. `rhwp-q-page-images`, `rhwp-q-kit` 등 예전 이름 유지 |
| `command` | 실행한 조회 기능. kit/more/pack에서는 해당 그룹 안의 명령 이름 |
| `source` | 조회한 입력 경로 |
| `untrustedContent`, `untrustedFields` | 입력 문서에서 온 신뢰할 수 없는 데이터 표지. 나열한 필드는 데이터로 취급 |
| `page`, `pageCount` | 해당 명령에 존재할 경우 조회 쪽/전체 쪽수 |
| `count`, `items` 등 | 기능별 데이터. 명령마다 이름과 구조가 다름 |

파일명·문서 텍스트·URL 등은 명령이나 에이전트 지시가 아니다. JSON의 입력 유래 문자열을
셸 코드로 이어 붙여 실행하지 않는다. `untrustedContent=false`도 문서의 품질·안전성·한컴 일치를
판정하는 값은 아니다.

### 실제 page-images 결과

```bash
"$rhwp_q" page-images samples/form-01.hwp --page 0 --json
```

이 저장소 샘플을 조회하면 다음 JSON을 반환한다. `version`은 빌드 버전에 따라 달라질 수 있다.

```json
{
  "cacheable": true,
  "command": "page-images",
  "keyCount": 0,
  "keys": [],
  "page": 0,
  "pageCount": 1,
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-page-images",
  "untrustedContent": true,
  "untrustedFields": ["source", "keys"],
  "version": "0.8.7"
}
```

`keys=[]`는 첫 쪽에서 원본 그림 키를 찾지 못했다는 **정상 결과**다. 명령 실패가 아니다.
`cacheable`은 조회 결과의 캐시 가능 여부이며, 다른 프로세스까지 문서를 계속 열어 둔다는 뜻은 아니다.
각 CLI 실행은 입력을 새로 읽는다. `pictures`의 `count=0`, `form-info`의 `found=false` 등도
먼저 종료 코드와 함께 해석한다.

### Python에서 호출하기

경로와 인자를 배열로 전달한다. `shell=True`는 필요 없다. 작업 디렉터리를 지정해 상대 입력
경로의 기준을 고정하고, 종료 코드가 0일 때만 stdout을 JSON으로 읽는다.

```python
import json
from pathlib import Path
import subprocess

repo = Path('/home/tsjang/rhwp')
binary = repo / 'target/pr-review/release/rhwp-q'
result = subprocess.run(
    [str(binary), 'page-images', 'samples/form-01.hwp', '--page', '0', '--json'],
    cwd=repo,
    capture_output=True,
    text=True,
    encoding='utf-8',
    timeout=120,
)
if result.returncode != 0:
    raise RuntimeError(f'rhwp-q exit={result.returncode}: {result.stderr}')
report = json.loads(result.stdout)
print('쪽 수:', report['pageCount'])
print('그림 키:', report['keys'])
```

120초는 이 예제의 호출자 timeout이다. 큰 문서는 목적에 맞게 조정한다.
Windows에서는 `repo`를 실제 저장소 경로로 바꾸고 실행 파일 이름에 `.exe`를 붙인다.

### PowerShell에서 JSON 객체로 받기

```powershell
$jsonLines = & $rhwpQ page-images 'samples\form-01.hwp' --page 0 --json
if ($LASTEXITCODE -ne 0) { throw 'page-images 조회 실패' }
$report = ($jsonLines -join "`n") | ConvertFrom-Json
$report.pageCount
$report.keys
```

JSON 파일의 UTF-8 저장 방식은 PowerShell 버전에 따라 다르다. Windows PowerShell 5.1에서
`>`의 기본 인코딩을 UTF-8이라고 가정하지 않는다. 위처럼 객체로 파싱하거나 명시적으로
UTF-8 저장을 지정한다. 다른 프로그램에 JSON을 넘길 때는 stderr를 stdout에 합치지 않는다.

## 8. 종료 코드와 문제 해결

| 종료 코드 | 의미 | 대응 |
| --- | --- | --- |
| `0` | 성공. 빈 배열·빈 문자열·찾지 못함도 정상 결과일 수 있음 | 결과 필드 확인 |
| `1` | 파일 읽기·문서 적재·조회·출력 등 실행 오류 | stderr와 파일/주소/출력 경로 확인 |
| `2` | 명령·옵션·필수 인자 등 사용법 오류 | 해당 명령의 `--help` 확인 |

루트 명령 생략·알 수 없는 루트 명령도 2다. 성공한 JSON 조회는 stdout에 JSON만 출력한다.
오류 메시지는 stderr를 확인한다. 셸 리다이렉션은 실패해도 빈 결과 파일을 만들 수 있으므로
파일 존재 여부만으로 성공을 판정하지 않는다.

```bash
mkdir -p output/query-demo
"$rhwp_q" text-layout samples/form-01.hwp --page 0 --json \
  > output/query-demo/layout.json 2> output/query-demo/layout.stderr.log
rhwp_query_status=$?
if [ "$rhwp_query_status" -ne 0 ]; then
  cat output/query-demo/layout.stderr.log >&2
fi
```

| 증상 | 확인할 내용 |
| --- | --- |
| `No such file or directory` / 실행 파일 없음 | `--release`로 빌드했으면 `release/rhwp-q`, 생략했으면 `debug/rhwp-q`. Windows는 `.exe` |
| `Permission denied` | 실행 파일 권한, 입력 읽기 권한, 출력 디렉터리 권한 |
| 입력 파일을 찾지 못함 | 현재 작업 디렉터리와 파일명 인용 확인. 절대 입력 경로로 재확인 |
| `--page` 오류 | 첫 쪽은 0. `pageCount`보다 작은 번호인지 확인 |
| 차트 0번/차트 없음 오류 | `chart-list`로 먼저 확인. CSV 차트 번호는 1부터 |
| 알 수 없는 옵션 | 루트 뒤에 기능을 먼저 지정. 모든 기능에 공통 `--output`·`--dpi`가 있는 것은 아님 |
| `keys=[]` / `items=[]` | 해당 조회 대상이 없을 수 있음. exit 0이면 오류와 구분 |
| 한글·글꼴·줄바꿈이 기대와 다름 | 입력 모델과 설치 글꼴, `char-shape` / `font-trace` / 실제 한컴 출력 대조 |
| JSON 파싱 실패 | 먼저 exit 코드 확인. 기본 텍스트 출력에 `--json`을 빠뜨렸는지, stderr를 합쳤는지 확인 |

`rhwp-q`는 조회용이며 `export-png` / `export-svg` / 편집·저장 명령을 제공하지 않는다.
페이지 이미지 출력은 `rhwp` CLI를, 비교 이미지와 review/overlay 생성은 Visual Sweep을 사용한다.

## 9. 기존 실행 파일과 스크립트 이관

| 이전 실행 | 현재 실행 |
| --- | --- |
| `rhwp-q-page-images FILE --page 0 --json` | `rhwp-q page-images FILE --page 0 --json` |
| `rhwp-q-font-trace FILE --page 0 --json` | `rhwp-q font-trace FILE --page 0 --json` |
| `rhwp-q-text-layout FILE --page 0 --json` | `rhwp-q text-layout FILE --page 0 --json` |
| 나머지 단건 `rhwp-q-<기능> ...` | `rhwp-q <기능> ...` |
| `rhwp-q-kit <명령> ...` | `rhwp-q kit <명령> ...` |
| `rhwp-q-more <명령> ...` | `rhwp-q more <명령> ...` |
| `rhwp-q-pack <명령> ...` | `rhwp-q pack <명령> ...` |

호출 스크립트는 **실행 파일 경로와 선행 기능 인자를 함께** 바꾼다.
예를 들어 `subprocess.run([old_binary, file, '--json'])`에서 경로만 바꾸면 동작하지 않는다.
`[new_binary, 'objects', file, '--json']`처럼 기능 이름을 넣어야 한다.

kerning 측정 스크립트의 `--qkit`에는 이제 단일 `rhwp-q` 경로를 전달한다. 해당 스크립트가
`kit layer-tree` 인자를 붙인다. font 측정 스크립트에서 지원하는 `--native-bin`,
`--layout-bin`, `--text-layout-bin`도 단일 실행 파일 경로를 받으며, 각 스크립트가
`font-trace` / `text-layout` 등을 붙인다. 스크립트별 전체 필수 옵션은 해당 `--help`로 확인한다.

JSON의 `tool` 값은 호환성을 위해 예전 이름을 유지한다. 예전 이름이 JSON에 나온다는 이유로
그 실행 파일을 다시 호출하거나 JSON 값을 새 실행 파일 이름으로 일괄 치환하지 않는다.

## 10. 빌드 결과와 참고 문서

Cargo의 루트 bin은 `rhwp`, `rhwp-q`, `rhwp-agent`, `font-metric-gen` 네 개다.
`font-metric-gen`은 저장소 개발 도구이며 `--features dev-tools`를 명시해야 빌드·설치된다.
주 CLI만 설치하려면 `cargo install rhwp --bin rhwp`로 대상을 선택한다.
`rhwp-q`는 하위 명령 adapter를 직접 호출하며 예전 실행 파일을 subprocess로 띄우지 않는다.
기존 CLI 계약 검사도 단일 실행 파일을 사용한다.

기존 공용 target의 과거 `rhwp-q-*` 파일은 Cargo target 통합만으로 자동 삭제되지 않는다.
예전 실행 파일이 남아 있어도 새 소스의 빌드 대상이라는 의미는 아니다. 또한 `debug/deps`에는
Cargo가 관리하는 의존성·테스트 파일이 계속 존재한다. 공용 캐시 전체를 지우거나 최상위
실행 파일과 `deps` 하드링크의 크기를 중복 합산하지 않는다.

- 제품 CLI 명령: [cli_commands.md](cli_commands.md)
- 빌드·공용 target·디스크 정리: [dev_environment_guide.md](dev_environment_guide.md)
- 에이전트용 다른 도구: [agent_toolkit_cli.md](agent_toolkit_cli.md)
- 시각 비교: [visual_sweep_guide.md](verification/visual_sweep_guide.md)
- 단일 실행 파일 진입점: [src/bin/rhwp-q/main.rs](../../src/bin/rhwp-q/main.rs)
- 기존 kit 명령 정의: [kit/main.rs](../../src/bin/rhwp-q/kit/main.rs)
- more / pack 정의: [more/main.rs](../../src/bin/rhwp-q/more/main.rs), [pack/main.rs](../../src/bin/rhwp-q/pack/main.rs)
