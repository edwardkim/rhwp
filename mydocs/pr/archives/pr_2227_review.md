# PR #2227 리뷰 - 로컬 글꼴 다국어 매칭과 CanvasKit 등록

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | [#2227](https://github.com/edwardkim/rhwp/pull/2227) |
| 제목 | `[Font] 로컬 글꼴 다국어 매칭과 CanvasKit 등록` |
| 작성자 | `jangster77` |
| base | `devel` |
| 관련 이슈 | [#2217](https://github.com/edwardkim/rhwp/issues/2217), [#2206](https://github.com/edwardkim/rhwp/issues/2206) |
| 규모 | 문서 작성 시점 참고값: review 문서 전 29 files, +2,264/-177 |
| 작성 시점 상태 | open. GitHub Actions와 merge 가능 상태는 최종 merge 직전에 최신 head 기준으로 다시 확인한다. |

## 변경 범위

- `rhwp-studio/src/core/local-fonts.ts`
  - `queryLocalFonts()`의 family/full name/PostScript name과 OpenType Unicode name record를 합쳐
    한글/영문 alias를 안정적으로 해석한다.
  - legacy Macintosh name record와 저장 snapshot의 깨진 표시 이름을 UI 후보에서 제외하거나
    browser family/full name으로 복구한다.
  - 원본 SFNT 바이트는 동의한 현재 렌더링 세션에만 보관한다.
- `rhwp-studio/src/core/document-font-status.ts`, `font-substitution.ts`
  - 문서가 지정한 한글 글꼴명을 local record와 매칭하고, CSS에는 브라우저가 실제 해석할 canonical family를
    전달한다.
- `rhwp-studio/src/view/canvaskit-renderer.ts`, `canvas-view.ts`, `page-renderer.ts`,
  `viewport-manager.ts`, `main.ts`
  - 확인된 SFNT를 CanvasKit Typeface로 등록한다. 글꼴 확인이 끝난 뒤 입력을 활성화하고, 재감지와 이미지
    렌더를 전체 문서 재초기화가 아닌 필요한 화면 갱신으로 제한한다.
- `rhwp-studio/src/ui/toolbar.ts`, `styles/style-bar.css`
  - 한컴과 같은 범주형 글꼴 메뉴를 제공한다. 기본은 문서 글꼴이고 시스템 글꼴은 명시 선택 시에만 조회하며,
    긴 목록은 고정 높이 목록에서 스크롤된다.
- 재현 자료와 회귀 테스트
  - `samples/issue2217/20200830.hwp`, HWP 2020 기준 PDF
    `pdf/issue2217/20200830-2020.pdf`를 보존한다.
  - alias, legacy name 제외, snapshot 복구, 초기 입력 활성화, CanvasKit local face, 메뉴 lazy loading과
    문서 글꼴 보존 테스트를 추가한다.

## Findings

blocking finding은 없다.

### 비차단 잔여 - 브라우저 권한/실제 글꼴 목록 최종 확인

Local Font Access API는 Chromium의 사용자 권한과 실제 설치 글꼴에 의존한다. 단위 테스트는 Unicode SFNT
name record, alias resolver, 저장 snapshot 복구와 메뉴의 lazy/scroll 계약을 검증한다. 다만 merge 전 최신
PR head에서 `20200830.hwp`를 열어 다음 상호작용을 한 번 더 확인해야 한다.

- `08서울한강체 M`이 문서 글꼴과 현재 글꼴에서 깨지지 않은 이름으로 보이는지
- 로컬 글꼴 재감지 뒤에도 문서 글꼴 목록이 캐럿 언어별 일부 글꼴로 축소되지 않는지
- 시스템 글꼴 범주의 긴 목록이 고정 높이 안에서 스크롤되고, 재감지 후에도 편집 입력과 캐럿 이동이 가능한지

지원하지 않는 브라우저에서는 기존 웹폰트/대체 글꼴 경로를 유지하도록 설계되어 있어 이 조건은 merge
blocker가 아니라 Chromium 직접 확인 항목으로 기록한다.

## 검증 결과

PR 직전 `target/`을 제거한 콜드 상태에서 수행했다.

- `cargo fmt --all -- --check` 통과
- `CARGO_INCREMENTAL=0 cargo build --profile release-test --verbose` 통과
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests --verbose` 통과
- `CARGO_INCREMENTAL=0 cargo check --target wasm32-unknown-unknown --lib` 통과
- `CARGO_INCREMENTAL=0 cargo clippy -- -D warnings` 통과
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --features native-skia skia --lib --verbose`
  - 48 passed, 0 failed
- `wasm-pack build --target web --out-dir pkg` 통과
- `cd rhwp-studio && npm test`
  - 204 passed, 0 failed
- `cd rhwp-studio && npm run build` 통과
- `git diff --check upstream/devel...HEAD` 통과

## 기준 PDF 및 시각 검증

`samples/issue2217/20200830.hwp`를 HWP 2020 MCP CLI로 변환해
`pdf/issue2217/20200830-2020.pdf`(4쪽, 928,693 bytes)로 보존했다.
SHA-256은 `ea0110e5c6325f7d2b22620017791b8ab5e53768f84dfa2c0656390d931c6563`이며, 변환 응답은
`status=success`, `run_status=0`, `validation=ok`였다.

기준 PDF와 rhwp SVG를 4쪽 전체 비교한 자동 visual sweep은 평균 pixel match `85.10261%`,
visual accuracy proxy `6.05756%`를 기록했다. 이 값과 `render_tree_frame_tail_overflow`, line/column
band drift, content-bottom drift 후보는 기존 HWP 2020 PDF 대 CLI SVG 조판 fidelity 축이며, 이번 PR의
로컬 글꼴 alias/CanvasKit 등록과 직접 인과로 판단하지 않았다. 따라서 이 PR의 합격 기준은 페이지 픽셀
일치가 아니라 실제 Chrome에서 문서 글꼴을 해석하고 CanvasKit 렌더 경로로 연결하는지다.

## 최종 권고

[#2217](https://github.com/edwardkim/rhwp/issues/2217)의 다국어 글꼴명 해석, CanvasKit local Typeface 등록,
초기 입력 gating, 한컴형 목록 분류라는 목적은 코드·회귀 테스트·콜드 CI급 검증으로 확인됐다. 최신 head의
GitHub Actions와 위 Chromium 직접 확인을 마친 뒤 merge 후보로 권고한다. [#2206](https://github.com/edwardkim/rhwp/issues/2206)의
메트릭 fidelity는 별도 축으로 유지한다.
