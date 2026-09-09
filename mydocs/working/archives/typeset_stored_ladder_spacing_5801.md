---
kind: working
status: active
issue: 5801
---

# 저장 사다리가 문단 위 간격을 안 담은 문서에서 쪽 채움 회계가 짧아진다 (#5801)

작업 브랜치: `fix/5801-stored-ladder-spacing-gate`
대상: `src/renderer/typeset.rs` · `tests/cases/issue_5801_stored_ladder_spacing_before.rs`

## 한 줄

`#2279 ①` spacing 트림의 전제("저장 ladder 가 spacing 을 이미 반영")를 **실제로 확인**하고,
안 담은 사다리에서는 트림하지 않는다.

## 이슈가 요구한 것 — 그리고 처음 분석이 틀렸던 것

`156677324` 1쪽에서 typeset 누적이 `796.4px` 인데 layout 은 `~990px` 까지 그린다. 쪽이 다
찼는데 137px 남았다고 보고 다음 문단을 얹는다(#5755 의 상류).

처음엔 "layout 이 문단 앞 간격을 이중 적용한다"고 봤는데 **틀렸다.** 한글 실물 PDF 로 재니
rhwp 배치가 한글과 같았다.

| | 저장 사다리 | rhwp | 한글 PDF |
|---|---|---|---|
| 문단 **내** 줄 간격 | 29.87px | 29.86px | 29.86px |
| 문단 **간** 간격 | **29.87px** | 49.87px | **50.23px** |
| `pi=3`첫~`pi=8`끝 스팬 | **525.20px** | 598.53px | **600.88px** (차 −0.39%) |

혼자 다른 것은 **저장 사다리**다. `paraPr id="45"` 가 문단 위 간격(`hc:prev` 1500/3000 HU)을
선언하는데 사다리는 문단 간 delta 를 문단 내 줄 advance 와 똑같이(2240 HU) 적었다. 한글은
그 간격을 실제로 그린다.

## 고친 방법

`flow_advance_height` 의 `sb` 트림을 켜는 `trim_spacing_before_for_flow` 에 조건을 하나 더한다.

```rust
&& stored_ladder_encodes_spacing_before(paragraphs, para_idx, fmt.spacing_before, self.dpi)
```

판별은 데이터 안에 있다 — **앞 문단 마지막 줄 아래에서 이 문단 첫 줄까지의 저장 간격**이
`문단 내 줄 간격 + 문단 위 간격` 을 담고 있으면 권위 사다리, 아니면 아니다. 비교 기준인
줄 간격은 저장 `line_spacing` **필드가 아니라 같은 사다리 안의 줄 간 실제 delta** 를 쓴다
(필드를 쓰면 문단 경계에서 줄 간격을 흡수한 사다리를 오탐한다 — 아래 v1 실측).

합성 사다리(vpos 전부 0)·쪽 경계 되감김·컨트롤 문단·`sb <= 0.5px` 는 판별 대상이 아니라
종전대로 둔다.

## 판별식 세 벌을 코퍼스로 재서 고른 것이다

정답지는 `_oracle_pdf_2022` 한글 2022 PDF 중 원본 있고 1-up 세로인 **223건**.

| | 쪽수 정확 일치 | 평균 \|Δ\| | 본문칸밖 노드 | 용지밖 노드 |
|---|---|---|---|---|
| 기준선 `fb434269e` | **201 / 223 (90.1%)** | 0.126 | 714 | 78 |
| 안 B — 트림 전면 중단 | 197 (88.3%) | 0.166 | 627 | 56 |
| v1 — `저장간격 ≥ line_spacing필드 + sb` | 200 (89.7%) | 0.139 | 647 | 60 |
| v2 — `저장간격 ≥ sb` | 201 (90.1%) | 0.126 | 712 | 78 |
| **v3 — `저장간격 ≥ 실제 줄delta + sb`** | **201 (90.1%)** | **0.126** | **705** | **73** |

- 안 B 는 쪽수를 6건 악화·1건 개선 → 기각. 트림은 사다리가 진짜 권위인 문서에서 일하고 있다.
- v1 은 넘침을 크게 줄이지만 `2990099`(290→293)·`3249937`(60→61) 두 건을 깨뜨린다.
- **v3 은 쪽수 변화 0건**, 본문칸밖 −9노드(3문서)·용지밖 −5노드(1문서), **늘어난 문서 0건.**

## 시험 명령

```bash
cargo test --test regression_suite_013 issue_5801   # 신규 계약 2건, ok
cargo fmt --all -- --check                          # exit 0
cargo clippy --all-targets -- -D warnings           # exit 0
node scripts/rust-unit-test-tiers.mjs --check       # 4225 (증가 없음)
```

신규 계약(`tests/cases/issue_5801_stored_ladder_spacing_before.rs`)은 커밋된 샘플
`samples/2025 행정업무운영 편람(최종).hwpx` 272쪽을 쓴다.

```
기준선   used=729.2px  hwp_used≈748.2px  diff=-18.9px   ← 18.9px 짧게 셈
게이트   used=748.2px  hwp_used≈748.2px  diff= +0.0px   ← 저장 좌표와 일치
쪽수     383 → 383 (변화 없음)
```

기준선에서는 첫 테스트가 실패한다(`diff.abs() <= 1.0` 위반) — 빈 테스트가 아니다.

## 남은 것

이 변경은 **회계만** 고친다. `156677324` 는 용지 밖 그리기가 2→0 으로 사라지지만 본문 칸
초과 5노드가 남는다 — 그건 `saved_tail_vpos_fit` 이 되감긴 저장 좌표를 믿는 #5755 의 몫이다.

## PR 메모

`gh pr create --base devel --body-file` · `closes #5801`
