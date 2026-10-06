---
kind: task
status: local-validation-passed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# Task #536 Stage 1 — Canvas2D 한컴 사각 안 숫자 PUA 폴백

- Issue: [#536](https://github.com/edwardkim/rhwp/issues/536) (`Refs`, 트래킹 이슈 유지)
- 의존 PR: [#4122](https://github.com/edwardkim/rhwp/pull/4122)
- 구현 PR: [#4139](https://github.com/edwardkim/rhwp/pull/4139) (stacked Draft)
- stacked base/head: `task_m100_4069` / `task_m100_536_canvas2d_boxed_pua`
- 기준 head: `41404b4e5b6ab5d1a43e7a2e38c5744a7f87a1e5`

## 판정

`samples/basic/issue2007_nested_cell_pagination_42065.hwp` 물리 2쪽의 `U+F02B1`은 한컴에서
사각형 안 숫자 1로 출력된다. 교체 전 정답지 PDF에도 두부 글자 오류가 있어 #4122 시각 검토에서
같은 모양을 정상으로 오인했으나, 작업지시자가 정상 변환 PDF로 교체한 뒤 차이가 확인됐다.

이 결함은 #4122가 만든 pagination 회귀가 아니다. #4122 head와 그 이전 경로 모두 대상 문자를
`charOverlap=null`인 원문 `TextRun`으로 보존한다. CanvasKit은 missing-glyph 경로에서 이미
사각형과 숫자를 합성하지만 기본 renderer인 Canvas2D는 브라우저 글꼴에 raw PUA를 맡겨 두부 글자를
그렸다. 따라서 #4122 위의 독립 stacked PR로 Canvas2D backend parity만 보완한다.

## fixture와 정답지

| 역할 | 경로 | SHA-256 |
| --- | --- | --- |
| 입력 HWP | `samples/basic/issue2007_nested_cell_pagination_42065.hwp` | `bebd4ce3691246b0fb3ae332e1d40bc51d9035cddb9fc3d378466b6a8a2b5626` |
| 교체된 한컴 2020 PDF | `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf` | `9b0390f856bb9ad43337679babf6677209b7c7ab678b6616fcc6d6d5551ff1c4` |

정답지는 17쪽이며 2026-08-07 작업지시자가 한컴에서 정상 변환한 파일로 교체했다. 이전 PDF의
SHA-256은 `1f9d2f5705a64899c2b081832d2e6548dfe7bc3b9d1fb1b92f41221d39c8b3e7`이다.

## 구현 계약

- `U+F02B1..U+F02C4`만 1..20으로 해석하는 공통 bounded helper를 둔다.
- IR·텍스트 폭·SVG의 raw PUA 보존 계약은 바꾸지 않는다.
- Canvas2D만 CanvasKit과 같은 `0.72em` 사각형, `0.5em` 숫자, bounded stroke를 합성한다.
- 일반 텍스트와 effect pass에서 raw PUA를 건너뛰어 두부 글자나 이중 paint를 남기지 않는다.
- 다른 PUA, 실제 `CharOverlap`, CanvasKit·native Skia·PDF backend는 변경하지 않는다.

## 회귀와 시각 근거

- Rust unit: PUA 범위의 1·10·20과 양쪽 경계 밖 문자를 고정한다.
- Studio E2E: 실제 HWP를 WASM Canvas2D로 물리 2쪽에 렌더링한다.
- E2E는 #4122의 17쪽 계약, raw `U+F02B1`, `charOverlap=null`, 사각 외곽의 폭·높이와 내부 숫자
  잉크를 함께 확인한다.
- `cargo build --release`: 통과.
- `cargo test --release --lib`: 3,286 passed, 8 ignored, 0 failed.
- `cargo test --profile release-test --tests`: library 3,286 passed, 8 ignored, 0 failed와 모든
  integration test binary 통과. #2007 4건, IR sweep, overflow-cell baseline, SVG snapshot 포함.
- Native Skia 공식 3종: library 58, #2225 2, direct PDF 4 passed.
- `cargo fmt --check`, staged/unstaged diff check, `cargo clippy --all-targets -- -D warnings`: 통과.
- doc test: 4 passed, 2 ignored.
- Studio TypeScript: `npx tsc --noEmit` 통과.
- Studio `npm test`: sandbox 안에서는 중첩 Node driver 5개의 종료코드가 0인데 stdout이 소실돼
  155/160으로 실패했다. 동일 공식 명령을 sandbox 밖에서 재실행해 763/763 통과했으며, 소스나
  테스트는 바꾸지 않았다.
- 표준 `wasm-pack 0.15.0` web build: 통과. `pkg/rhwp_bg.wasm` SHA-256
  `79fe6bc3c22741f7c6fd293a50e42e7f60d7ec27c874e7eb4af6ef2aafe54109`.
- 최종 WASM 실제 E2E: 17쪽, 사각 잉크 33x34px(예상 30.7px), 내부 숫자 잉크 82px로 통과.

임시 산출물은 `output/536/issue2007_p002_canvas2d.png`, 저장소 대표 asset은
`mydocs/pr/assets/pr_4139_536_boxed_pua_canvas2d_p002.png`에 둔다. 수행자 확인에서는 사각형 안
숫자 1이 정상 표시된다. 작업지시자가 승인한 PR 직전 전체 검증은 완료했으며, 최종 시각 판정은
2026-08-07 통과했다. #4122 merge, #4139의 `devel` retarget와 최신 CI는 후속 게이트로 남긴다.
