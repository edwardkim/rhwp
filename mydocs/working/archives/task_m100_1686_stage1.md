# task m100 1686 stage1: pr-1674 RowBreak 페이지 경계 보정

## 기준선

- GitHub Issue: #1686
- 기준 브랜치: `upstream/devel`
- 대상 샘플:
  - `samples/hwpx/pr-1674.hwpx`
  - `samples/pr-1674.hwp`
- 시각 기준:
  - HWPX: `pdf-2020/pr-1674-2020.pdf` (한컴 2020 PDF, 35쪽)
  - HWP: `pdf/pr-1674-2024.pdf` (한컴 2024 PDF)
- page count 기준:
  - HWPX: 35쪽
  - HWP: 35쪽

## 재현

`devel` 기준 page 3에서 `0.27` 문단의 두 번째 co-anchored 표(`[응시자격요건 고려사항]`)가
`0.28`의 `다. 우대요건 등 [원서접수 마감일 기준]`보다 먼저 배치됐다.
또한 HWP 출력은 한컴 2024 PDF 기준 35쪽이어야 하나 rhwp가 36~37쪽으로 밀리고,
page 5가 기준 PDF처럼 `동일 기간에 경력이 중복될 경우...`로 시작하지 못했다.

- 시작 page count:
  - HWPX: 36쪽
  - HWP: 37쪽
- RED:
  - `cargo test --profile release-test --test issue_1686 -- --nocapture`
  - HWPX/HWP 모두 page 3에서 `다.우대요건등[원서접수마감일기준]` 누락으로 실패

## 구현

- `TypesetState`에 같은 문단의 후행 표를 임시 보류하는 `deferred_table_controls` 큐를 추가했다.
- 빈 host 문단의 비-TAC `TopAndBottom`/`Para` 기준 `RowBreak` 표가 분할 continuation을 만들면,
  같은 문단의 후행 양수 offset `RowBreak` 표를 즉시 배치하지 않고 보류한다.
- 보류된 표는 뒤쪽 표 문단 처리 후 또는 문서 마지막 flush 직전에 다시 조판한다.
- 적용 범위는 아래 조건으로 제한했다.
  - host 문단에 visible text 없음
  - 비-TAC 표
  - `TextWrap::TopAndBottom`
  - `VertRelTo::Para`
  - `TablePageBreak::RowBreak`
  - 보류 대상 후행 표는 `vertical_offset > 0`
- RowBreak 표 셀 내부의 첫 LINE_SEG `vpos`가 0이 아니더라도 0~500HU 범위이면 셀 내부
  로컬 좌표계 시작으로 보고 문단/라인 vpos reset을 인식하도록 제한 가드를 추가했다.
- HWP RowBreak 분할 행 overflow 허용치를 2px로 조정했다. `pr-1674`의 1.7px 수준 미세 초과는
  기준 PDF와 같이 같은 페이지에 수용하되, `kps-ai`처럼 33px 이상 초과하는 기존 회귀 케이스는
  다음 페이지로 넘긴다. HWPX 경로의 64px 허용치는 기존 보정 범위로 유지했다.
- 선행 RowBreak 표 조각 뒤에 남는 빈 guide 문단이 이전 좌표계의 큰 vpos를 들고 다음 실질
  앵커보다 아래에 기록되는 경우 flow 높이로 누적하지 않도록 흡수했다.
- visible host text가 있는 비-TAC `TopAndBottom` RowBreak 표는 첫 조각 앞에 host text를
  먼저 렌더하지 않고, 마지막 continuation 조각 뒤에 렌더하도록 조정했다.

## 결과

패치 후 `release-test` binary 기준 page 3:

```text
PartialTable   pi=27 ci=0  rows=4..7  cont=true
FullParagraph  pi=28  "다. 우대요건 등 [원서접수 마감일 기준]"
Table          pi=29 ci=0
PartialTable   pi=27 ci=1  rows=0..3  cont=false
```

최종 page count:

- HWPX: 35쪽 (PDF 오라클 35쪽과 일치)
- HWP: 35쪽 (`pdf/pr-1674-2024.pdf` 35쪽과 일치)

HWP page 5:

- 기준 PDF처럼 `동일 기간에 경력이 중복될 경우 유리한 경력 1개만 인정함`으로 시작한다.
- `[우대요건 등 고려사항]` 표와 page 5 하단 `대학 조교... 포함` 문단까지 같은 흐름으로 배치된다.
- `☞ 임용예정직위, 응시자격요건 및 우대요건 등 관련 사항은...` 안내문은 page 5에 나오지 않고
  RowBreak 표가 끝난 뒤 page 7에 배치된다.

