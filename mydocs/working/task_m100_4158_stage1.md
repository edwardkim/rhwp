# Stage 1 — task_m100_4158 구현·집중 검증

- **이슈**: [#4158](https://github.com/edwardkim/rhwp/issues/4158)
- **계획서**: [`mydocs/plans/task_m100_4158.md`](../plans/task_m100_4158.md)
- **브랜치**: `task_m100_4158_char_overlap_boxed_pua`
- **분기 기준**: `upstream/devel` `5119ea498`
- **작업 시각**: 2026-08-07 KST

## 1. 진단

`samples/basic/issue2007_nested_cell_pagination_42065.hwp` 물리 10쪽의 `공정거래위원회` 앞 표식은
다음 IR을 가진 실제 `CharOverlap`이다.

```json
{
  "text": "\udb80\udeb1",
  "bbox": {"x": 80.467, "y": 160.933, "width": 13.333, "height": 17.333},
  "charOverlap": {"borderType": 0, "innerCharSize": 0},
  "style": {"fontFamily": "굴림", "fontSize": 13.333}
}
```

#4139의 물리 2쪽 보정은 `charOverlap=null`인 일반 `TextRun`의 Canvas2D 경로만 다뤘다. 실제
`CharOverlap` 세 backend는 `U+F02B1`을 숫자로 바꾸지 않고 raw PUA를 폰트에 전달했고,
Canvas2D에서는 글꼴에 따라 tofu가 출력됐다.

## 2. 구현

`src/renderer/mod.rs`에 `boxed_pua_char_overlap_semantics`를 추가했다. 단일 문자
`U+F02B1..U+F02C4`만 1..20으로 해석하며 raw border 0을 사각형(3)으로 승격한다. 작성된 1..4
테두리와 다중 문자 겹침은 기존 경로를 보존한다.

Canvas2D, SVG, Native Skia가 이 헬퍼로 같은 `effective_border`와 표시 숫자를 사용한다. IR의
raw PUA와 `borderType=0`은 바꾸지 않았다. `composer.rs`의 텍스트 표면 주석도 새 렌더 계약과
일치하도록 정리했다.

## 3. 래칫

### Rust

- 1·20 경계, 범위 밖 문자, 다중 문자 제외, 명시적 border 보존 단위 테스트
- SVG 실제 출력에서 `<rect>`와 `>1</text>`를 단언하고 raw `U+F02B1` 부재 확인
- #4085의 일반 `borderType=0` 무테두리·본문 크기 계약은 특수 PUA와 분리해 유지

### WASM Canvas2D

`rhwp-studio/e2e/issue-4158-char-overlap-boxed-pua-canvas2d.test.mjs`가 실제 HWP 물리 10쪽에서
다음을 단언한다.

- 총 17쪽
- render tree의 raw `U+F02B1`, `charOverlap.borderType=0`
- `공정거래위원회` 문맥
- 대상 중심 `fillText("1")`
- 대상 중심 raw PUA `fillText` 없음
- 대상 bbox의 `strokeRect`

`MANIFEST.md`와 `package.json`의 `e2e:issue-4158` 배선도 함께 추가했다.

## 4. 집중 검증 결과

| 검증 | 결과 |
| --- | --- |
| `cargo test --lib boxed_pua_char_overlap -- --nocapture` | PASS, 2 passed |
| `cargo test --lib char_overlap_without_border_keeps_body_font_size -- --nocapture` | PASS, 1 passed |
| `cargo test --profile release-test --features native-skia --lib boxed_pua_char_overlap` | PASS, 2 passed; Native Skia 경로 포함 컴파일 |
| `cargo clippy --lib -- -D warnings` | PASS |
| `cargo fmt --check`, `git diff --check` | PASS |
| 표준 release WASM build | PASS, wasm-bindgen·wasm-opt·packaging 완료 |
| `npm run e2e:issue-4158` | PASS, 7개 계약 |
| `npm run e2e:issue-536` | PASS, 기존 물리 2쪽 6개 계약 |
| 실제 HWP 물리 10쪽 `export-svg` | PASS, 17쪽 유지·`<rect>`+숫자 1·raw PUA 없음 |

로컬 셸에는 `wasm-pack`이 없어 0.13.1과 Cargo.lock의 `wasm-bindgen-cli 0.2.125`를
`/tmp/rhwp-4158-tools`에 설치했다. 홈 캐시가 읽기 전용이어서 기존
`/home/edward/dev/emsdk/upstream/bin/wasm-opt`를 PATH에 포함해 표준 빌드를 완주했다.

## 5. 시각 증적

`output/4158/`:

- `hancom-p010.png` — 17쪽 정답지 PDF의 물리 10쪽
- `issue2007_p010_char_overlap_canvas2d.png` — 수정 후 WASM Canvas2D 전체 쪽
- `issue2007_p010_char_overlap_crop.png` — 대상 사각형+숫자 1 crop
- `render_tree_010.json` — 실제 E2E render tree
- `svg/issue2007_nested_cell_pagination_42065_010.svg` — 실제 SVG 출력

정답지와 수정 후 출력 모두 `공정거래위원회` 앞에 사각형 안 숫자 1이 보인다. SVG의 대상은
다음과 같다.

```xml
<rect x="80.47" y="162.93" width="13.33" height="13.33" .../>
<text x="87.13" y="169.60" ...>1</text>
```

## 6. 전체 PR 검증 결과

집중 결과를 보고한 뒤 작업지시자 승인을 받아 `local_validation.md` 4.3의 renderer 전체 게이트를
실행했다.

| 검증 | 결과 |
| --- | --- |
| `CARGO_INCREMENTAL=0 cargo build --release` | PASS |
| `CARGO_INCREMENTAL=0 cargo test --release --lib` | PASS, 3,292 passed / 10 ignored / 0 failed |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | PASS, 모든 통합 테스트 0 failed |
| Native Skia 공식 3종 | PASS, 58 + 2 + 4 passed |
| `cargo fmt --check`, `git diff --check` | PASS |
| `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` | PASS |
| `CARGO_INCREMENTAL=0 cargo test --doc` | PASS, 4 passed / 2 ignored |
| `rhwp-studio: npx tsc --noEmit` | PASS |
| `rhwp-studio: npm test` | PASS, Node 22에서 765 passed / 0 failed |
| 표준 release WASM build | PASS, compile·wasm-bindgen·wasm-opt·`pkg` packaging 완료 |
| 새 WASM의 `e2e:issue-4158` / `e2e:issue-536` | PASS, 7 + 6개 계약 |
| `e2e:manifest-check` | PASS, tracked 86개 / manifest 86행 |

샌드박스 내부의 Node 22/24에서는 `spawnSync` 자식 stdout이 비어 결과 마커 기반 Studio 테스트
5개가 실패했지만, 같은 Node 22 명령을 샌드박스 밖에서 실행하면 765개가 모두 통과했다. 직접
드라이버도 정상 마커를 출력하므로 제품 회귀가 아니라 실행 격리의 pipe 캡처 현상으로 판정했다.

원격 push·PR·이슈 comment는 아직 수행하지 않았다.

## 7. 2026-08-08 최신 devel 병합·재검증

- 최신 기준: `upstream/devel` `5a4f26d0d`
- 병합 commit: `5356207db`
- 수동 충돌 해소: `mydocs/orders/20260807.md`, `rhwp-studio/e2e/MANIFEST.md`
- 해소 원칙: #4158 기록과 이미 병합된 #4159 기록·E2E 등록을 모두 보존

현재 head에서 다음 집중 게이트를 재실행했다.

| 검증 | 결과 |
| --- | --- |
| `cargo test --lib boxed_pua_char_overlap -- --nocapture` | PASS, 2 passed |
| `cargo test --lib char_overlap_without_border_keeps_body_font_size -- --nocapture` | PASS, 1 passed |
| `cargo test --profile release-test --features native-skia --lib boxed_pua_char_overlap` | PASS, 2 passed |
| `CARGO_INCREMENTAL=0 wasm-pack build --target web --out-dir pkg` | PASS, release compile·wasm-bindgen·wasm-opt·packaging 완료 |
| `npm run e2e:issue-4158` | PASS, 물리 10쪽 7개 계약 |
| `npm run e2e:issue-536` | PASS, 물리 2쪽 6개 계약 |
| `npm run e2e:issue-4159` | PASS, 물리 3쪽 2개 계약 |
| `npm run e2e:manifest-check` | PASS, tracked 87개 / manifest 87행 |
| `cargo fmt --check`, `git diff --check` | PASS |

WASM 도구는 `wasm-pack 0.13.1`, `wasm-bindgen 0.2.125`, `wasm-opt 122`를 사용했다. 새
Canvas2D crop에서 `공정거래위원회` 앞 표식이 tofu가 아닌 사각형 안 숫자 1로 출력되는 것을
확인했다. 현재 head의 전체 PR 게이트는 이 집중 결과 보고 뒤 별도 승인받아 실행한다.

작업지시자가 2026-08-08 rhwp-studio에서 물리 10쪽 `공정거래위원회` 앞 사각형 안 숫자 1을
확인해 #4158 시각 판정을 통과시켰다. 같은 검증에서 발견된 별도 `U+F02FB` tofu 문제는
#4158 범위에 섞지 않고 독립 작업으로 분리한다.
