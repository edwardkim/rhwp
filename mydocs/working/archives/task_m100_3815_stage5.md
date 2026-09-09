# Task #3815 Stage 5 — 최신 3단 stack 제한 검증

- 이슈: [#3815](https://github.com/edwardkim/rhwp/issues/3815)
- 브랜치: stack/issue-3815-pagination-coalescing
- 최신 기준: upstream/devel cf5d462dc
- code candidate: 416eb37a1
- 작성일: 2026-08-04

## Stack

1. #3937 browser glyph 폭 보정
2. #3822 overlong token 반복 줄바꿈
3. #3815 pagination coalescing과 연속 입력 E2E

세 레이어를 최신 devel 위에 선형으로 재구성했고 integration merge commit이나 중복
cherry-pick은 포함하지 않았다.

## 최신 기준 검증

- renderer spacing focused: 42 / 42
- SVG renderer: 41 / 41
- composer: 53 / 53
- runner + InputHandler focused: 23 / 23
- Studio 전체 unit: 763 / 763 (이전 stack 전체 검증, 최신 devel은 Studio 파일 변경 없음)
- npx tsc --noEmit: 통과
- production wasm-pack release build: 통과
- wasm32-unknown-unknown library check: 통과
- git diff --check: 통과

## HWP/HWPX combined smoke

production WASM과 새 headless Chrome에서 continuous-only 시나리오를 각 형식 한 번 실행했다.

| 형식 | 숫자 줄 전환 | 최종 숫자 | pending operation p95 | 결과 |
| --- | --- | ---: | ---: | --- |
| HWP | 11 / 69 | 73 | 49.6ms | GREEN |
| HWPX | 11 / 69 | 73 | 49.7ms | GREEN |

두 형식 모두 IME 조합 뒤 숫자가 두 번 줄바꿈되고 overflow 없이 최종 revision까지 게시됐다.
최종 쪽수는 116, latest begin/final step revision은 132 / 132이며 synchronous flush는 0이다.
이전 동일 smoke의 50.2ms / 49.7ms 대비 변화는 각각 -1.2% / 0.0%로 ±10% gate 안이다.
따라서 2026-08-03에 완료한 current, 80ms, 250ms 형식별 3회 전체 측정은 반복하지 않았다.

첫 최종 검증 뒤 devel이 aeb5805cb로 전진했다. 추가 변경은 Stack 제품 파일과 직접 겹치지 않았지만
typeset 쪽 경계 수정이 실제 HWP/HWPX 쪽수에 영향을 줄 수 있어 production WASM과 combined
smoke까지 다시 실행했다. spacing 42 / 42, SVG 41 / 41, composer 53 / 53, Studio focused
23 / 23, TypeScript와 production WASM을 모두 통과했고 두 형식의 115 → 116 결과도 유지됐다.

검토 CI 중 devel이 중첩 표 배치 수정 #3949를 포함한 cf5d462dc로 다시 전진했다. 공용 오늘할일
충돌은 양쪽 기록을 보존해 해소했고 제품 코드는 충돌하지 않았다. spacing 42 / 42,
`issue_2189_cell_text_clip` 1 / 1, composer 53 / 53, fmt·diff와 production WASM을 제한
재검증했다. HWP/HWPX는 11 / 69, 숫자 73, 116쪽, p95 49.6 / 49.7ms로 GREEN이며 대표
HWP crop의 SHA-256도 기존 보존 asset과 일치했다.

## 기존 성능 근거

이전 production 측정에서 current 완료 중앙값은 HWP 1,581.6ms, HWPX 1,586.0ms였고,
input dispatch와 begin overlap, superseded publication과 flow sync flush는 모두 0이었다.
step p95 최대는 8.4ms였다.

남은 pending operation 비용은 exact cursor query가 지배하며 이번 scheduler PR의 비범위다.
flow가 안정되면 stable-tail fast path로 돌아간다.

200ms보다 짧은 간격의 입력이 계속되면 active restart timer가 재예약되어 pagination step과
쪽수 게시가 잠시 멈춘다. 200ms 이상 쉬면 최신 revision에서 재개되고, 완료 뒤 stable-tail
fast path로 복귀한다. 이 pending 구간에서는 기존 120ms idle flush도 선점하지 않는다.

HWP/HWPX 2 / 2, pending operation p95, 115 → 116 쪽수, revision 일치와 synchronous flush 0은
모두 로컬 production WASM + Chrome `--continuous-only` 결과다. Render Diff CI는 이 E2E의
`node --check`만 실행하며 브라우저 성능·쪽수 결과를 대신하지 않는다. 리뷰 재현을 위해
`npm run e2e:issue-3815` 진입점과 MANIFEST 설명을 추가했다.
최신 devel에 이미 추적되던 #3682 진단 프로브의 누락 행도 함께 등록해 새 MANIFEST 검사를
stack top에서 통과시켰다. 프로브 동작이나 #3682 제품 범위는 변경하지 않는다.

## 게시 상태

GitHub Stack #3947을 만들고 세 branch를 upstream에 push했다. Draft PR은 아래와 같다.

- [#3944](https://github.com/edwardkim/rhwp/pull/3944): #3937 browser glyph 폭
- [#3945](https://github.com/edwardkim/rhwp/pull/3945): #3822 overlong token wrap
- [#3946](https://github.com/edwardkim/rhwp/pull/3946): #3815 pagination coalescing
