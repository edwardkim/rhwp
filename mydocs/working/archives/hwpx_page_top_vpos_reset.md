# 개체 host 뒤의 저장 vpos 리셋이 쪽나눔 신호로 남지 않는다 (#6342)

## 무엇이 문제였나

HWPX 저장 사다리(`hp:lineseg@vertpos`)는 **쪽마다 0 에서 다시 시작한다**. 그래서
"직전 문단이 쪽 아래쪽에서 끝났는데 이 문단의 저장 vpos 가 작다" 는 것이 한글이
거기서 쪽을 끊었다는 신호다. `recalculate_section_vpos` 는 렌더용으로 이 좌표를
구역 누적으로 바꾸는데, 그때 이 리셋 신호를 잃지 않도록 두 예외를 이미 갖고 있었다.

- `#1920` — 쪽 하단 고정 틀(vert=쪽·valign=Bottom) host 문단의 저장 0
- `#2158` — 직전 저장 vpos > 60000HU 이고 지금 저장 vpos 가 0 초과 5000 미만

`samples/hwpx/opengov/36385445_결재문서본문_화재발생종합보고서(제2026-189호, 2026. 5. 14.).hwpx`
는 둘 중 어디에도 걸리지 않는다.

```
최상위 문단 3개 (Contents/section0.xml)
  pi=0  결재 표(TopAndBottom, 4x1)   lineseg vertpos=0  vertsize=67460  spacing=540
  pi=1  "붙임  1. 화재현장조사서…"   lineseg vertpos=0        ← 쪽 상단 = 새 쪽
  pi=2  "      2. 국가화재정보…"     lineseg vertpos=2160
```

pi=1 의 저장 0 은 **2쪽 상단 좌표**다. 정답지도 그렇게 나뉜다.

```bash
git show "HEAD:pdf/36385445_결재문서본문_화재발생종합보고서(제2026-189호, 2026. 5. 14.)-2024.pdf" > oracle.pdf
# p0 = 결재 표 / p1 = 붙임 두 줄
```

그런데 `#2158` 규칙이 보는 `prev_stored_last_vpos` 는 직전 문단의 저장 **vpos** 뿐이다.
pi=0 은 TopAndBottom 표 host 라 개체 높이가 `line_height`(67460HU) 에 들어가고 저장
vpos 자체는 0 이다. 게다가 이 문단은 reflow 대상이라 저장 좌표 스냅샷(`orig_span`)이
아예 `None` 이어서 `prev_stored_last_vpos` 가 0 에 머문다(실측 확인).

```
DBG pi=1 first=0 reflow=false prev_last=0 span_prev=None span_self=Some((0, 2160))
```

그래서 임계(60000HU)에 걸리지 않고, pi=1 의 저장 0 이 연속 좌표 68000 으로 덮인다.
그 뒤로는 `hwp_used≈used` 가 되어 rhwp 는 자기 배치가 한글과 같다고 판단하고,
붙임 목록을 앞 쪽에 흡수한 채 1쪽으로 끝냈다 — 본문 952.5px 에 964.3px 를 담아
11.8px 넘긴 상태로.

## 어떻게 고쳤나

직전 문단의 저장 스냅샷이 **없을 때만** 대체 근거로 재계산 사다리 자신의 위치를 쓴다.

```rust
} else if first == 0
    && running_vpos > 60000
    && pi.checked_sub(1)
        .is_some_and(|prev| orig_span.get(prev).copied().flatten().is_none())
{
    running_vpos = 0;
}
```

사다리는 쪽 리셋을 만나면 위 두 규칙이 0 으로 되돌리므로, 60000HU(#1921 near-top
임계와 같은 값)를 넘었다는 것은 "지금 쪽을 이미 다 채웠다" 는 뜻이다. 거기서 만난
저장 0 은 이어붙일 좌표가 아니라 다음 쪽 상단 좌표다.

스냅샷 조건이 핵심이다. 위 규칙이 판단할 근거를 **가졌던** 자리는 그대로 위 규칙에
맡긴다 — 거기서 `first == 0` 을 제외한 것은 의도된 결정이고, 이 변경은 그 결정을
뒤집지 않는다. 새 분기는 근거 자체가 없어 아무도 보지 못했던 자리만 맡는다.

### 좁히지 않으면 깨진다 (실측)

스냅샷 조건 없이 `first == 0 && running_vpos > 60000` 만으로 돌렸을 때 전체
`cargo test --tests` 에서 **6건이 실패**했다. devel 기준선에서는 7건 모두 통과하므로
전부 이 변경이 원인이다.

