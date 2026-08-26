# Task M100 / 이슈 #5924 처리 결과

**제목**: 쪽-하단 고정 앵커 footer — 복원이 상향하지 않았는데도 불확실 마진 50px 이 걸려
여유 45.6px 쪽이 분할되던 것을 고친다

- 이슈: https://github.com/edwardkim/rhwp/issues/5924
- 대상 문서: `samples/task2098/page_bottom_fixed_anchor_margin_split.hwpx`
- 정본: `pdf/task2098/page_bottom_fixed_anchor_margin_split-2020.pdf` (한글 2020, **1쪽**)
- 브랜치: `fix/task2098-bottom-anchor` (base `origin/devel` = `b9eb55107`)

---

## 1. 결함

쪽-하단 고정 앵커 개체(`vertRelTo=PAGE` · `vertAlign=BOTTOM` · `textWrap=TOP_AND_BOTTOM`,
발신명의 틀 계열)가 1쪽 하단 배타 영역에 **45.6px 여유로 실제로 들어가는데도** 2쪽으로
단독 분리되고, 그만큼 쪽이 하나 더 생깁니다.

| | 쪽수 | 1쪽 내용 | 2쪽 내용 |
|---|---:|---|---|
| 한글 2020 정본 | 1 | BODY TEXT · LAST FLOW LINE · **FOOTER FRAME 틀** | — |
| rhwp (수정 전) | 2 | BODY TEXT · LAST FLOW LINE | FOOTER FRAME 틀만 |
| rhwp (수정 후) | 1 | BODY TEXT · LAST FLOW LINE · **FOOTER FRAME 틀** | — |

`tests/fixtures/render_page_samples.tsv` 도 이 갭을 이미
`hangul_pages=1 / rhwp_pages_baseline=2 / delta=+1` 로 기록해 두고 있었습니다.

## 2. 원인

판정 지점은 `src/renderer/typeset.rs` 의 page-bottom footer fit 입니다.

```
$ RHWP_DIAG_SCAN=1 rhwp dump-pages samples/task2098/page_bottom_fixed_anchor_margin_split.hwpx
DIAG_SCAN FOOTER pi=2 anchor_vpos=0 cur_h=754.67 target_y=754.67 sync_h=754.67
                 block_h=133.33 avail=933.57 avail_after=800.24 slack_code=45.57 margin=50.0
```

- 본문 끝 `754.67px`, 배타 잔여 `800.24px` → **45.57px 여유로 들어감**
- 그런데 `uncertain_anchor_margin = 50.0` 이 얹혀 `754.67 + 50 > 800.24` → 분할

