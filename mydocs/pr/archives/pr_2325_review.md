# PR #2325 검토 — tac TopAndBottom 소형 개체 줄 하단 여백 스필 (#2137 부분)

- PR: https://github.com/edwardkim/rhwp/pull/2325 (planet6897)
- 이슈: #2137 (Refs — 부분 수정, 잔존 27건 별도 축 코멘트 갱신 명시)
- base=devel 단독, 스택과 독립

## 변경 본질

소형 tac TopAndBottom 그림/도형 전용 문단(정책브리핑 배너류)이 저장
lineseg 상 쪽 하단 여백으로 스필(956.6 > 933.6)하는데 rhwp 는 두 경로
모두에서 배제되어 다음 쪽으로 밀던 결함:

- `para_controls_only_tac_topbottom_objects` 신설 (전 컨트롤이 tac+
  TopAndBottom 그림/도형인 문단)
- 기존 #2093 저장 page-last 신뢰 게이트(`saved_single_line_bottom_fits`)에
  편입 + 해당 케이스만 하단 여백급 스필 허용폭 40px
- **저장 page-last 증거 결합 시에만 발동** — 대형 박스(#1027-E2)는 저장
  vpos 가 다음 쪽을 인코딩해 bounds 에서 자연 배제 (push 정합 보존 논리
  가 구조적)

기존 신뢰 경로(#2093/#2137 비-TAC float)의 tac 판 — 저장 신호 신뢰 계보.

## 로컬 재실증 (merged tree)

| 게이트 | 결과 |
|--------|------|
| 156637323 | devel **2 → 1쪽** (한글 정합, 주장 일치) |
| PR 테스트 | 2/2 |
| 핀 | byeolpyo 4/26, 시장구조조사 312 유지 |
| `cargo test --tests` | 실패 0 / fmt 통과 |

r12 OVER+SHAPE 14→17(+3, 악화 0) 주장은 그들 오라클 산출 — 표본
(156637323)과 스위트·핀 무회귀로 방향 확인.

## 판단

**merge 권고.** 기존 게이트에 케이스 하나를 증거 결합 조건으로 편입한
최소 정정. 스필 40px 은 하단 여백 규모의 물리적 근거 있는 허용폭.
