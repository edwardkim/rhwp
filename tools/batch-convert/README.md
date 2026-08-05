# batch-convert — HWP/HWPX 일괄 변환 도구

입력 폴더를 재귀 탐색해 찾은 `.hwp`/`.hwpx` 문서를 rhwp CLI(`export-pdf` /
`export-png` / `export-svg` / `export-text`)로 병렬 변환하는 배치
오케스트레이터다. rhwp 라이브러리에 직접 링크하지 않고 이미 검증된 CLI 계약에
위임하므로, rhwp 내부 API 가 바뀌어도 함께 낡지 않는다.

## 요구사항

- rhwp CLI 바이너리가 필요하다. 탐색 순서:
  1. `--rhwp-bin <경로>` 로 명시한 경로
  2. `PATH` 의 `rhwp`
  3. batch-convert 실행 파일과 같은 폴더의 `rhwp(.exe)`
  4. 저장소 루트의 `target/{release,debug}/rhwp(.exe)`
- `formats.png` 을 쓰려면 rhwp 를 `native-skia` feature 로 빌드해야 한다
  (`cargo build --release --features native-skia`).

## 빌드

```bash
# 저장소 루트에서
cargo build --release -p batch-convert
```

빌드 결과는 `target/release/batch-convert`(Windows 는 `batch-convert.exe`)에
생긴다.

## 사용법

```bash
batch-convert --input-dir ./documents --output-dir ./output
batch-convert --input-dir ./documents --output-dir ./output --config ./config.pdf-only.json
batch-convert --input-dir ./documents --output-dir ./output --jobs 8
batch-convert --input-dir ./documents --output-dir ./output --pattern "^report_.*\.hwp$"
batch-convert --input-dir ./documents --output-dir ./output --dry-run --verbose
```

### 명령줄 옵션

```
USAGE:
    batch-convert [OPTIONS] --input-dir <INPUT_DIR> --output-dir <OUTPUT_DIR>

OPTIONS:
    -i, --input-dir <INPUT_DIR>      HWP/HWPX 파일이 있는 입력 폴더 (재귀 탐색)
    -o, --output-dir <OUTPUT_DIR>    변환 결과를 놓을 출력 폴더
    -c, --config <CONFIG>            변환 옵션 JSON 설정 파일
    -j, --jobs <JOBS>                병렬 worker 수, 1 이상 [기본: 4]
    -p, --pattern <PATTERN>          파일 이름 필터 (정규식)
        --rhwp-bin <RHWP_BIN>        rhwp CLI 바이너리 경로 (기본: 위 탐색 순서)
        --dry-run                    실제 변환 없이 대상만 확인
    -v, --verbose                    디버그 로깅
    -h, --help                       도움말
```

### `--jobs` 의미

`--jobs N` 은 **전용 Rayon thread pool 을 `num_threads(N)` 으로 만들어** 동시에
도는 rhwp 프로세스 수를 N 으로 상한한다. `0` 은 인자 파싱 단계에서 거부된다
(`invalid value '0' for '--jobs <JOBS>'`). 이 상한은 통합 테스트가 mock rhwp 의
동시 실행 표식 파일로 검증한다 (`tests/cli.rs`).

## 출력 구조

입력 폴더의 하위 구조는 출력에도 그대로 보존된다 (`input/2026/a.hwp` →
`output/pdf/2026/a.pdf`). PDF 는 단일 파일이고, PNG/SVG/텍스트는 rhwp 가
페이지별 파일을 쓰는 **폴더**다 (rhwp 이름 규칙: 1페이지 문서는
`<이름>.png`, 여러 페이지는 `<이름>_001.png` 식).

`behavior.create_format_dirs: true`(기본):

```
output/
├── pdf/문서1.pdf
├── png/문서1/문서1_001.png ...
├── svg/문서1/문서1_001.svg ...
└── text/문서1/문서1_001.txt ...
```

`behavior.create_format_dirs: false` — 포맷 하위 폴더 없이 출력 루트에 놓는다.
PNG/SVG/텍스트는 문서별 폴더 하나를 공유하고 확장자로 구분된다:

