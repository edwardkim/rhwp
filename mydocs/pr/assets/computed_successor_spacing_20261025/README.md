# 분할 표 뒤 문단의 앞 간격·쪽 소유 회귀 증적

## 검증 범위

구현 source는 `8dc75860cec88ddf7c3f8aeee230b2ca7e2e5537`이다. 같은 소스의
Native224와 최적화한 fresh WASM063에서 입력 8개, 독립 Print 8개, 각 18쪽을
전쪽 비교했다. 두 경로 모두 모든 쪽의 `silhouette_raw_match_percent`가 **100%**이고
전체 쪽수가 일치한다. 2px 내용 실루엣의 결과이며 엄격 RGB 일치나 전체 PR 완료를 뜻하지 않는다.

새 검사에는 아래 두 모듈을 사용한다. 기존의 정확한 조각 소유 검사는 변경하지 않았다.

- [앞 간격·쪽 수용 6건](../../../../tests/cases/verified_computed_split_successor_spacing.rs)
- [여러 쪽에 걸친 뒤 문단 2건](../../../../tests/cases/verified_computed_split_successor_multiline.rs)

두 모듈은 기존 수집기의 `--prepare`로 각각 `regression_suite_011`과
`regression_suite_027`에 수집됐다. 생성 harness·manifest·실행 로그는 커밋하지 않는다.

## 원 PR과 반례

