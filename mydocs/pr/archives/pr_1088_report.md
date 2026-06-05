# PR #1088 처리 보고 — para-float 표 vertical_offset 정렬 (재검토: 수용 권고)

## 1. 결정

**재검토 후 수용 권고** — 이전 회귀 판정은 한컴 PDF 기준 대조 후
정정한다.

| 항목 | 값 |
|------|-----|
| 번호 | #1088 |
| 작성자 | HaimLee-4869 (Lee eunjung) — 기존 컨트리뷰터 (6번째 PR) |
| 연결 이슈 | Closes #1087 |
| 상태 | **OPEN** — PR head 동일, 작성자 반박 검증 완료 |
| 권고 | 작업지시자 시각 판정 후 GitHub merge |

## 1.1 정정 사유

2026-05-25 보고에서는 `samples/hwp-multi-001.hwp` 1페이지를
시각 회귀로 판단했으나, 작성자가 한글 2024 실측 반박을 제출했다.
이에 저장소 권위 자료 `pdf/hwp-multi-001-2022.pdf` 1페이지를
`pdftotext -bbox-layout` 로 다시 대조했다.

```text
보도시점: yMin=122.000990
해외직접투자는: yMin=144.548828
지정학적: yMin=173.077650
```

한컴 2022 PDF 기준으로 `보도시점` 표가 제목 박스보다 위에 오는 것이
정답이다. 따라서 PR 적용 후 `보도시점` 이 위로 이동하는 현상은 회귀가
아니라 정합 개선이다.

## 1.2 재검증 결과 (2026-05-27)

PR head `5efd0470` 을 최신 `local/devel` 에 체리픽한 검증 브랜치
`pr1088-current` 에서 확인했다. 이후 작업지시자 debug SVG 판정에서
`hwp-multi-001.hwp` 1페이지 `s0:pi=14` 가 표 높이만큼 과도하게
밀리는 회귀를 확인했고, maintainer 보강 패치를 추가했다.

| 항목 | 결과 |
|------|------|
| cherry-pick `5efd0470` → 최신 `local/devel` | 성공, 충돌 없음 |
| `cargo fmt --all -- --check` | 성공 |
| `cargo test --lib` | 1405 passed, 0 failed, 6 ignored |
| `cargo clippy -- -D warnings` | 성공 |
| `cargo test --test svg_snapshot` | 8 passed |

`samples/hwp-multi-001.hwp` 1페이지 SVG 비교:

| 기준 | 보도시점 표 | 제목 박스 | 판정 |
|------|-------------|-----------|------|
| `local/devel` | ci=1 y=306.4 | ci=0 y=184.7 | 한컴 PDF와 반대 |
| PR #1088 | ci=1 y=157.2 | ci=0 y=184.7 | 한컴 PDF와 정합 |

`fixed-pr2` debug overlay 기준:

```text
s0:pi=3  ci=1 1x4 y=157.2
s0:pi=3  ci=0 1x1 y=184.7
s0:pi=13 y=658.1
s0:pi=14 y=674.1
s0:pi=14 ci=0 4x6 y=683.6
```

작업지시자 시각 판정:

```text
output/poc/pr1088-layout-debug/fixed-pr2/hwp-multi-001_001.svg
→ 통과
```

## 1.2.1 보강 패치

원인:

```text
PR #1088 정렬로 같은 문단의 TAC 표가 먼저, para-relative float 표가 나중에 렌더됨
→ 마지막 PageItem 이 TAC 가 아니게 됨
→ 다음 문단에서 TAC line-seg guard 가 풀림
→ pi=14 의 first_vpos 가 inter-item 보정 목표로 사용됨
→ para-relative TopAndBottom 표 예약 높이만큼 빈 공간이 생김
```

수정:

```text
1. TAC line-seg guard 를 "직전 PageItem" 이 아니라 "해당 host 문단에서 TAC segment 렌더 여부"로 유지
2. 현재 문단이 para-relative TopAndBottom 표 host 이면 first_vpos 를 inter-item 보정 목표로 쓰지 않음
   - 표 위치/높이는 Table PageItem 렌더 단계에서 반영
```

변경 파일:

```text
src/renderer/layout.rs
src/renderer/height_cursor.rs
tests/golden_svg/issue-157/page-1.svg
```

## 1.2.2 3페이지 페이지네이션 추가 보강

