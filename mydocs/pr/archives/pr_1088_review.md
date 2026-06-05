# PR #1088 검토 — 같은 문단 컨트롤 배치를 vertical_offset 기준으로 정합

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #1088 |
| 제목 | fix(typeset): 같은 문단 컨트롤 배치를 vertical_offset 기준으로 정합 — para-float 표 vs in-flow 라벨 순서 (closes #1087) |
| 작성자 | **HaimLee-4869 (Lee eunjung)** — 기존 컨트리뷰터 (6번째 PR, #1020/#1021/#1026/#1047/#1059) |
| base ← head | `devel` ← `HaimLee-4869:pr/para-control-vertical-order` (fork) |
| 연결 이슈 | Closes #1087 (#103/#157/#266 후속), assignee 본인 지정 완료 |
| mergeable | MERGEABLE / BEHIND |
| CI | ⚠️ **no checks reported** (작성자 다른 fork PR #1059도 동일 — fork 첫 푸시 패턴, 본 PR 결함 아님) |
| 변경 | `src/renderer/typeset.rs` 단일 (+119, -17) |
| 커밋 | 1 (5efd0470) |

## 2. 배경 (이슈 #1087)

`typeset.rs:2277` `typeset_table_paragraph` 의 표 루프가
`para.controls` 를 **배열 인덱스 순**으로 순회. 한컴 정답 배치
순서는 컨트롤의 **vertical_offset** 순. 배열 순서와 무관.

**증거 (PR 본문 명시):**
- 공직기강 pi407: 라벨 v_off=0, 표 v_off=+3063 → **라벨이 표 위**
- 국립국어원 pi586: 표 v_off=-1796, 라벨 v_off=0 → **표가 라벨 위**
- 둘 다 IR 배열은 `[표, 라벨]` 인데 정답 반대

#103/#157/#266 이 비-TAC wrap=위아래 표의 **위치(vpos·높이)** 는
정합했으나 **배치 순서** 는 본선이 배열 인덱스 순으로 남아 있던
잔존.

## 3. 트러블슈팅 사전 검색 (feedback_search_troubleshootings_first)

관련 문서 3건:
- `typeset_partial_table_wrap_around.md`
- `task_m100_103_attempt1_postmortem.md` (#103 시도 실패 분석 — 본 PR 직접 선행 작업)
- `typeset_page_num_newumber_application.md`

→ PR 의 진단(배치 순서 vs 위치는 별개 문제) 이 이들의 결론과 정합.
   #266 (#103 후속) 이후 미해결로 남았던 \"같은 문단 내 정렬\" 사안.

## 4. 변경 내용 (typeset.rs 한 파일)

### 4.1 정렬 키 (`float_table_voffset`)
```rust
let float_table_voffset = |ctrl: &Control| -> i32 {
    match ctrl {
        Control::Table(t)
            if !t.common.treat_as_char
                && matches!(t.common.text_wrap, TextWrap::TopAndBottom)
                && matches!(t.common.vert_rel_to, VertRelTo::Para) =>
        {
            t.common.vertical_offset as i32
        }
        _ => 0,
    }
};
```
**비-TAC + wrap=위아래 + vert=Para 인 표만** vertical_offset 사용.
나머지 in-flow/TAC/다른 wrap 은 key 0 → 배열 순서 유지.

### 4.2 안정 정렬
```rust
let mut ctrl_order: Vec<usize> = (0..para.controls.len()).collect();
ctrl_order.sort_by_key(|&i| float_table_voffset(&para.controls[i]));
```
- `sort_by_key` 는 안정 정렬 — 동률 시 배열 순서 유지 (TAC 다수
  연속 시 회귀 면 격리).
- **`ctrl_idx` 는 원래 배열 인덱스를 유지** —
  `format_table`/`measured_tables`/`PageItem` 조회가 의존.

### 4.3 `is_first_placed` / `is_last_placed`
배열순서 기반 `is_first/last_table` 을 **배치순서 기반** 으로 재산출:
```rust
let first_placed_table = ctrl_order.iter().copied().find(|&i| matches!(...));
let last_placed_table  = ctrl_order.iter().copied().rev().find(|&i| matches!(...));
```
→ pre/post 텍스트와 spacing 이 실제 배치 첫/마지막 표에 붙도록.

### 4.4 시그니처 변경
- `typeset_tac_table`, `typeset_block_table`, `place_table_with_text`
  세 함수에 `is_first_placed`/`is_last_placed: bool` 인자 추가
  (`#[allow(clippy::too_many_arguments)]` 동반)

## 5. 검토 의견

### 5.1 강점

- **root cause 진단 정밀**: 두 실제 문서(공직기강·국립국어원) 의
  반대 케이스(라벨 먼저 vs 표 먼저) 로 정렬 알고리즘 검증.
  vertical_offset 순이 한컴의 진실인 결정적 증거.
- **적용 범위 명확히 격리**: 비-TAC + wrap=위아래 + vert=Para 조합
  만 정렬 key 사용. TAC/in-flow/Paper-absolute 는 key 0 → 회귀 면 0.
- **안정 정렬 + ctrl_idx 보존**: 동률 시 배열 순서 유지 →
  다른 코드가 의존하는 ctrl_idx 가 정렬과 분리. 영리한 설계.
- **광범위 회귀 검증 (PR 본문 명시)**:
  - `cargo test --lib 1336 passed`
  - `svg_snapshot 8골든 불변`
  - `fmt·clippy 통과`
  - native + WASM 빌드 통과
  - **전 samples 276개: 페이지수·LAYOUT_OVERFLOW 변동 0**
  - 한컴 PDF 좌표 대조: 공직기강(22p)·국립국어원(35p) 정합
  - #103 원본 케이스(hwpspec sec3.pi238) 도 라벨앞으로 교정
- 주석 풍부 — 진단 근거(공직기강 v_off=+3063, 국립국어원
  v_off=-1796, pic-in-* 전부 0) 코드 내 보존.
- `task_m100_103_attempt1_postmortem.md` 의 교훈을 직접 후속 작업
  으로 흡수.

### 5.2 검토 포인트

- **CI 미실행** (\"no checks reported\"): 작성자 다른 fork PR #1059
  도 동일 — fork 첫 push 패턴. 본 PR 자체 결함 아니나 자체 검증
  으로 대체 필요. PR 본문이 매우 철저한 자체 검증 (test 1336 +
  svg 골든 + 276 샘플 회귀 0) 명시 — 사실상 CI 가 잡았을 항목
  모두 자체 검증.
- typeset core 변경이라 회귀 위험 영역이긴 하나, 적용 범위 격리
  (비-TAC + wrap=위아래 + vert=Para) 와 안정 정렬로 위험 최소화.
- 본질이 시각 판정 대상 — \"라벨/표 순서\" 가 한컴과 일치 여부.
  PR 첨부 스크린샷이 공직기강 케이스 시각 정합을 보임.

### 5.3 재검토 메모 (2026-05-27)

작성자가 `hwp-multi-001.hwp` 회귀 판정에 대해 반박 댓글을 남겨
해당 케이스를 다시 검증했다.

#### 한컴 PDF 기준 대조

권위 자료 `pdf/hwp-multi-001-2022.pdf` 1페이지에서 `pdftotext
-bbox-layout` 로 좌표를 추출했다.

```text
보도시점: yMin=122.000990
해외직접투자는: yMin=144.548828
지정학적: yMin=173.077650
```

따라서 한컴 2022 PDF 기준 정답 순서는 `보도시점` 표가 제목 박스보다
위에 배치되는 형태다.

#### local/devel vs PR SVG 비교

`samples/hwp-multi-001.hwp` 1페이지 debug overlay SVG를 비교했다.

```text
local/devel:
  제목 박스 ci=0 y=184.7, 보도시점 표 ci=1 y=306.4
  보도시점 첫 글자 y=321.49

PR #1088:
  보도시점 표 ci=1 y=157.2, 제목 박스 ci=0 y=184.7
  보도시점 첫 글자 y=172.31
```

PR 적용 후 배치가 한컴 PDF 기준과 일치한다. 이전 보고서의
`hwp-multi-001.hwp` 회귀 판정은 IR 배열 순서를 정답으로 본 오판으로
정정한다.

#### 최신 local/devel 적용성

PR head `5efd0470` 을 최신 `local/devel` 에 체리픽한 검증 브랜치
`pr1088-current` 에서 확인했다. 작업지시자 debug SVG 판정 중
`hwp-multi-001.hwp` 1페이지 `s0:pi=14` 가 표 높이만큼 내려가는
회귀를 추가로 확인했고, maintainer 보강 패치를 적용했다.

```text
cargo fmt --all -- --check: success
cargo test --lib: 1405 passed, 0 failed, 6 ignored
cargo clippy -- -D warnings: success
cargo test --test svg_snapshot: 8 passed
```

#### 보강 패치

원인:

```text
PR #1088이 같은 문단 안에서 TAC 표와 para-relative float 표의 렌더 순서를 바꿈
→ 마지막으로 처리된 항목이 TAC 가 아니게 됨
→ TAC 문단 직후 vpos 보정 skip 상태가 사라짐
→ para-relative TopAndBottom 표 host 문단의 first_vpos 가 다음 y 목표로 사용됨
→ pi=14 앞에 표 높이만큼 빈 공간 발생
```

처리:

```text
src/renderer/layout.rs
  - TAC guard 를 PageItem 단위가 아닌 host paragraph 단위로 유지

src/renderer/height_cursor.rs
  - 현재 문단이 para-relative TopAndBottom 표 host 이면 first_vpos 를 inter-item 목표 y 로 쓰지 않음
  - 표의 위치/높이는 Table PageItem 렌더 단계에서만 반영
```

검증 SVG:

```text
output/poc/pr1088-layout-debug/fixed-pr2/hwp-multi-001_001.svg

s0:pi=3  ci=1 1x4 y=157.2
s0:pi=3  ci=0 1x1 y=184.7
s0:pi=13 y=658.1
s0:pi=14 y=674.1
s0:pi=14 ci=0 4x6 y=683.6
```

작업지시자 판정: 통과.

### 5.4 재검토 판단

작성자 반박은 타당하다. `hwp-multi-001.hwp` 는 회귀가 아니라 PR 이
한컴 배치 순서에 더 가까워지는 케이스로 재분류한다. 추가로 발견된
`pi=14` vpos 회귀는 maintainer 보강 패치로 해소했다.

### 5.5 3페이지 페이지네이션 후속 점검

1페이지 보강 후 작업지시자가 `hwp-multi-001.hwp` 3페이지에서
남은 영역에 위치해야 할 표가 다음 페이지로 밀리는 현상을 추가 보고했다.

진단 결과 `pi=46` 문단은 다음 구조였다.

```text
line0: TAC 12x12 표
line1: "□ 지역별 동향" post-text
```

기존 typeset 은 단일 TAC 표의 fit 판단에 문단 전체 `height_for_fit`
을 사용했다. 그래서 표 자체는 page3 잔여 영역에 들어가지만, 뒤따르는
제목 줄까지 함께 들어가지 않는다는 이유로 표까지 page4로 이동했다.

보강 내용:

```text
src/renderer/typeset.rs
  - raw attr bit0 없이 common.treat_as_char=true 인 표도 LINE_SEG line0
    높이와 표 높이가 일치하면 effective TAC 로 취급
  - line0 TAC 표는 표 line_height만 fit 판단에 사용
  - 뒤따르는 post-text 는 필요 시 다음 페이지로 분리
  - line0 TAC 에만 적용하여 복학원서 pi=16 PUA filler 표/도장 회귀 방지
```

검증 결과:

```text
output/poc/pr1088-layout-debug/fixed-pr6-page3/hwp-multi-001_003.svg

dump-pages page3:
  body h=933.6px
  used=919.1px
  pi=46 table=12x12, y=684.7, h=326.7px

cargo test --test svg_snapshot: 8 passed
cargo test --lib: 1405 passed, 0 failed, 6 ignored
cargo clippy -- -D warnings: success
docker compose --env-file .env.docker run --rm wasm: success
작업지시자 시각 판정: 통과
```

CI 후속 확인:

```text
devel push 후 CI run 26487958667:
  tests/issue_554.rs::task554_no_regression_2025_donations_hwpx 실패
  expected page_count=30, actual=31

원인:
  samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx 의 3페이지 pi=25는
  HWPX lineSeg상 line0=표줄, line1=post-text 구조다.
  line0 TAC 표가 남은 영역에 들어가지만 line_advance(0)
  (=line_height + line_spacing)를 fit 판단에 사용해 표가 다음 페이지로 밀렸다.

조치:
  line0 TAC 표 단독 fit 판단은 line_height만 사용하도록 좁힘.

재검증:
  cargo test --test issue_554: 12 passed
  cargo test --test svg_snapshot: 8 passed
  cargo test: success
  cargo fmt --all -- --check: success
  cargo clippy -- -D warnings: success
```

남은 절차는 다음과 같다.

```text
1. PR #1088의 BEHIND 상태 해소 또는 maintainer merge 전략 결정
2. 통과 시 PR merge 및 #1087 close
```

## 6. 처리 방식 — **GitHub 머지** (작업지시자 지시)

> 작업지시자 지시: \"이번 PR 은 cherry-pick 하지 말고 머지로
> 처리. first-time contributor 가 사라질 것 같다.\"

비고: 작성자 통계상 6번째 PR (#1020/#1021/#1026/#1047/#1059 +
본 PR) 이라 \"기존 컨트리뷰터\" 분류이긴 하나, GitHub 의
\"first-time contributor\" 배지·기여 인식이 cherry-pick 처리
누적으로 사라질 우려가 있다는 작업지시자 판단 — fork base PR 의
정식 머지(`gh pr merge`) 로 author co-authored merge 기록을 GitHub
에 남기는 방향.

## 7. 검증 계획

- [x] 로컬 검증용 cherry-pick (검증 후 폐기, 머지는 GitHub 에서) —
      또는 fetch + 작업 브랜치 분리
- [x] 전체 `cargo test` + `cargo clippy -- -D warnings` +
      `cargo fmt --all -- --check`
- [x] WASM 빌드 (typeset core 변경)
- [x] svg_snapshot 골든 불변 확인
- [x] 시각 판정 (작업지시자 판단) — 공직기강·국립국어원 샘플
      라벨/표 순서, #103 원본 케이스 회귀
- [ ] BEHIND 해소 (devel update branch 또는 작업지시자 `--admin`
      merge)
- [ ] GitHub 머지 — `gh pr merge 1088 --merge` (필요 시 `--admin`)

## 7. 판단 (잠정)

root cause 정밀 + 적용 범위 격리 + 안정 정렬 + 매우 철저한 자체
검증 (276 샘플 회귀 0) + #103/#266 잔존 문제의 직접적 마무리.
fork CI 미실행은 자체 검증으로 대체 충분.

검증 + 시각 판정 통과 시 수용 권고. 최근 #1076/#1077 처럼 회귀
가드 부재로 인한 잔존 버그 위험이 본 PR 은 자체 광범위 검증
(276 샘플 + 페이지수·LAYOUT_OVERFLOW 변동 0) 으로 상당히
완화됨.

검증 결과에 따라 `pr_1088_report.md` 작성.
