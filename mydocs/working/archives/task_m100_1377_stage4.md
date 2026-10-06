# Task #1377 Stage 4 — 결정-전파 plumbing 설계 확정 (proxy 음성, 아키텍처 스펙 도출)

- **이슈**: #1377 (M100) / 브랜치 `local/task1377`
- **단계**: Stage 4 — 결정-전파 plumbing 구현 시도 → 설계 확정
- **결과**: render-side proxy 2종 모두 음성으로 **typeset acc 전파가 유일 정답** 확정. 구현 스펙 도출.
  10+ 생성 지점 영향으로 본 세션 미구현(코드 클린).

## 1. proxy 음성 2종 (render 만으로는 불가 입증)

| proxy | 결과 |
|-------|------|
| empty_tac_guide line_spacing 8px cap | p22 해소(1100.59→1087.72) **그러나 14건 회귀** (between-notes gap 일괄 제거) |
| 빈-수식-spacer advance 를 vpos-delta(next−this) 로 clamp | **31건 회귀** (vpos-delta 가 보편 신호 아님; p22 과압축 741px) |

→ render 의 local 신호(line_spacing·vpos-delta)로는 typeset 의 **선택적** compact 결정을 복제 불가.
typeset acc(pi=1128=33.6, 14건 gap 은 보존값)만이 정답.

## 2. 확정 설계 (typeset acc 전파)

1. **`ColumnContent`(pagination.rs:151)** 에 `endnote_para_advance: HashMap<usize, f64>` 추가
   (`wrap_anchors` 선례와 동일 per-para 메타 채널).
2. **typeset** (`pagination/state.rs` flush_column / state) 가 단 닫을 때, 단 내 미주 para 별 누적
   advance(`en_advance` = `st.current_height += en_advance` 의 값)를 이 맵에 기록.
3. **render** (`layout.rs` build_single_column): 미주 para 의 렌더 advance(`new_y − _y_in`)가 기록된
   typeset advance 보다 크면 **`new_y = _y_in + typeset_advance`** 로 clamp(min).
   - 선택성 자동 보장: typeset 이 보존한 gap 은 typeset_advance ≈ render → min 무변. pi=1128 처럼
     typeset 이 compact 한 것만 줄어듦.

## 3. 구현 규모 / 리스크

- **생성 지점 10+** (`ColumnContent {` : pagination.rs:543·layout.rs:2839·pagination/state.rs:86/106·
  page_number.rs:120·layout/tests.rs×4) 전부 새 필드 추가(대부분 `Default::default()`).
- typeset 기록 로직 + render clamp. **전 exam(B·A3) + 골든 SVG 회귀 가드 필수.**
- 비단조 영역이나, **typeset acc 가 정답(14 테스트가 그 정확성 보증)** 이므로 proxy 대비 안전. 단
  split/first-para/TAC 누적 특례에서 advance 기록 정합 주의.

## 4. 성과 / 권고

- **완성**: 발산 진단(pi=1128 빈 수식 spacer phantom line_spacing) + 수정 가치 입증(p22 실해소) +
  proxy 음성으로 정답(typeset acc 전파) 확정 + 구현 스펙 도출.
- **권고**: 위 3-step plumbing 을 신규 집중 작업으로 구현(생성 지점 일괄 + typeset 기록 + render clamp,
  골든 SVG 가드). 진단·설계가 완비돼 구현은 기계적이나 다지점이라 신중 1회 필요.

## 5. 코드 상태

src 클린(전 시도 revert). 유효 수정 = #1375 `821a8b32`. 진단/설계 = #1377 Stage1~4.
