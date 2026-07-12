# PR #2216 검토 — 프론트 패키지 변경 시 build/test CI gate (#2183)

- 작성자: postmelee (collaborator, #2183 assignee) / base: devel / 검토일: 2026-07-12
- 리뷰 요청: edwardkim / 판정: **APPROVE** (비차단 제안 2건 + rebase 요청)
- 실측 공유: PR #issuecomment-4947538533 (GHA 전체 PASS, frontend gate 2분 10초,
  fast-pass 보정 실측 3련)

## 변경 요약

- preflight에 frontend 영향 판정(fail-open: 감지 실패/목록 잘림/미지원 이벤트 →
  실행, rename 양쪽 판정, ci.yml·wasm_api.rs 포함).
- `frontend-package-gates` worker: fresh dev WASM(wasm-pack --dev) + 4패키지
  npm ci + binding/SW/unit/build/dist/compile — #2174 로컬 게이트의 CI 편입.
- aggregate(`Build & Test`)에 frontend 결과 집계 + required=false 시 skipped
  강제 검증. `${{ }}` 인라인 → env 전환(script injection 방어).
- fork PR fast-pass fallback: Check Runs 부재 시 Actions Jobs API로 exact
  workflow·event·branch·head SHA + aggregate job success 검증 (fail-closed).
- 캐시: PR restore-only / trusted devel·main push만 save (poisoning 방어).

## 판정 근거

- fresh WASM 범위 확장(주요 안건): #2174 실증(stale pkg false-green) 근거로 동의.
  비용 2분 10초 합리적. wasm-pack pin(0.13.1/0.15.0)은 별도 toolchain 이슈 분리 수용.
- 비차단 제안: ①frontend 트리거에 Cargo.lock 추가 검토 ②wasm-pack 설치 버전 pin.
- merge 전: mydocs/orders/20260712.md 충돌 1건 rebase 해소 요청. merge는 관례대로
  postmelee 진행 (collaborator 분업).

## 검증

작성자 GHA 실측(전체 PASS + fast-pass 2련 + fallback 사유 재현) + 로컬 fixture
28건(판정 12/aggregate 8/detector 8) + actionlint PASS. ci.yml 전문 diff 검토.
