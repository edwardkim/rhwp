# [R50] 편집 전후 render-diff 회귀 게이트 — r1 실측과 임계 제안 (2026-08-08)

로드맵 트랙 E `R50 render-diff 임계 정책 상시 게이트화`
(`mydocs/tech/agent_roadmap/track_e_capabilities.md`)의 첫 착지 기록이다.
render-diff 기구 자체(#3618, `src/diagnostics/render_geom_diff.rs`)는 완비돼
있었고, 없던 것은 **"편집 전후"를 이 기구에 배선한 상시 게이트**였다 —
`tests/edit_*.rs` 어디에도 render-diff 호출이 없었다.

이 문서는 임계를 **확정하지 않는다**. 대표 편집 3종의 전후 maxDisp 분포를
실측하고, 그 분포에서 나오는 임계 **제안과 근거**를 남긴다. R50 DoD 가 요구하는
"임계값 실측 합의"의 합의 주체는 메인테이너다.

## 1. 방법

- 비교 기구: `rhwp render-diff <전> <후> --json` (pair 모드) — 재구현 없이 기존
  단일 출처를 그대로 배선. 종료 코드 계약: 0=PASS, 3=회귀 검출.
- fixture: 기존 편집 계약 테스트와 같은 실물 2종.
  - `samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` (30쪽, 표 53) — set-cell.
  - `samples/field-01.hwp` (3쪽, 누름틀) — fill-fields / replace-text.
- 편집 결과물은 gitignore 된 `output/` 아래에만 생성. 저장소 무변경.
- 바이너리: `target/release-test/rhwp.exe` (rhwp v0.8.2, devel 5a4f26d0d).
- 재현: `python tools/measure_edit_render_gate.py` (기본 2회 반복으로 결정성 동시 검증).

## 2. 결정성 선확인 — hard 게이트 전제 성립

같은 문서 자기 pair diff 를 2회 반복, 봉투(JSON 전체)를 비교했다.

| 문서 | maxDisp | status | exit | 반복 봉투 동일 |
| --- | ---: | --- | ---: | --- |
| field-01.hwp (3쪽) | 0.0 | PASS | 0 | 예 (완전 동일) |
| 기부·답례품 양식.hwpx (30쪽) | 0.0 | PASS | 0 | 예 (완전 동일) |

편집 전후 비교 5종(아래 표 전부)도 각 2회 반복에서 봉투가 완전히 동일했다.
**비결정 관측 0건** — soft(관측) 모드로 낮출 이유가 없어, 게이트를 hard
(계약 테스트 단언)로 설계했다. 참고: `LAYOUT_OVERFLOW` 진단 로그는 stderr 로만
나가고 stdout JSON 계약을 오염시키지 않는다(이 역시 반복 간 동일했다).

## 3. 편집 전후 maxDisp 분포 실측

| 편집 | fixture | maxDisp(px) | status | exit | 쪽수 | 변화 페이지 |
| --- | --- | ---: | --- | ---: | --- | --- |
| set-cell (표1 (0,0) → "실증테스트값") | 양식.hwpx | 539.6 | OVER | 3 | 30→30 | page 0 만 |
| set-cell, `--max-disp 600` | 〃 | 539.6 | PASS | 0 | 30→30 | 〃 |
| fill-fields (회사명 → "주식회사 검증") | field-01 | 152.0 | STRUCT_MISMATCH | 3 | 3→3 | page 0 만 (TextRun −2) |
| fill-fields, `--max-disp 100000` | 〃 | 152.0 | STRUCT_MISMATCH | 3 | 3→3 | 〃 |
| replace-text 동폭 (회사→기관, 2자→2자) | field-01 | 0.0 | PASS | 0 | 3→3 | 없음 |
| replace-text 장문 (회사→14자, red 주입) | field-01 | 279.0 | OVER | 3 | 3→3 | page 0 만 |

관찰:

1. **국소성** — 모든 편집에서 변화(변위·구조)는 편집이 닿은 페이지 1곳에만
   나타났고, 나머지 페이지는 전부 maxDisp 정확히 0.0·구조 불일치 없음이었다
   (set-cell 은 30쪽 중 29쪽이 0). 쪽수도 전 케이스 불변.
2. **편집 페이지의 변위는 편집 그 자체다** — set-cell 539.6px 는 셀의 다행
   안내문이 실값 한 줄로 접히며 생긴 정당한 변화다. 편집 페이지에 1px 임계를
   그대로 적용하면 정상 편집이 전부 OVER 가 된다(너무 엄함의 실측 사례).
3. **fill-fields 의 구조 신호** — 누름틀 안내 run 이 실값 run 으로 합쳐지며
   TextRun −2. `--max-disp` 를 아무리 키워도 STRUCT_MISMATCH·exit 3 이
   유지된다 — 구조 회귀는 임계로 침묵시킬 수 없는 독립 하드 신호다.
4. **동폭 치환은 기하 0** — 같은 글자 수·폭의 치환(회사→기관)은 maxDisp 정확히
   0.0. 편집·레이아웃 어느 쪽의 잡음도 이 0 을 먼저 깨므로 예민한 카나리다.

## 4. 임계 제안 (확정은 메인테이너 몫)

실측 분포에서 나오는 제안이며, `tests/edit_render_diff_gate.rs` 가 이 값으로
고정돼 있다. 값 변경은 테스트 상수 수정으로 끝난다.

| 축 | 제안 | 근거 |
| --- | --- | --- |
| 비편집 페이지 변위 | **0.0px (동일해야 함)** | 실측 전 케이스에서 정확히 0. 0 초과는 편집이 닿지 않은 곳이 움직인 것 = 레이아웃 회귀. |
| 쪽수 | **불변** | 실측 전 케이스 불변. 쪽수 변화는 render-diff 자체도 최강 신호(PAGE_MISMATCH)로 정의. |
| 편집 페이지 변위 상한 | set-cell 류 **600px**, fill-fields 류 **200px** | 실측 최대 539.6 / 152.0 + 여유 ~10~30%. 편집 자체의 정당한 변화를 통과시키되, 회귀로 증폭되면 잡는 봉투. |
| 동폭 치환 | **0.0px 고정** | 실측 정확히 0. 잡음 조기 경보 카나리. |
| 구조 신호 | fill-fields TextRun −2 를 **정체까지 고정** | 실측값. 다른 타입·다른 개수로 변하면 편집 의미가 달라진 것. |

너무 엄함/너무 느슨함의 경계 실측: 기본 임계 1.0px 에서는 정상 set-cell 도
OVER(정상 편집을 막음), 600px 에서는 PASS 하면서 장문 치환 red(279px)는 1.0px
기본 임계 경로로 계속 잡힌다.

## 5. red 실증 (변이 후 복원)

게이트가 "항상 green" 이 아님을 두 경로로 실증했다.

1. **상시 red 테스트** — `disruptive_edit_is_caught_as_regression`: 장문 치환
   (회사→주식회사법인등기부등본상호명)은 기본 임계에서 OVER·maxDisp 279.0·
   exit 3 으로 잡힌다. 이 테스트가 계속 CI 에서 red 경로를 돌린다.
2. **일회 변이 실증** (2026-08-08, 검증 후 복원) — 카나리 테스트의 `--replace`
   를 장문으로 바꿔 돌리면:
   `assertion 'left == right' failed: ... "status":"OVER","maxDisp":279.0,`
   `"regression":true` (exit 3 ≠ 기대 0) 로 즉시 실패했다. 복원 후 5/5 green.

## 6. 산출물과 재현 절차

- `tests/edit_render_diff_gate.rs` — 계약 테스트 5건. 기존 CI 가 `tests/` 를
  그대로 돌리므로 워크플로 변경 없음(상시 게이트화).
- `tools/measure_edit_render_gate.py` — 실측 러너. 재현:

```
cargo build --profile release-test --bin rhwp   # 바이너리가 없을 때만
python tools/measure_edit_render_gate.py        # 기본 2회 반복, output/ 아래 봉투 저장
cargo test --profile release-test --test edit_render_diff_gate
```

- 검증 로그 (이 PC, 2026-08-08): 계약 테스트 5 passed / 0 failed, 러너 exit 0
  (전 측정 결정적), `cargo clippy -- -D warnings` 통과.

## 7. 한계와 다음 조각

- fixture 2종·편집 3종의 분포다. 임계 일반화에는 코퍼스 확대(편집 대상 문서
  10~50종)가 필요하다 — 러너가 그 확대의 재현 도구다.
- 편집 페이지 상한(600/200)은 이 fixture 의 편집 의미에 묶인 값이다. fixture 나
  편집 내용을 바꾸면 재실측이 선행돼야 한다(러너 1회 실행).
- insert-image·redact·sanitize 등 나머지 편집 동사의 전후 분포는 미측정 —
  같은 러너 패턴으로 후속 조각에서 확장 가능하다.
