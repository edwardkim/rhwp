# Task M100 #1139 Stage 22

## 목적

Stage21 커밋 이후 남은 `3-09월_교육_통합_2022.hwp` 미주/페이지 overflow와 한컴 정답지 시각 정합 문제를 이어서 해결한다.

## 시작 기준

- Stage21에서 미주 내부 수식 `noteRef` 노출, 미주 수식 속성 WASM API, rhwp-studio 수식 개체 속성 경로를 연결했다.
- `cargo build`, #1139 회귀 테스트, WASM 빌드, studio production 빌드는 통과했다.
- compact 미주 흐름에서 큰 역방향 VPOS 보정을 허용하는 후보 변경이 포함되어 있으나, SVG 재출력 확인은 중단되어 최종 overflow 결과가 확정되지 않았다.
- 현재 긴 빌드/테스트/export 프로세스는 실행 중이 아니며, Vite dev 서버만 `7700` 포트에서 실행 중이다.

## 남은 문제

- 수식 개체를 선택한 뒤 `개체 속성`을 누르면 한컴처럼 `수식 속성` 대화상자가 떠야 하지만, 현재는 `수식 편집` 대화상자가 열린다.
  - `수식 편집` 명령/더블클릭 편집 경로는 기존 편집기를 유지한다.
  - `개체 속성(P)...`/`format:object-properties`/`insert:picture-props`의 수식 분기만 `수식 속성` 대화상자로 연결한다.
  - `수식 속성` 대화상자의 `편집(E)` 버튼은 기존 `수식 편집` 대화상자를 여는 보조 경로로 둔다.
- 15쪽 이후 미주/본문 배치가 한컴 정답지와 일치하는지 다시 확인해야 한다.
- 16쪽 overflow와 17~18쪽 헤더/본문 오버랩이 새 기준에서 해소됐는지 검증해야 한다.
- 12쪽의 약 0.9px draw overflow가 허용 가능한 반올림 수준인지, 실제 시각 차이인지 분리해야 한다.
- 미주 내부 수식 `개체 속성` 상호작용은 자동 테스트를 통과했지만, studio 화면에서 선택/대화상자 열림을 한 번 더 확인한다.

## 진행 계획

