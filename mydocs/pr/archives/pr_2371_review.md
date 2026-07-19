# PR #2371 검토 — sub-100% 퍼센트 줄간격 음수 gap 존중 (#2279, 스택 4 완결)

- PR: https://github.com/edwardkim/rhwp/pull/2371 (planet6897)

## 변경 본질

sub-100% 줄간격(line=60%)에서 한글은 advance = lh×pct(음수 gap, 압축)를
그대로 존중하는데 rhwp 의 fresh/합성 4지점은 `.max(0)` 클램프로 팽창시킴
— 클램프 제거. Fixed/SpaceOnly/Minimum 불변. 근거 이중 실측: 생성기 stored
ls=-680 = 한글 재저장 anchor advance 정확 일치 + sb-상쇄 가설 반증 사다리
16케이스(순수 가산 모델 확인 — 가설을 세우고 반증까지 돌린 완결 실험).

## 로컬 재실증 (merged tree)

전체 스위트 0 실패 · svg_snapshot 8/8 · 전 핀 유지(시장 313·prep 145·
byeolpyo) · 최소 diff(+28) · fmt/clippy 0 · 충돌 0. 정상 모드(stored 신뢰)
무변화 구조라 게이트 중립 주장과 정합.

## 판단

**merge 권고 — 스택 4연작 완결.** 92셋 86→90 누적(캠페인), 잔여 −1×2.
