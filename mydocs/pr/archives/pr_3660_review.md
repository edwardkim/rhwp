---
kind: pr-review
status: active
---

# PR #3660 검토 — 본문 그림 narrow query와 object URL 재사용

| 항목 | 값 |
| --- | --- |
| 작성자 / reviewer | `@lpaiu-cs` / `@jangster77` |
| 원 PR / 관련 이슈 | [#3660](https://github.com/edwardkim/rhwp/pull/3660) / [#3315](https://github.com/edwardkim/rhwp/issues/3315) Track 4 |
| 원 head 참고값 | `5991109d6efd50e46343dd1788e46e91f5ab572d` |
| 통합 후보 | [#3661](https://github.com/edwardkim/rhwp/pull/3661) `52903c91bf132f7f3a977afc9cc265859b024c85` |
| 원 변경 규모 | stacked #3653 포함 18 files, +1,794 / -127; 통합에는 #3660 전용 기능 commit만 한 번 적용 |
| 권고 | #3661의 #3653 선행 API와 함께 수용. #3315는 open 유지 |

## 변경과 통합 판정

본문 flow 그림을 DOM overlay로 분리할 때 Studio는 매 편집마다 전체 PageLayerTree에서 base64를 받았다.
이 PR은 bbox·clip·crop·transform·effect·mime·source key만 담은 `getPageFlowImageOps()`를 먼저 묻고,
바이트는 key별 `Blob` object URL로 한 번만 만든다.

- Rust narrow query는 layer inheritance, master-page 제외, replay plane, nested `clipRect` 교집합을
기존 tree 소비자와 같은 pre-order로 보존한다.
- TypeScript parser는 한 항목이라도 malformed·unresolvable이면 `null`을 반환한다. partial image를
그리지 않고 전체 tree의 기존 data-URL 경로로 fall back한다.
- source key가 없는 합성 그림은 narrow query를 cacheable false로 만들어 fallback한다.
- `FlowImageUrlCache`는 document revision 교체와 `dispose`에서 모든 object URL을 revoke한다. bbox는
본문 흐름에 따라 바뀌므로 page/key 캐시로 재사용하지 않고, 바이트 URL만 재사용한다.
- DOM split은 narrow image count가 layer summary와 같을 때만 활성화한다. raw SVG가 섞이면 기존
static layer 경로를 유지한다.

#3660 source는 #3653 위에 stack돼 있으므로 source branch의 선행 commit을 중복 체리픽하지 않았다.
전용 기능 commit `5991109d6`은 통합 `52903c91b`의 마지막 commit이며, #3653 통합 commit
`18631bce7`을 선행해 전체 patch를 정확히 재구성한다.

## 검증

| 검증 | 결과 |
| --- | --- |
| source #3660 CI | full CI, CodeQL, Canvas visual diff, `Build & Test` success |
| 통합 code head CI | lint·WASM check, frontend package gates, Native Skia, archive, default-feature 8 shards, CodeQL, Canvas visual diff, `Build & Test` 모두 success |
| Rust 회귀 | 실제 flow-image HWP/HWPX 4개에서 tree와 narrow query의 개수·순서·bbox·clip·effect·transform·mime·key 동등성, key-byte 해석, bbox 이동, no-flow 페이지를 고정 |
| Studio 회귀 | data URL/object URL mapping, cacheable false, malformed/unresolvable all-or-nothing fallback, object URL release를 고정 |
| 실제 브라우저 | headless Chrome Canvas2D에서 cacheable key→bytes, blob URL 그림 3장 decode, 문서 교체 뒤 기존 URL 전부 revoke·cache 0 확인 |
| 추가 전체 Cargo | source 및 exact integration CI와 중복되므로 작업지시에 따라 실행하지 않음. 성공 근거로 사용하지 않음 |

실제 Studio 검증은 `3-10월_교육_통합_2022.hwp` p0에서 수행했다. cacheable key의 바이트를 받고 DOM
blob URL 그림 세 장이 모두 natural size로 decode됨을 확인한 뒤, `field-01.hwp`로 문서를 바꿔 그 URL들이
전부 revoke되고 cache가 0이 됨을 확인했다. raw SVG가 섞인 `143E433F503322BD33.hwp`는 DOM split을 하지
않고 기존 static 경로로 fallback했으며, 추가 flow-image HWP/HWPX 표본 세 건은 blob DOM layer를 만들었다.
이는 source·exact integration head의 Canvas visual diff, frontend gates, Rust 실문서 등가성 회귀를 보완하는
실제 소비자 경로 검증이다.

## 권고

**권고: 수용.** 성능 수치를 일반 renderer 정합 성공으로 확대하지 않고, narrow query와 기존 tree의
등가성·fallback·URL 수명 계약으로 한정한다. #3315는 Track 4 뒤에도 umbrella이므로 open을 유지한다.
#3661 code head의 8 shards와 `Build & Test` 성공, `MERGEABLE`을 확인했다. 문서-only fast-pass 뒤 하나의
통합 PR로 merge한다.