이 마진(#2098 r12 62 → #2138 → #2279 50 으로 재보정)이 보정하려는 불확실성의 원천은
**"앵커 저장 vpos ≤ 0 이라 본문 끝을 `prev_body_bottom_vpos` 에서 복원했다"** 는 점입니다.
코드 주석의 코호트 근거도 복원이 흐름을 끌어올린 사례입니다 — 36387725 는 `cur_h 578` 을
복원값 `640.7` 로 상향했고, **그 상향분**이 과관용의 근원이었습니다.

그런데 마진은 `anchor_vpos <= 0` 이기만 하면 무조건 걸립니다. 이 문서처럼 복원값과 흐름
`cur_h` 가 `754.67px` 로 **완전히 일치**해 복원이 판정을 전혀 바꾸지 않은 경우
(`sync_h == cur_h`) — 즉 보정할 불확실성이 남아 있지 않은 경우 — 에도 흐름 좌표 기준 fit 을
일률적으로 50px 깎아, 여유가 실재하는 쪽을 분할합니다.

`#2279` 가 동기화를 건너뛰는 경로(`page_has_page_abs_top_table`)도 같은 성질입니다. 그
경로는 저장 누적이 허상이라 **흐름 좌표를 믿기로** 결정한 자리인데, 저장 복원에서 온
마진만 그대로 남아 흐름 좌표 fit 을 깎습니다.

### 코퍼스 실측 (samples 전수 259건)

footer fit 판정에 도달하는 문서-지점 11건:

| 문서 | anchor_vpos | cur_h | target_y | 복원 상향 | 슬랙 | 마진 | 판정 |
|---|---:|---:|---:|---|---:|---:|---|
| exam_math.hwp (x3) | 0 | 0.00 | 0.00 | 없음 | 1068~1124 | 50 | 흡수 |
| exam_math_no.hwp (x3) | 0 | 0.00 | 0.00 | 없음 | 1068~1124 | 50 | 흡수 |
| hwpx/issue1948_cross_para_fieldend.hwpx | 42733 | 578.47 | 569.77 | 없음 | 60.40 | 0 | 흡수 |
| task1772/table_outer_margin_common_sync.hwpx | 28638 | 395.81 | 381.84 | 없음 | 327.44 | 0 | 흡수 |
| task1789/exclusion_probe_line_spacing.hwpx | 24000 | 320.00 | 320.00 | 없음 | 283.69 | 0 | 흡수 |
| task2098/page_bottom_fixed_anchor_vpos0.hwpx | 0 | 101.33 | 101.33 | 없음 | 698.91 | 50 | 흡수 |
| **task2098/…margin_split.hwpx** | **0** | **754.67** | **754.67** | **없음** | **45.57** | **50** | **분할** |

마진이 **판정을 뒤집는** 문서는 코퍼스 전체에서 이 1건뿐입니다.

## 3. 수정

`src/renderer/typeset.rs` — 마진의 스칼라(50px)와 코호트 재판정 신호는 그대로 두고,
**적용 조건만** 좁혔습니다.

```rust
let restoration_raised_fit = sync_h > st.current_height;
let uncertain_anchor_margin = if anchor_vpos <= 0 && restoration_raised_fit {
    50.0
} else {
    0.0
};
```

복원이 실제로 판정을 끌어올린 경우에만 마진을 겁니다. 코호트 분할 정답군
(36387725: `cur_h 578` → 복원 `640.7` 로 상향)은 복원이 상향하므로 마진이 그대로 유지됩니다.

### 함께 갱신한 파일

- `tests/fixtures/render_page_samples.tsv` — 자기 문서 행 1줄:
  `hangul=1 / rhwp=2 / delta=1` → `hangul=1 / rhwp=1 / delta=0`
- `tests/suites/issue_regression_pilot/issue_2098_margin_boundary_split.rs` — 이 문서에 대해
  2쪽을 잠그던 단정을 **정본대로 1쪽**으로 바로잡음.

  종전 이 테스트는 이 합성 문서가 코호트(결재문서 60건, 슬랙 3.4~61.3px)를 **대리한다**는
  전제로 2쪽을 잠갔습니다. 그러나 같은 저장소가 들고 있는 이 문서 **자신의** 한글 2020 정본
  (`pdf/task2098/…-2020.pdf`)과 `render_page_samples.tsv`(`hangul_pages=1`)가 모두 1쪽을
  가리킵니다. 두 정본 기록과 어긋나는 단정이라 정본 쪽으로 맞추고, 코호트 대역을 대리하지
  않는다는 점을 주석에 명시했습니다. 코호트 분할군 자체는 복원 상향 조건으로 계속 보호됩니다.

## 4. 검증

| 게이트 | 결과 |
|---|---|
| `render_page_gate.py` (samples 259건) 전 | 일치 245 (94.6%), delta +1: 9건 |
| `render_page_gate.py` (samples 259건) 후 | **일치 246 (95.0%), delta +1: 8건** |
| 게이트 행 변화 | **1건** (대상 문서 delta +1 → 0). 회귀 **0** |
| 코퍼스 SVG self-diff (259 문서 × 앞 2쪽) | 변화 **대상 문서 2쪽뿐**, 의도 외 변화 **0** |
| 쪽 밖 글자 계수 (`<text y>` > 쪽 높이) | 전 0 → 후 **0** (증가 없음) |
| 글리프 총량 | 전 31 (20+11, 2쪽) → 후 31 (1쪽) — **소실 0** |
| `cargo test --profile release-test --test overflow_cell_baseline` | **ok** (1 passed, 37.21s) |
| `cargo test --profile release-test --lib -p rhwp` | **ok** (3893 passed / 0 failed / 13 ignored) |
| `cargo test --profile release-test --test regression_suite_023 issue_2098` | **ok** (1 passed) |
| 위 테스트 red→green | 수정 되돌린 상태에서 **FAILED** (`left: 2, right: 1`) → 수정 적용 시 **ok** |
| `rustfmt --edition 2021 --check` (변경 파일) | 통과 (Windows CRLF 알림만, LF 사본으로 재확인 시 무출력) |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| 열린 PR 5건(#5900·#5909·#5911·#5913·#5914) `git merge-tree` | 전건 **무충돌** |

## 5. 전/후 스크린샷

`mydocs/report/edit_demo_5924/task2098_margin_split_before_after.png`
— 수정 전 1쪽 / 수정 후 1쪽 / 한글 2020 정본 1쪽 3단 비교.

`mydocs/report/edit_demo_5924/task2098_before_ghost_page2.png`
— 수정 전에만 존재하던 유령 2쪽(틀만 홀로 놓임).

### 남는 차이 (이 PR 범위 밖)

이 합성 문서는 `linesegarray` 에 실제 내용과 맞지 않는 줄 높이가 박혀 있습니다
(`BODY TEXT` 의 `vertsize=55000HU` = 733px). 한글은 문서를 열 때 `linesegarray` 를 캐시로
보고 **재조판**하므로 정본에서는 본문 두 줄이 쪽 상단에 자연 줄높이로 붙지만, rhwp 는 저장
줄높이를 존중하므로 본문이 쪽 하단 쪽으로 내려가 있습니다. 이 저장-캐시 신뢰 정책 자체는
실문서 정합의 근간이라 이 PR 에서 건드리지 않았고, 스크린샷의 본문 세로 위치 차이는 그
때문입니다. **이 PR 이 닫는 것은 쪽수 갭(+1 → 0)과 틀의 소속 쪽입니다.**
