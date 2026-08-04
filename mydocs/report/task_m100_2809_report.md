# Task #2809 처리 결과 보고서 — 나눔정렬 마지막 줄과 문단 속성 차이

- 일자: 2026-07-22
- 이슈: [#2809](https://github.com/edwardkim/rhwp/issues/2809)
- 브랜치: `task/2809-distribute-align`
- 기준: `upstream/devel` `58991a768`
- 상태: 완료

## 1. 결론

HWP5 `Alignment::Split`(HWPX `DISTRIBUTE_SPACE`, 한컴 UI의 나눔 정렬)을
일반 `Justify`와 같은 마지막 줄 규칙으로 처리하던 결함을 정정했다. 한 줄짜리
`다 같 이` 문단도 내부 두 공백에 남은 폭을 배분한다. 일반 `Justify`의 마지막 줄과
두 정렬의 강제 줄바꿈 억제는 기존 동작을 유지한다.

추가 실화면 검토에서 최초 문제는 단순 잘림이 아니라 위쪽 음수 자간 문단의 glyph만
가로로 눌려 아래쪽과 다르게 보이는 것임을 확인했다. 위쪽은 자간 `-50%`/`6972HU`,
아래쪽은 자간 `0%`/`6872HU`다. glyph 폭만 정상화하면 마지막 `이`가 셀 우측에서
잘리므로, glyph 폭 유지와 마지막 잉크 여유 예약을 결합해야 했다.

최종 원인은 Canvas 2D의 glyph 폭 맞춤이었다. 음수 자간으로 줄어든 cluster advance에
브라우저 실측 glyph 폭을 강제로 맞추면서 위쪽 한글만 가로로 눌렸다. 음수 자간에서는
이 폭 맞춤을 생략해 glyph 외형을 유지했다. 그 결과 정상 폭 마지막 glyph가 셀을
넘지 않도록 `Split` 분배 계산에는 마지막 glyph의 실제 잉크 여유도 함께 예약했다.

검토 중 visual sweep의 `--dpi`가 PDF에만 적용되고 SVG에는 적용되지 않는 문제도
확인해, SVG에 `dpi / 96` zoom을 적용하도록 함께 정정했다.

## 2. 원인과 수정

### 렌더러

기존 판정은 `Justify | Split`을 한 분기로 묶고 마지막 줄이면 공백 분배를 껐다.
이슈 첨부 문단은 `ParaShape ID 6`, HWP5 정렬값 `5`인 `Split`이므로 한 줄 문단에서
분배가 비활성화됐다.

`needs_word_distribution` 헬퍼로 두 정렬을 분리했다.

- `Split`: 강제 줄바꿈이 아니면 마지막 줄도 공백 분배.
- `Justify`: 기존처럼 마지막 줄(머리말·꼬리말 예외)과 강제 줄바꿈에서 억제.
- 일반 `Justify`와 그 밖의 정렬, 원문 자간과 LineSeg는 기존 값을 유지.
- 음수 자간 `Split`은 마지막 glyph 실제 잉크 폭을 분배 영역 안에 예약.
- Canvas 2D는 음수 자간에서 glyph를 cluster advance 폭으로 축소하지 않음.

### visual sweep

`export-svg`의 width/height는 unitless CSS px(96dpi)라 `rsvg-convert --dpi-*`만으로는
출력 크기가 바뀌지 않는다. `--dpi` 옵션값을 `dpi / 96` zoom으로 변환해 SVG에도
적용했다. 기본 96dpi 결과는 동일하고, 144dpi 이상에서 PDF와 SVG가 같은 배율로
생성된다.

## 3. 변경 파일

| 파일 | 변경 |
|---|---|
| `src/renderer/layout/paragraph_layout.rs` | Split/Justify 마지막 줄 판정 분리, 회귀 테스트 2건 추가 |
| `src/renderer/web_canvas.rs` | 음수 자간 Canvas glyph 폭 축소 방지 |
| `tests/golden_svg/issue-617/exam-kor-page5.svg` | 한컴 2022 기준과 가까워진 `<보 기>` 나눔정렬 좌표 반영 |
| `scripts/task1274_visual_sweep.py` | `--dpi`를 SVG zoom에도 적용, 0 이하 DPI 거부 |
| `mydocs/plans/task_m100_2809*.md` | 조사·구현 계약 기록 |
| `mydocs/working/task_m100_2809_stage*.md` | 단계별 원인·검증 증적 기록 |
| `rhwp-studio/e2e/issue-2809-split-alignment.test.mjs` | 실제 백엔드, 좌표 span과 최종 Canvas glyph 픽셀 폭 회귀 검증 |
| `samples/issues/2809/jubo_20260104.hwp`, `pdf/issue-2809-*.pdf` | 테스트 원본 HWP와 정상 기준 PDF |
| `mydocs/pr/assets/task2809/*` | 최종 review PNG와 WASM E2E HTML |

## 4. 증적

### 이슈 첨부 HWP 2쪽

- 기준 PDF: HWP 2020 변환, 6쪽, SHA-256
  `a73d50620bf8fe96beaff72ba0e40cd34f396ec75de9798ac1fd0402e28f8e2b`.
- 수정 전 첫 라벨 `다/같/이` x:
  `416.2533 / 427.4533 / 438.6533px`.
- 마지막 glyph 잉크 여유를 예약하지 않은 중간 결과:
  `416.2533 / 458.9067 / 501.5600px`.
- native 위쪽 최종 결과: `416.2533 / 454.9067 / 493.5600px`,
  span `77.3067px`.
- native 아래쪽 최종 결과: `417.5333 / 455.7467 / 493.9600px`,
  span `76.4267px`.
- WASM SVG/페이지 레이어 트리 span: 위 `77.973px`, 아래 `76.693px`.
- 위쪽과 아래쪽은 원본 글자모양/LineSeg가 서로 다르다.
  - 위: 자간 `-50%`, `6972HU`, render bbox `93.0px`.
  - 아래: 자간 `0%`, `6872HU`, render bbox `91.0px`.
- 144dpi visual sweep: rhwp `845×1191`, PDF `844×1190`, 후보 `0/1`.
- rhwp Studio 2배율 Canvas: `1126×1587`, E2E assertion `7/7`.
- 수정 전 첫 `다` glyph 잉크 폭은 위 약 `15px`, 아래 `27px`였으며, 수정 후
  `28px / 28px`로 동일하다. 음수 자간은 위치에만 반영된다.
- 위쪽 마지막 `이`는 `22px` 잉크 폭이 온전히 표시되어 셀 우측 clip에 잘리지 않는다.
- rhwp 편집기 페이지 보기 100% 화면을 별도 캡처했다. PDF는 변경하지 않은 정상
  기준이다.

visual sweep의 전체 잉크 일치 보조값은 `12.95680%`다. Poppler가 한컴 전용 폰트를
대체 렌더한 영향이 커 이 수치 자체는 합격 판정으로 사용하지 않았다. 저장 LineSeg,
문자 좌표와 최종 Canvas 픽셀 폭을 함께 합격 근거로 사용한다.

### 기존 golden 영향

`exam-kor-page5.svg`의 `<보 기>`에 마지막 줄 분배 좌표를 반영했다.
HWP 2022 PDF의 좌우 괄호는 `229.44pt / 266.88pt`, 수정 SVG 환산값은
`229.53pt / 268.84pt`다. 기존 golden 오른쪽 `261.64pt`보다 기준에 가까워
의도된 변경으로 판정했다.

## 5. 검증

- `CARGO_INCREMENTAL=0 cargo test --lib` — `2512 passed; 0 failed; 7 ignored`.
- `CARGO_INCREMENTAL=0 cargo test --test svg_snapshot` — `8 passed; 0 failed`.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` — 통과.
- `cargo fmt --all --check`, `git diff --check` — 통과.
- `python3 -m py_compile scripts/task1274_visual_sweep.py` — 통과.
- visual sweep 96dpi/144dpi 실동작 — 각 `flagged=0/1`.
- `wasm-pack build --target web --out-dir pkg` — 통과.
- `cd rhwp-studio && npm run e2e:issue-2809` — assertion `7/7` 통과.
- `cd rhwp-studio && npm run build` — 통과.
- 최종 review PNG의 rhwp 패널은 실제 Studio `canvas2d` E2E 캡처를 사용하며,
  `다 같 이`의 마지막 `이`가 온전히 표시됨을 직접 확인했다.

## 6. 후속

원본 HWP, 기준 PDF, 최종 review PNG와 WASM E2E HTML의 SHA-256은
[`mydocs/pr/assets/task2809/README.md`](../pr/assets/task2809/README.md)에
고정했다. 원본 ZIP 묶음은 후속 요청에 따라 추적 대상에서 제외한다.