1페이지 시각 판정 통과 후, 작업지시자가 `hwp-multi-001.hwp` 3페이지에서
남은 영역에 들어가야 할 표가 다음 페이지로 넘어가는 문제를 추가 보고했다.

진단:

```text
문단 pi=46:
  line0 = treat-as-char 표(12x12)
  line1 = "□ 지역별 동향" 제목 텍스트

기존 typeset:
  단일 TAC 표 fit 판단에 문단 전체 height_for_fit 사용
  → 표 + 뒤따르는 제목 줄을 한 번에 fit 해야 한다고 판단
  → 제목 줄까지는 남은 영역에 들어가지 않으므로 표까지 다음 페이지로 이동
```

수정:

```text
src/renderer/typeset.rs
  - raw TAC bit가 없더라도, common.treat_as_char 이고 LINE_SEG line0 이 표 높이와
    일치하면 effective TAC 표로 취급
  - line0 TAC 표 + line1 post-text 문단은 표 line_height만 먼저 fit 판단
    (line_spacing은 뒤따르는 텍스트 줄과 함께 처리)
  - post-text 만 현재 페이지 잔여 영역에 들어가지 않을 때 다음 페이지로 분리
  - 이 분리는 line0 TAC 표 케이스에만 적용하여 복학원서 pi=16 PUA filler 표 회귀 방지
```

검증:

```text
output/poc/pr1088-layout-debug/fixed-pr6-page3/hwp-multi-001_003.svg

dump-pages:
  page 3 body h=933.6px
  page 3 used=919.1px
  pi=46 12x12 table y=684.7, h=326.7px
```

회귀 확인:

```text
cargo fmt --all -- --check: success
cargo test --test svg_snapshot: 8 passed
cargo test --lib: 1405 passed, 0 failed, 6 ignored
cargo clippy -- -D warnings: success
docker compose --env-file .env.docker run --rm wasm: success
```

CI 후속 보정:

```text
devel push 후 CI run 26487958667에서 tests/issue_554.rs::task554_no_regression_2025_donations_hwpx 실패.
samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx 의 3페이지 pi=25 표가
다음 페이지로 밀려 page_count 30 -> 31 회귀.

원인:
  HWPX lineSeg는 line0=표줄(line_height=표 높이+outer margin),
  line1=post-text로 분리되어 있었지만, fit 판단에 line_advance(0)
  (=line_height + line_spacing)를 사용해 표 자체는 남은 영역에 들어가는데도
  spacing 때문에 페이지를 넘겼다.

보정:
  line0 TAC 표 단독 fit 판단은 line_height만 사용하고,
  line_spacing/post-text는 후속 텍스트 줄 처리에 맡긴다.

검증:
  cargo test --test issue_554: 12 passed
  cargo test --test svg_snapshot: 8 passed
  cargo test: success
  cargo fmt --all -- --check: success
  cargo clippy -- -D warnings: success
```

작업지시자 시각 판정:

```text
2026-05-27: 통과
```

## 1.3 현재 판단

이전 보고서의 "회귀 1건 확정" 및 "수정 요청" 판단은 철회한다.
PR #1088의 핵심 변경은 한컴 PDF 기준과 정합하며, maintainer 보강
패치 후 `hwp-multi-001.hwp`의 `pi=14` vpos 회귀와 3페이지 `pi=46`
표 페이지네이션 회귀도 해소되었다. 이후 CI에서 드러난 `2025 donations`
HWPX page_count 회귀도 line0 TAC fit 기준을 line_height로 좁혀 해소했다.
WASM 빌드와 작업지시자 시각 판정도 통과했다.

남은 게이트:

```text
1. PR #1088 BEHIND 상태 해소 또는 maintainer merge 전략 결정
2. 통과 시 PR merge 및 #1087 close
```

---

## 이전 보고 기록 (2026-05-25, 정정됨)

아래 기록은 당시 판단 이력 보존용이다. `hwp-multi-001.hwp` 회귀
판정은 위 재검토 결과에 의해 정정되었다.

## 2. 검증 결과

### 자동 검증 (통과)
| 항목 | 결과 |
|------|------|
| cherry-pick `b7f6f6e2` (검증 전용) | ✅ 충돌 없음 |
| 전체 `cargo test` | ✅ 1596 passed, 0 failed |
| `svg_snapshot` (골든 불변) | ✅ 8 passed, 0 failed |
| `cargo clippy -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all -- --check` | ✅ 위반 0건 |
| WASM 빌드 | ✅ 성공 |
| CI | ⚠️ 미실행 (fork 첫 푸시 패턴, 본 PR 결함 아님) |