```
output/
├── 문서1.pdf
└── 문서1/문서1_001.png, 문서1_001.svg, 문서1_001.txt ...
```

변환에 실패한 원본은 `behavior.collect_failed: true` 일 때
`output/failed/<상대경로>` 로 복사된다.

## 설정 파일

모든 필드는 변환 경로에 실제로 연결되어 있고, 포맷 옵션은 rhwp CLI 플래그와
1:1 대응한다. **알 수 없는 필드는 파싱 단계에서 거부**되므로(unknown field
오류) 선언만 되고 동작하지 않는 설정이 조용히 살아남을 수 없다.

전체 스키마 (formats 는 필수·네 키 모두 명시, 나머지 섹션은 생략 가능):

```json
{
  "formats": {
    "pdf": true,
    "png": false,
    "svg": false,
    "text": true
  },
  "pdf": {
    "backend": "svg",
    "profile": "print",
    "raster_dpi": 144,
    "text_as_paths": false
  },
  "png": {
    "profile": "high-quality",
    "dpi": 300,
    "scale": 1.5,
    "max_dimension": 1568
  },
  "svg": {
    "profile": "print",
    "embed_fonts": false
  },
  "behavior": {
    "overwrite": true,
    "create_format_dirs": true,
    "collect_failed": false,
    "fail_fast": false,
    "max_retries": 3,
    "skip_existing": false
  }
}
```

### formats

활성화할 출력 포맷. **하나 이상 true 여야 한다.** 텍스트 내보내기는
`rhwp export-text` 에 배치 변환에서 쓸 수 있는 추가 플래그가 없어 별도 옵션
섹션이 없다 (`--json`/`--max-chars` 는 파일 저장 모드와 호환되지 않음).

### pdf — `rhwp export-pdf` 플래그 대응

| 필드 | rhwp 플래그 | 설명 |
|---|---|---|
| `backend` | `--backend <svg\|direct>` | PDF backend (생략 시 rhwp 기본값 svg) |
| `profile` | `--profile <프로필>` | `screen\|print\|high-quality\|fast-preview` |
| `raster_dpi` | `--raster-dpi <DPI>` | **direct backend 전용** fallback raster DPI |
| `text_as_paths` | `--text-as-paths` | 텍스트를 path 로 변환 (**svg backend 전용**) |

### png — `rhwp export-png` 플래그 대응 (rhwp 에 native-skia feature 필요)

| 필드 | rhwp 플래그 | 설명 |
|---|---|---|
| `profile` | `--profile <프로필>` | 출력 프로필 (rhwp 기본: high-quality) |
| `dpi` | `--dpi <값>` | PNG pHYs 메타데이터. `scale` 미지정 시 scale=dpi/96 자동 계산 |
| `scale` | `--scale <배율>` | 렌더링 배율 (rhwp 기본: 1.0) |
| `max_dimension` | `--max-dimension <픽셀>` | 긴 변 최대 픽셀 (VLM 입력 한도용) |

### svg — `rhwp export-svg` 플래그 대응

| 필드 | rhwp 플래그 | 설명 |
|---|---|---|
| `profile` | `--profile <프로필>` | layer 출력 프로필 |
| `embed_fonts` | `--embed-fonts` | 사용 글자만 폰트 서브셋 임베딩 |

### behavior

| 필드 | 기본 | 동작 |
|---|---|---|
| `overwrite` | `true` | `false` 면 이미 존재하는 산출물을 **포맷 단위로** 건너뛴다 (재작성하지 않음). 활성 포맷이 전부 건너뛰어진 파일은 Skipped 로 집계 |
| `create_format_dirs` | `true` | 포맷별 하위 폴더(`pdf/`·`png/`·`svg/`·`text/`) 생성 여부 — 위 "출력 구조" 참조 |
| `collect_failed` | `false` | 변환 실패 원본을 `<출력>/failed/` 로 복사 |
| `fail_fast` | `false` | 파일 실패가 확정되는 즉시 아직 시작하지 않은 파일을 건너뛴다 (진행 중이던 파일은 마저 끝난다) |
| `max_retries` | `3` | 포맷별 rhwp 호출 실패 시 추가 재시도 횟수 (총 시도 = 1 + N) |
| `skip_existing` | `false` | 활성 포맷의 산출물이 **전부** 존재하는 파일을 통째로 건너뛴다 |

