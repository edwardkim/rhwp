# PR #1193 검토 — Fix mac window resizing

- **작성일**: 2026-06-01
- **PR**: #1193 (OPEN)
- **컨트리뷰터**: @chkwon (Changhyun Kwon)
- **연결 이슈**: #1191 (`closes #1191`) — "창 너비를 넓혔다 줄이면 가운데 정렬이 유지되지 않는 문제"
- **base/head**: `devel` ← `fix_mac_window_resizing` (`cd410a7c`)
- **mergeable**: MERGEABLE / **BLOCKED** (required "Build & Test" 체크 부재 — CSS only 라 CI 미트리거).
  merge-base = 현재 devel `a593a7e0` → BEHIND 아님.
- **규모**: **1 파일, +5 / −2** (`rhwp-studio/src/styles/editor.css` 만)
- **CI**: check-runs 0 (CSS only). render-diff 는 `rhwp-studio/**` paths 대상이나 미실행 상태.

## 1. 문제와 원인

rhwp-studio `#editor-area` 가 `grid-template-columns/rows: 20px 1fr` 로 정의됨.
CSS 에서 `1fr` 은 암묵적으로 `minmax(auto, 1fr)` → 트랙 최소 크기가 **콘텐츠 기반**.
콘텐츠(페이지)가 커진 뒤에는 트랙이 그 최소 아래로 줄지 않는 **한 방향 래칫**이 발생 →
창을 넓혔다 줄이면 `#scroll-container` 폭이 따라 줄지 않아 페이지가 우측으로 치우치고
가운데 정렬이 깨짐 (#1191, macOS 에서 관찰).

## 2. 수정 내용 검토

```diff
-  grid-template-columns: 20px 1fr;
-  grid-template-rows: 20px 1fr;
+  grid-template-columns: 20px minmax(0, 1fr);
+  grid-template-rows: 20px minmax(0, 1fr);
```

- `1fr` → `minmax(0, 1fr)` 로 트랙 최소를 **0** 으로 고정 → 창 축소 시에도 트랙이 정상 축소,
  가운데 정렬 유지.
- **이는 CSS grid 의 표준/정석 해결책** — overflow 컨테이너에서 `minmax(0,1fr)` 로 콘텐츠
  최소 래칫을 푸는 잘 알려진 관용구.
- 변경 의도를 설명하는 주석 동반 (양호).

## 3. 위험 평가

- **매우 낮음.** CSS 한 속성(2 트랙)만 변경. 로직/렌더러/직렬화 무관.
- 회귀 가능성: `minmax(0,1fr)` 는 `1fr` 의 상위호환에 가까움 — 콘텐츠가 트랙보다 작을 때
  동작 동일, 클 때만 축소 허용. overflow:hidden 컨테이너라 시각 부작용 거의 없음.
- 다른 grid 사용처 영향 없음 (`#editor-area` 국소).

## 4. 검증 결과 (로컬 머지 시뮬레이션 `pr1193-verify`)

| 단계 | 명령 | 결과 |
|------|------|------|
| merge | `git merge --no-ff` | ✅ CLEAN (충돌 0) |
| studio build | `npm --prefix rhwp-studio run build` | ✅ (tsc + vite) |
| (Rust 무관 — 변경 없음) | — | — |

> CSS only 라 cargo 테스트는 무관. 시각 본질이라 WASM 빌드 후 작업지시자 직접 판정이 게이트.

## 5. 판단 (예정)

표준 CSS 해결책 + 위험 매우 낮음 → **머지 권고**. 단, macOS 창 리사이즈 시각 회귀 수정이므로
**작업지시자 WASM 빌드 후 직접 시각 판정**을 게이트로 둠 (창 넓혔다 줄였을 때 가운데 정렬 유지).
승인 + 시각 판정 통과 시 메인테이너 로컬 `--no-ff` 머지 + push. `closes #1191` → 자동 클로즈.
결과는 `pr_1193_report.md` 에 기록.