1. 수식 선택 상태의 `개체 속성` 명령 경로를 먼저 조사하고, 기존 `수식 편집` 경로와 분리한다.
2. 수식 개체 속성 전용 대화상자를 추가해 제목/탭/버튼 구성이 `수식 속성`으로 보이게 한다.
3. `3-09월_교육_통합_2022.hwp`의 9/12/15/16/17/18쪽 SVG 또는 PNG를 새로 출력한다.
4. export 로그와 산출물에서 overflow 위치를 확인하고, Stage21의 `height_cursor.rs` 보정 효과를 판정한다.
5. overflow가 남으면 pagination 기준 좌표와 renderer sequential cursor 좌표를 작은 디버그 출력으로 비교한다.
6. 원인이 확정되면 Stage22 수정 승인 후 최소 범위로 소스 수정한다.
7. 검증은 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`, `cargo build`, `wasm-pack build --target web --out-dir pkg`, studio 빌드와 SVG/PNG 비교로 진행한다.

## 승인 상태

- 작업지시자 승인 후 Stage22 소스 수정 및 검증 완료.
- 2026-05-29 작업지시자 지시에 따라 현재 Stage22 상태를 커밋하고 Stage23으로 전환한다.

## 진행 기록

- 이슈 #1139는 OPEN 상태이며 assignee는 비어 있다.
  - 현재 `jangster77` 계정은 `edwardkim/rhwp` assignee 지정 권한이 없으므로 assignee 지정 명령은 시도하지 않았다.
- 열린 PR은 외부 Draft PR #1159(`render-p20` → `devel`) 1건을 확인했다.
- Stage21 커밋 기준 `target/debug/rhwp export-svg`로 9/12/15/16/17/18쪽을 재출력했다.
- 2026-05-29 추가 확인:
  - 수식에서 `개체 속성`을 누르면 `수식 속성`이 떠야 하는데 현재 studio는 `수식 편집`을 연다.
  - Stage22는 이 UI 경로 분리 문제를 먼저 처리한 뒤 overflow 검증을 이어간다.
- 수식 개체 속성 경로 수정:
  - `format:object-properties`와 `insert:picture-props`의 수식 분기를 새 `EquationPropertiesDialog`로 연결했다.
  - 기존 `insert:equation-edit`와 수식 삽입 직후 편집 경로는 `EquationEditorDialog`를 계속 사용한다.
  - `수식 속성` 대화상자에는 `기본`, `여백/캡션`, `수식` 탭과 우측 `설정(D)`, `취소`, `편집(E)` 버튼을 추가했다.
  - `편집(E)`는 기존 수식 편집 대화상자를 여는 보조 경로로 동작한다.
- 수식 개체 속성 경로 수정 후 검증:
  - `npm run build` 통과.
  - `cargo fmt` 완료.
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과.
    - 10개 테스트 통과.
    - 9쪽 미주 표가 제목/머리말 영역과 겹치지 않는 회귀 테스트를 추가했다.
  - `cargo build` 통과.
  - `wasm-pack build --target web --out-dir pkg` 통과.
  - `pkg` 갱신 후 `npm run build` 재실행 통과.
- 9쪽 오버랩 재확인 및 보정:
  - 작업지시자 화면 확인에서 9쪽 상단 정답표와 미주 풀이가 겹치는 문제가 남아 있음을 확인했다.
  - 원인: 본문 정답표 뒤 첫 미주 `pi=468` 진입 시 lazy VPOS 기준점 역산이 음수였는데, `suppress_large_forward_jump` 경로가 큰 backward 보정을 허용해 `y_in=361.37`을 단 상단 `end_y=90.71`로 되감았다.
  - 수정: lazy 기준점 역산이 음수인 경우는 이전 TAC 표/개체 높이가 sequential y에 이미 반영된 상태로 보고 VPOS 보정을 건너뛴다.
  - 회귀 테스트: 9쪽 정답표(`pi=466 ci=0`)의 bottom과 첫 미주(`pi=468`)의 text y를 render tree 좌표로 비교해, 첫 미주가 표 아래에서 시작하는지 확인한다.
  - 시각 산출물:
    - `output/task1139_stage22_fixed_page9_png/3-09월_교육_통합_2022_009.png`
    - `output/task1139_stage22_fixed_page18_png/3-09월_교육_통합_2022_018.png`
  - 검증:
    - `cargo test invalid_lazy_base -- --nocapture` 통과.
    - `cargo test compact_endnote -- --nocapture` 통과.
    - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과.
    - `cargo build` 통과.
    - 9쪽/18쪽 `target/debug/rhwp export-svg` 통과, 18쪽 `LAYOUT_OVERFLOW` 없음.
    - `wasm-pack build --target web --out-dir pkg` 통과.
    - `npm run build` 통과.

## 새 기준 SVG/PNG 산출물

- 디버그 오버레이 SVG: `output/task1139_stage22_svg/`
- 디버그 오버레이 PNG: `output/task1139_stage22_png/`
- 깨끗한 SVG: `output/task1139_stage22_clean_svg/`
- 깨끗한 PNG: `output/task1139_stage22_clean_png/`

대상 PNG:

- `output/task1139_stage22_clean_png/3-09월_교육_통합_2022_009.png`
- `output/task1139_stage22_clean_png/3-09월_교육_통합_2022_012.png`
- `output/task1139_stage22_clean_png/3-09월_교육_통합_2022_015.png`
- `output/task1139_stage22_clean_png/3-09월_교육_통합_2022_016.png`
- `output/task1139_stage22_clean_png/3-09월_교육_통합_2022_017.png`
- `output/task1139_stage22_clean_png/3-09월_교육_통합_2022_018.png`

## 재검증 결과

- 9쪽: `LAYOUT_OVERFLOW` 없음.
- 12쪽: 기존과 같은 `pi=673` `LAYOUT_OVERFLOW_DRAW` 약 0.9px만 남음.
- 15쪽: `LAYOUT_OVERFLOW` 없음.
- 16쪽: `LAYOUT_OVERFLOW` 없음.
- 17쪽: `LAYOUT_OVERFLOW` 없음.
- 18쪽: `pi=952~955`에서 최대 125.9px overflow 재현.

18쪽 주요 로그:

```text
LAYOUT_OVERFLOW_DRAW: section=0 pi=952 line=0 y=1096.2 col_bottom=1092.3 overflow=3.9px
LAYOUT_OVERFLOW: page=17, sec=0, col=0, para=952, type=FullParagraph, first=false, y=1096.2, bottom=1092.3, overflow=3.9px
LAYOUT_OVERFLOW_DRAW: section=0 pi=953 line=0 y=1132.9 col_bottom=1092.3 overflow=40.6px
LAYOUT_OVERFLOW_DRAW: section=0 pi=953 line=1 y=1166.5 col_bottom=1092.3 overflow=74.2px
LAYOUT_OVERFLOW: page=17, sec=0, col=0, para=953, type=FullParagraph, first=false, y=1166.5, bottom=1092.3, overflow=74.2px
LAYOUT_OVERFLOW_DRAW: section=0 pi=954 line=0 y=1200.1 col_bottom=1092.3 overflow=107.8px
LAYOUT_OVERFLOW: page=17, sec=0, col=0, para=954, type=FullParagraph, first=false, y=1200.1, bottom=1092.3, overflow=107.8px
LAYOUT_OVERFLOW_DRAW: section=0 pi=955 line=0 y=1218.1 col_bottom=1092.3 overflow=125.9px
LAYOUT_OVERFLOW: page=17, sec=0, col=0, para=955, type=FullParagraph, first=false, y=1218.1, bottom=1092.3, overflow=125.9px
```

## 원인 후보

- `dump-pages -p 17` 기준 18쪽 1단은 `used=988.8px`로 본문 높이 1001.6px 안에 들어간다고 판정한다.
- 실제 렌더에서는 `pi=949` 이후 `pi=950`의 원본 VPOS가 `939252 → 930793`으로 크게 되감긴다.
- 렌더러의 `HeightCursor`는 이 되감김을 보고 vpos 기준점은 끊지만, y 위치는 기존 순차 진행 상태로 유지한다.
- 그 결과 `pi=950~955`가 이미 하단에 가까운 y에서 계속 누적되고, `pi=952~955`가 body bottom을 초과한다.
- 페이지네이터는 compact 미주 흐름에서 되감김/새 미주 시작을 더 압축된 높이로 계산하므로, 렌더러와 페이지네이터의 되감김 처리 방식이 18쪽 후반에서 다시 벌어진 것으로 보인다.

## Stage22 수정 방향 후보

1. compact 미주 흐름에서 큰 VPOS 되감김을 만났을 때 렌더러가 단순히 기준점만 끊지 않고, 페이지네이터의 `compute_en_metrics`와 같은 높이 기준으로 순차 y를 보정한다.
2. 적용 조건은 좁게 제한한다.
   - `col_content.endnote_flow`
   - 다단 미주
   - 현재 문단 first VPOS가 직전 문단 first VPOS보다 작음
   - 대상 문단이 미주 가상 문단 영역
   - 18쪽처럼 현재 y가 본문 하단에 근접해 이후 overflow가 확정되는 경우
3. 우선 `height_cursor.rs` 또는 `layout.rs`의 compact endnote rewind 처리만 수정하고, 일반 본문/표/그림 흐름에는 적용하지 않는다.

## 승인 요청

위 원인 후보와 수정 방향으로 Stage22 소스 수정을 진행해도 되는지 작업지시자 승인 대기.