[원 PR #8055](https://github.com/edwardkim/rhwp/pull/8055)의 앞 간격 중복 소비 주장을
[기존 독립 Print 증거](../float_fragment_frames_20261020/README.md)와 대조했다.
원 PR의 축약 입력은 `None` 정책이므로 실제 분할 표의 뒤 문단 계약을 대신하지 않는다.
여기서는 기존에 보존한 공개 API RowBreak 대조군
`../float_fragment_frames_20261020/inputs/hwpx/split-rows-60-policy-1-body-false-flow-false.hwpx`
(SHA-256 `59b53a4206de44fceb9ca261a4b9863141d273681f32bdaf35a539e054b251d2`)을 사용했다.

생성 runtime은 `62656a8c807b79b2620ee9081e88ce27d7a1194b`이다. 공개
`applyParaFormat(0, 1, { spacingBefore: ... })`로 0·30000·105000을 적용하고
HWP/HWPX로 각각 내보냈다. 105000 대조군의 여러 줄 입력에는 같은 문단의 offset 5에
`후속 문단의 앞 간격과 줄 소유를 함께 확인합니다. `를 40번 삽입했다.
일반 대조군의 뒤 본문은 `다음 문단`이며 여러 줄 입력은 공백을 제외한 844글자다.
저장 LineSeg를 수동으로 바꾸거나 기대 좌표를 입력에 주입하지 않았다.

| 대조군 | 입력 수 | 독립 Print 계약 | 검출하는 경계 |
| --- | ---: | --- | --- |
| 앞 간격 0 | 2 | 2쪽, 뒤 본문 y=310.7180 | 기존 정상 대조군 유지 |
| 앞 간격 30000 | 2 | 2쪽, 뒤 본문 y=332.2944 | 표 점유 뒤 간격의 중복 소비 방지 |
| 앞 간격 105000 | 2 | 2쪽, 뒤 본문 y=831.7486 | 측정·쪽 수용과 실제 배치의 같은 간격 소비 |
| 앞 간격 105000, 여러 줄 | 2 | 3쪽, 2쪽 8줄·3쪽 15줄 | 단 이동 후 앞 간격 복원과 원문 보존 |

## 독립 기대값과 인쇄

각 입력 자체를 MCP의 **한컴 2020 11.0.0.9136**, `PrintMethod=0`, 1-up으로
인쇄했다. PDF는 `pdf/computed-successor-spacing/`에 보존한다. 인쇄 경로는
`hancom2020_pdf_driver_one_up`이며 입력 전처리는 `scripts_disabled`다.
인쇄 job과 입력·PDF 해시는 아래 표에 기록한다. MCP endpoint·인증 값은 포함하지 않는다.

`spacing_coordinates.json`과 `multiline_coordinates.json`의 좌표·줄 텍스트는 독립 PDF에서
추출한 span bbox다. RHWP의 배치값을 기대값으로 복사하지 않았다. 좌표계는 **96 DPI,
종이 왼쪽 위 원점의 CSS px**이며 bbox는 `[x0, y0, x1, y1]`, 쪽 번호는 0부터다.

두 경로의 전쪽 raw 100%를 확인하고 같은 페이지의 review·overlay를 직접 열었다.
큰 앞 간격의 표 종료·뒤 본문, 여러 줄 문단의 2쪽 첫 줄·마지막 줄과 3쪽 이어받기·마지막 줄을
대조했다. 줄 수·줄바꿈·내용 순서와 문단 소유를 보존하고, 표 아래의 본문이 표와 겹치지 않는다.
얇은 괘선의 raster 농도와 글자의 안티앨리어싱 차이는 남는다. 기존 대조군에서 확인한
0.36pt 선의 물리 좌표·원본 폭을 유지했으며 점수를 맞추려고 선폭·팔레트·허용치를 바꾸지 않았다.

관계 검사와 함께, 두 경로 모두 98% 이상인 완료 페이지에만 독립 Print 원점의 **±2px**
검사를 적용한다. 전체 SVG 해시나 RHWP의 관측 좌표를 고정하지 않는다. 이후에도 두 경로의
해당 쪽 raw 98% 이상을 유지해야 한다.

## 실행과 재현

Native 실행 파일의 SHA-256은
`e333c70587ecf48e24632de8e6f3b843f33eb328051580442af4f46214743ca4`다.
빌드는 `CARGO_BUILD_JOBS=8`, 공용 `target/pr-review`에서
`cargo build --locked --profile release-test --features native-skia,pdf-jbig2,pdf-jpeg2000 --bin rhwp`
로 수행했다. 실제 실행 파일과 소스 지문의 일치는 빌드 receipt로 확인했다.
fresh WASM은 같은 source의 표준 release 및 실제 `wasm-opt -O`를 거친 web package다.

파일별 실제 Sweep 명령과 TSV SHA-256은 두 좌표 JSON의 `qualification`에 있다.
명령의 임시 입력 경로와 여기 보존한 입력의 바이트 동일성은 `input_sha256`으로 확인한다.
`tsv/native/`와 `tsv/wasm/`는 전체 페이지 TSV를 바이트 그대로 복사한 원본이다.
대표 2쪽과 이어받기 3쪽의 review·overlay는 `fine/{native,wasm}/`에 있다.

```bash
venv/bin/python scripts/visual_sweep.py --silhouette-only \
  --file-target <이름> <같은 입력> <같은 Print PDF> \
  --rhwp-bin <Native 실행 파일> --compat 2022 --dpi 96 --page-workers 1 \
  --font-path /usr/share/fonts --font-path /usr/local/share/fonts \
  --font-path /home/tsjang/.local/share/fonts --font-path /home/tsjang/rhwp/assets/fonts \
  --out output/pr-review/<검토명>/<이름>
# fresh WASM은 같은 명령에 --wasm-pkg <최적화한 web package>를 추가한다.
# 대표 PNG는 --silhouette-only 없이 --pages 2 또는 --pages 2,3으로 생성했다.
```

`--compat 2022`는 RHWP의 조판 프로필이며 실제 한컴 인쇄 엔진의 연도가 아니다.
실행 로그와 MCP 응답 원문은 ignored `output/pr-review/semanticist21-20261019/`에 보존했다.

동일한 정식 테스트 소스로 수정 전 Native223은 **2 PASS / 6 FAIL**, 수정 후 Native224는
**8 PASS / 0 FAIL**, ignore·누락은 0건이다. 수정 전에는 큰 간격의 추가 페이지 및 뒤 문단의
잘못된 쪽·줄 소유 또는 독립 원점 차이로 실패했다. 두 앞 간격 0 대조군은 양쪽에서 통과한다.
이는 동결 라이브러리를 사용한 정식 소스 실행이며 전체 nextest 완료 판정은 아니다.
원본 36조건·새 간격 6조건·여러 줄 2조건의 현재 전쪽 82쪽도 Native/fresh 모두 raw 100%다.
고정 devel과 전체 영향 문서의 대조·남은 Rust 실패·최종 lint·Studio 검증은 계속 진행한다.

## 입력·Print 해시와 job

| 입력 | 쪽수 | Print job | 입력 SHA-256 | PDF SHA-256 |
| --- | ---: | --- | --- | --- |
| `split-successor-before-0-hwp` | 2 | `2696c896-6ac7-4521-bce6-b19fc5b831b0` | `d740d2a55e24854b9661fe0637f6e65bb26c8e707135232926725ad54f0a67b1` | `c63023c11c463eab2da606159321a2e6aaac2cadee4bd18e807da54d063bd852` |
| `split-successor-before-0-hwpx` | 2 | `192b6ab7-6113-42dd-a420-ce69bb8ccfc3` | `7e57e813b378a08fe038f23c215506f62ea7bf1e40729d987639db578416435d` | `eb412da804951be129a81be4e8c911d9d831a199d06ff40030253749aba3a7ea` |
| `split-successor-before-30000-hwp` | 2 | `f40ee49e-2462-4ca6-b2cc-bc3b8f3380e3` | `a59de5ceeac1f0d5b60cae99fcf78a6da2f62736cf6da244a24f47b51a77f5d3` | `ad5df9c1128ab247e945381df6af3f38d70f3d73d99f4f7b6cf5e6501ad8f27e` |
| `split-successor-before-30000-hwpx` | 2 | `2ddd08bb-49b0-4081-a347-1a751bc8e586` | `7dc538af0f35e1901bd9a8901873a9a30a232970991f3e087fa16cfccaf1b436` | `78ff1a5b08d83c156f800592096674cf986bc3847de697952b662eb7e58c2813` |
| `split-successor-before-105000-hwp` | 2 | `40a8bda1-2efb-4d0f-8e44-d57e3641c052` | `4d5724f5913a431ce17c41832ac370e0fdf8b8d2488d444d6fdb7a158f1beb6d` | `e34c84fdb00aed30addf55bf75041df1618af2670d3c95888b49292fd36092e5` |
| `split-successor-before-105000-hwpx` | 2 | `a25dfa90-ba19-497c-80cd-5771af62e64b` | `dde614e735c53e3e386de686f67d32456574e73cd79fa6caa5f0e6e88518d6c0` | `53c46e84098ba8e6690d6fa678b9fde37a05eb4fcf0b837f06252ce4f96ff3b6` |
| `split-successor-multiline-before-105000-hwp` | 3 | `772cb435-a5e2-4ec2-9b47-bd5369f0b7a7` | `8b2f4ad4ef5a0c244504e39abbc8c75cfda0b82a7a0b99023b72daec8b42793c` | `c92b577635dcb291c616cf2bd25cd5f4ed4fcec54915b11cec29f67907879070` |
| `split-successor-multiline-before-105000-hwpx` | 3 | `b2483ce7-f7a3-49c2-b387-8712df43d4bb` | `3ee19675cf77dbfbaf7c5cd62c4b4e0277e4a9c4a867b069ca63a709698c99fc` | `2ffa2484ad3a42345ac02289926223f286db511d1e360f77306489d1636c5c1a` |