산출물 존재 판정: PDF 는 파일 존재, PNG/SVG/텍스트는 해당 확장자 파일이 1개
이상 담긴 폴더 존재로 판정한다 (빈 폴더는 미완료로 간주해 다시 변환).

### 설정 검증

rhwp CLI 가 파일 단위로 거부할 조합은 배치 시작 전에 걸러 즉시 실패한다:

- 활성 포맷이 하나도 없으면 오류
- `pdf.backend` 는 `"svg"` 또는 `"direct"` 만 허용
- `pdf.raster_dpi` 는 `pdf.backend: "direct"` 에서만 허용
- `pdf.text_as_paths` 는 svg backend 전용 (direct 와 함께 쓰면 오류)
- `svg.profile` 과 `svg.embed_fonts` 는 동시 지정 불가 (rhwp export-svg 규칙)
- `raster_dpi`/`dpi`/`scale` 은 양수, `max_dimension` 은 1 이상

`profile` 값 자체(`screen|print|high-quality|fast-preview`)는 rhwp 가
검증한다 — 잘못된 값은 해당 파일 변환 실패로 보고된다.

## 제공 설정 예제

| 파일 | 내용 |
|---|---|
| `config.default.json` | PDF+텍스트, 기본 동작 (기본 설정과 동일) |
| `config.pdf-only.json` | PDF 만, print 프로필, 기존 산출물 보존(`overwrite: false`)+실패 수집 |
| `config.images.json` | PNG(300dpi)+SVG(폰트 임베딩), 이미 변환된 파일 건너뛰기 |
| `config.all-formats.json` | 네 포맷 전부 + 실패 수집 |
| `config.high-quality-png.json` | PNG 600dpi 아카이브용, 재시도 5회, 기존 파일 보존 |

## 결과 집계와 종료 코드

파일 하나는 활성 포맷 중 **하나 이상** 성공하면 Successful, 전부 실패하면
Failed(실패 사유 목록에 표시), 시도 없이 건너뛰면 Skipped 로 집계된다.

```
================== CONVERSION SUMMARY ==================
Total files processed: 10
Successful conversions: 8
Failed conversions: 1
Skipped files: 1
Total time: 12.34s
```

- `0`: 실패 0건으로 완료
- `1`: 1건 이상 실패, 또는 실행 오류 (설정 파일 오류 포함)
- `2`: 잘못된 명령줄 인자 (`--jobs 0` 등)

## 로깅

```bash
RUST_LOG=debug batch-convert --input-dir ./documents --output-dir ./output
# 또는
batch-convert --input-dir ./documents --output-dir ./output --verbose
```

- INFO: 진행/완료 요약, DEBUG: 파일·포맷 단위 상세, WARN: 개별 실패와 재시도

## 테스트

```bash
cargo test -p batch-convert
```

통합 테스트(`tests/cli.rs`)는 실제 rhwp 대신 보조 바이너리
`mock-rhwp`(`src/bin/mock_rhwp.rs`, 테스트 전용)를 `--rhwp-bin` 으로 주입해
검증한다. 동시 실행 수는 벽시계 시간이 아니라 mock 이 남기는 표식 파일로
판정한다: `--jobs 1` 이면 최대 동시 실행 1, `--jobs 4` 면 2 이상. 그 외
`--jobs 0` 거부, overwrite/skip_existing/fail_fast/max_retries/collect_failed/
create_format_dirs 동작, 포맷 옵션의 rhwp 플래그 전달, unknown field 거부를
회귀 테스트로 고정한다.

## 구조

- `src/main.rs` — CLI 인자 파싱과 오케스트레이션
- `src/config.rs` — 설정 스키마(`deny_unknown_fields`)·계약 검증
- `src/converter.rs` — 파일 탐색, 전용 Rayon 풀 병렬 변환, rhwp 호출
- `src/progress.rs` — 진행률·ETA 계산
- `src/bin/mock_rhwp.rs` — 통합 테스트 전용 rhwp 대역
- `tests/cli.rs` — 통합 테스트