### 시각 검증 — **회귀 1건 확정**

`samples/hwp-multi-001.hwp` 1페이지 — 작업지시자 보고. PR 본문
주장 \"276 샘플 변동 0\" 과 모순. 자동 측정(페이지수·LAYOUT_OVERFLOW)
으로는 검출되지 않는 시각 회귀.

## 3. 결함 진단 (확정)

### 회귀 케이스 IR (문단 0.3)

| 컨트롤 | wrap | treat_as_char | vert_rel_to | v_off | PR 정렬 키 |
|--------|------|---------------|-------------|-------|------------|
| [0] 표 (\"제목명\") | 위아래 | **false** | Para | **2346** | **2346** |
| [1] 표 (\"보도시점\") | 어울림 | **true** (TAC) | Para | 0 | **0** (TAC 라 제외) |

### PR 정렬 후
- IR 배열: `[표0(2346), 표1(0)]`
- 정렬 후: **`[표1(0), 표0(2346)]`** — 순서 역전
- 한컴 정답: IR 배열 순서 (\"제목명\" 위, \"보도시점\" 아래)

### 정량 입증 (SVG y 좌표 PR 전/후)

| 글자 | baseline | PR 적용 후 | 차이 |
|------|----------|-----------|------|
| \"보\" 작은 글자 | 324.95 | 172.31 | **−152.6px** |
| \"제\" 큰글자 | 609.87 | 583.88 | −25.99 |
| \"제\" 다른 인스턴스 | 727.47 / 886.31 | 792.46 / 951.29 | **+65px** |

### Root cause — PR 정렬 키의 사각지대

PR 정렬 키는 **TAC 표를 무조건 키=0으로 처리**:
```rust
match ctrl {
    Control::Table(t)
        if !t.common.treat_as_char  // ← TAC 제외
            && matches!(t.common.text_wrap, TextWrap::TopAndBottom)
            && matches!(t.common.vert_rel_to, VertRelTo::Para) =>
        t.common.vertical_offset as i32,
    _ => 0,
}
```

PR 검증 케이스(공직기강·국립국어원·#103) 는 모두 **라벨(in-flow
text) + 표(out-of-flow)** 조합. 본 케이스는 **두 표 모두 out-of-flow
이고 한쪽이 TAC, 한쪽이 비-TAC 위아래** 인 혼합 — PR 정렬이
다루지 않은 사각지대.

## 4. 처리

- **PR #1088 OPEN 유지** — 작성자에게 수정 요청 댓글 게시
  (https://github.com/edwardkim/rhwp/pull/1088#issuecomment-4537590143)
- 회귀 증거 + 옵션 A/B 수정 권고 + 회귀 가드 추가 요청
- verify 브랜치 `pr1088-verify` 삭제
- 이슈 #1087 OPEN 유지 (PR 재푸시 시 재검토)
- local/devel `c756d42d` 으로 origin/devel 동기화

## 5. 평가

### PR 의 강점 (유효)
- root cause 진단 정밀 (배치 순서 vs 위치 분리)
- 공직기강·국립국어원·#103 케이스 개선 명확
- 안정 정렬 + ctrl_idx 보존 설계 영리

### 한계 — 사각지대
- TAC + 비-TAC 위아래 표 혼합 케이스 미고려
- PR 본문 \"276 샘플 변동 0\" 은 자동 측정 한정 — 시각 회귀
  검출 못 함
- 검증 가드도 페이지수·LAYOUT_OVERFLOW 만, 시각 위치 차이 미검증

## 6. 후속 권고 (작성자에게 전달됨)

옵션 A: TAC 표에도 동일 정렬 키 (`treat_as_char` 조건 제거)
옵션 B: 혼합 케이스(TAC + 비-TAC 위아래) 시 정렬 건너뛰고 IR
        배열 순서 유지

어느 쪽이든 `hwp-multi-001.hwp` 1페이지를 회귀 가드 테스트로 추가
부탁. 한컴 정답(IR 배열 순서) 확인 후 옵션 결정 권고.

본 PR 의 진단(같은 문단 내 컨트롤은 vertical_offset 순) 자체는
옳음 — 다만 혼합 케이스 정합 후 재푸시 필요.