## 검증

- `cargo fmt`
- `/usr/bin/time -p env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo test --profile release-test --test issue_1686 -- --nocapture`
  - 4 passed
- `env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo build --profile release-test --bin rhwp`
  - 통과
- `target/release-test/rhwp info samples/hwpx/pr-1674.hwpx`
  - 35쪽
- `target/release-test/rhwp info samples/pr-1674.hwp`
  - 35쪽
- `pdfinfo pdf/pr-1674-2024.pdf`
  - Creator: `Hwp 2024 13.0.0.3622`
  - Pages: 35쪽
  - page 3에 `다. 우대요건 등 [원서접수 마감일 기준]`와 우대요건 표가 먼저 배치되는 것을 확인
- `target/release-test/rhwp dump-pages samples/hwpx/pr-1674.hwpx -p 2`
  - page 3 순서 정상
- `target/release-test/rhwp dump-pages samples/pr-1674.hwp -p 2`
  - page 3 순서 정상
- `target/release-test/rhwp export-text samples/pr-1674.hwp -o /tmp/rhwp-1686-text -p 4`
  - page 5 첫 문장: `동일 기간에 경력이 중복될 경우 유리한 경력 1개만 인정함`
  - page 5에 `☞ 임용예정직위...` 안내문 없음
- `target/release-test/rhwp export-text samples/pr-1674.hwp -o /tmp/rhwp-1686-text-page7 -p 6`
  - page 7에 `☞ 임용예정직위...` 안내문 배치 확인
- `target/release-test/rhwp export-pdf samples/pr-1674.hwp -o /tmp/rhwp-1686-visual-fixed/pr-1674-rhwp.pdf`
  - 35페이지 PDF 생성
  - page 22의 기존 `LAYOUT_OVERFLOW` 진단 1건은 관찰됐으나 #1686의 page 5 경계와 총 페이지 수에는 영향 없음
- `pdftoppm -f 5 -l 5 -png -r 120 pdf/pr-1674-2024.pdf /tmp/rhwp-1686-visual-fixed/ref-page`
- `pdftoppm -f 5 -l 5 -png -r 120 /tmp/rhwp-1686-visual-fixed/pr-1674-rhwp.pdf /tmp/rhwp-1686-visual-fixed/rhwp-page`
  - 기준/생성 page 5 PNG 시각 비교: page 시작 문장, `[우대요건 등 고려사항]` 위치, page 하단 흐름 일치 확인
- `env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo test --profile release-test --test issue_1686 --test issue_1156_rowbreak_fragment_fit -- --nocapture`
  - 7 passed
- `env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo test --profile release-test --test issue_1510 --test issue_1535 --test issue_1639 --test issue_1086 --test issue_1488_rowbreak_empty_overlay_pages --test issue_713 -- --nocapture`
  - 14 passed
- `/usr/bin/time -p env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo test --profile release-test --tests`
  - 전체 통합 회귀 테스트 통과
  - 실패/회귀 없음
  - real 172.25s
- `env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target wasm-pack build --target web --out-dir pkg`
  - 통과
  - `/Users/tsjang/rhwp/pkg` 갱신
- `node --input-type=module -e '...'`
  - 최신 `pkg/rhwp.js` + `pkg/rhwp_bg.wasm`을 직접 로드
  - `samples/pr-1674.hwp` WASM `HwpDocument.pageCount()`: 35
  - WASM `getPageRenderTree(4)` page 5 텍스트:
    - `동일기간에경력이중복될경우유리한경력1개만인정함` 포함
    - `임용예정직위,응시자격요건및우대요건등관련사항은` 미포함
- rhwp-studio WASM 시각 검증
  - 선행 page 3 검증은 headless Puppeteer 경로로 수행했다.
  - `samples/hwpx/pr-1674.hwpx` page 3 캔버스 캡처: `pdf-2020/pr-1674-2020.pdf`와 동일하게 `다. 우대요건...`/우대요건 표가 `[응시자격요건 고려사항]`보다 먼저 표시됨.
  - `samples/pr-1674.hwp` page 3 캔버스 캡처: `pdf/pr-1674-2024.pdf` 기준으로 동일 순서 확인.
  - WASM render tree 순서 확인:
    - HWPX: `다.우대요건등` index 1343 < `[응시자격요건고려사항]` index 2187
    - HWP: `다.우대요건등` index 1343 < `[응시자격요건고려사항]` index 2187
  - console error/pageerror: 없음
  - 캡처 산출물: `/tmp/rhwp-1686-visual/`