```
issue_1811_hwpx_pi52_rowbreak_cut_matches_hwp_reference   (issue_1749 파일 — 주석이 경고한 바로 그 노이즈)
issue_6031_page_tail_lines_stay_inside_body_bottom
issue_2470_masked_rewrap_36341511_pins_current_nine_pages
issue_4179 text_host_para_cursor_rect_builds_few_page_trees
hwpx_password_fixture 2건
```

스냅샷 조건을 넣으니 7건 전부 다시 통과한다.

### 하지 않은 것

`renderer/typeset.rs` 의 `paragraph_saved_vpos_reset_starts_new_page_after` 가
"직전 항목이 쪽을 채웠는가" 를 줄의 **시작** vpos 로만 보는 것도 같은 함정이다
(표 host 는 시작이 0). 끝 좌표(`vpos+lh+ls`)로 넓혀 봤지만 이 문서에서 **관측
가능한 차이가 없었다** — 근거 없는 확장이라 넣지 않았다.

## 검증 실측

### 대상 문서 전/후

| | 쪽수 | 정답지 | overflow |
| --- | ---: | ---: | ---: |
| 수정 전 | 1 | 2 | 1건 (2.9px) |
| 수정 후 | **2** | 2 | **0건** |

쪽 하단으로 11.8px 넘기던 것이 사라졌다 — 표(899.5px) + 붙임 1(28.8px) 이 본문
952.5px 안에 들어간다.

### 코퍼스 전수 (저장소 정답지 PDF)

`tests/fixtures/oracle_page_count_baseline.tsv`(#6337/PR #6338)의 555 문서를
수정 전 기준선과 대조했다.

```
대조 555문서  변화없음 554  개선 1  회귀 0  변했지만 여전히 불일치 0  측정실패 0
  [개선] samples/hwpx/opengov/36385445_…: 1 -> 2 (정답지 [2])
```

(넓은 조건에서는 `samples/issue2470/36341511_masked.hwpx` 도 9 → 8 로 정답지와
맞았지만, 그 조건은 위처럼 6건을 깨뜨렸다. 그 문서는 `issue_2470_…_pins_current_nine_pages`
가 9 쪽을 핀으로 박고 있는데 저장소 정답지 PDF 는 8 쪽이다 — 별도 조사거리로 남긴다.)

### 게이트

```
cargo fmt --all -- --check                                                   통과
cargo clippy --profile release-test --all-targets -- -D warnings             통과
node scripts/rust-test-suite-manifest.mjs --check --base-ref upstream/devel  통과
node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel      통과
cargo test --profile release-test --tests --no-fail-fast                     아래 참조
```

전체 통합 테스트에서 남은 실패는 2 타깃 4 건이고, **devel 에서도 똑같이 실패한다**
(같은 checkout, 같은 명령으로 대조):

| 타깃 | 실패 | devel |
| --- | --- | --- |
| `issue_4100_chart_data_edit` | `b2_judgment_assets_match_the_manifest` 외 2 | 동일 실패 |
| `regression_suite_021` | `issue_4179 … text_host_para_cursor_rect_builds_few_page_trees` | 동일 실패 |

`issue_4179` 는 이름을 지정해 단독 실행하면 통과하고 타깃 전체를 돌리면 실패한다 —
devel 에서도 같으므로 이 변경과 무관한 순서 의존이다.

## 남는 것

36385445 의 **쪽수**는 정답지와 같아졌지만 **분할 지점**은 아직 다르다.

| | 1쪽 | 2쪽 |
| --- | --- | --- |
| 정답지 | 결재 표 (500자) | 붙임 두 줄 (76자) |
| rhwp (수정 후) | 결재 표 + 붙임 1 (452자) | 붙임 2 (23자) |

붙임 1 이 아직 1쪽에 남는다. 저장 사다리는 pi=1 을 쪽 상단(0)에 두라고 말하지만,
HWPX 경로에는 그 리셋을 **강제 쪽나눔**으로 승격하는 규칙이 없다
(`variant_vpos_reset_break` 는 `profile.hwp3_layout()` 전용). 모든 저장 리셋을
강제 쪽나눔으로 올리면 rhwp 의 재배치 모델 자체가 저장 사다리 재생으로 바뀌므로
이 변경의 범위를 넘는다. 기하는 이제 정상이다 — 1쪽이 본문 안에 들어가고
overflow 신호가 사라졌다.
