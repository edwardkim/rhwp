# 최종 결과 보고서 — Task M100-1304: 수식 무브레이스 첨자 공백 구분

- 이슈: [#1304](https://github.com/edwardkim/rhwp/issues/1304)
- 브랜치: `local/task1304` (base: `stream/devel` 9d3aa212)
- 후속 이슈: [#1305](https://github.com/edwardkim/rhwp/issues/1305) (괄호 위첨자 orphan, 분리)

## 1. 신고 및 증상

`3-10월_교육_통합_2022.hwp` 11쪽 문18) 해설 시그마(∑)에서 하한 `k=1` 의 `=1` 이 본문으로 밀리고 상한 `6` 이 `1` 의 위첨자로 붙어 깨짐.
("시그마에서 인자가 안보임 k만 보임. 시작/끝(범위가) 표시 안됨 - 위치가 달라요")

## 2. 근본 원인

미주 내부 수식 스크립트가 본문과 다른 표기를 쓴다.

| 위치 | 스크립트 | 결과 |
|------|----------|------|
| 본문 | `sum _{k=1} ^{6}` (브레이스) | 정상 |
| 미주 | `sum_k=1 ^6` (브레이스 없음, 공백 구분) | 깨짐 |

HWP 수식에서 일반 공백은 무브레이스 operand 의 **구문적 경계 구분자**다. 그러나 (1) 토크나이저가 일반 공백을 버리고(#505), (2) 파서가 무브레이스 첨자 operand 를 단일 토큰만 취해(LaTeX식), `sub=k` 만 잡고 `=1`/`^6` 이 분리됐다.

권위 PDF(`pdf/3-10월_교육_통합_2022.pdf`) 검증으로 `sum_k=1 ^6` 의 의도가 ∑(하한 `k=1`, 상한 `6`)임을 확정했다.

## 3. 해결

최소 변경 설계:

1. **토크나이저**: `Token.space_before: bool` 플래그 1개 추가 (경계 토큰 없이 공백 유무만 기록).
2. **파서**: 무브레이스 **하한(아래첨자) 전용** `parse_script_operand` 도입 — `원자 (공백없는 관계연산자 원자)*` 패턴. 관계연산자 앞 공백이 있으면 묶지 않음.
3. **범위 제한**: 위첨자·명령 body 무변경 → `x^2=4` 류 미접촉.

`space_before` 가드(`x^2 = 4` 보호) + 관계연산자 한정(`k^2-4` 보호) + 하한 한정(위첨자 등식 보호)으로 회귀 표면을 최소화했다.

변경 파일: `src/renderer/equation/tokenizer.rs`, `src/renderer/equation/parser.rs` (테스트 6개 포함).

## 4. 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1582 passed, 0 failed |
| `cargo test --tests` | 0 failed |
| `cargo fmt --check` (변경 파일) / clippy | clean |
| 시각 (문18 4줄 ∑) | 상·하한 정상, 권위 PDF 일치 |

## 4.1 후속 보정 — 시그마 상·하한 가로 정렬 (Stage 2)

시각 검토에서 "시그마 인자가 왼쪽으로 밀림" 발견. 원인은 연산자 폭 추정 불일치 — layout 은 `estimate_text_width('∑')=0.8·op_fs`, render 는 `estimate_op_width=0.6·op_fs` 로 ∑ 를 과소추정해 우측 치우침. 하한이 ∑ 보다 넓을 때(`k=1`) 가시화됐다.

수정: `layout::estimate_text_width` 를 `pub(crate)` 로 노출하고 `svg_render`/`canvas_render` 의 big-op 중앙정렬을 이 단일 기준으로 통일(중복 `estimate_op_width` 제거). 픽셀 측정상 ∑ 중심 71→51.7, 하한 `k=1` 47→51.6 으로 정렬됨. 상세: `mydocs/working/task_m100_1304_stage2.md`.

## 5. 잔존 / 후속

- `(k+1)^2` 괄호 뒤 위첨자 orphan(지수 `2` 가 낮게 표시)은 브레이스 표기에서도 동일한 **사전 결함**으로, 본 task 와 분리해 [#1305](https://github.com/edwardkim/rhwp/issues/1305) 로 등록했다. `7^2`(숫자 base)는 정상.

## 6. 결론

사용자 신고의 근본 원인을 해소했고 전 테스트 통과·회귀 없음. 문18 해설 시그마가 권위 PDF와 일치하게 렌더된다.
