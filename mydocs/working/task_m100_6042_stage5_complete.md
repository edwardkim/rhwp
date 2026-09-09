# Task M100 #6042 Stage 5 종료 보고

- Issue: [#6042](https://github.com/edwardkim/rhwp/issues/6042)
- 측정일: 2026-09-02 KST
- 상태: **Stage 5 완료 — Stage 6 제출 준비 승인 대기**
- 제품 revision: `d685c7669`
- before: Stage 3 `5f5d60071bb403b361e796e6d229d2d5b5a9ebef`, `127.0.0.1:4191`
- after: `codex/issue-6042-page-virtualization`, `127.0.0.1:4193`
- 계획: [수행](../plans/task_m100_6042.md), [구현](../plans/task_m100_6042_impl.md)
- 선행 근거: [확장 matrix](task_m100_6042_stage5_expanded.md),
  [scroll 정착 화질 보정](task_m100_6042_stage5_scroll_quality_correction.md)

## 1. 판정

Stage 5에 남았던 실제 DPR 1, 실제 browser viewport resize, image decode 실패/fallback과 사용자 직접
조작 확인을 모두 채웠다. 따라서 #6042의 Stage 5를 완료한다. Stage 6 제출 준비, push, Draft PR 생성,
stack Ready 전환은 이 판정에 포함하지 않는다.

사용자가 제안한 장치 성능별 자동 확장과 짧은 문서 전체 surface 상주 절충안은 **이번 #6042에 넣지
않는다**. 현재 32M/40M 예산, scroll scheduler, 64M settled-visible gate를 그대로 검증한다. 후속으로
검토하려면 문서 길이·장치 메모리·warm 왕복의 독립 A/B와 별도 계획 승인이 필요하다.

## 2. 실제 DPR 1과 viewport resize

macOS의 설치 Chrome을 Puppeteer headless mode로 실행하고 `devicePixelRatio === 1`을 assertion으로
고정했다. `exam_kor.hwp`를 연 실제 Studio 화면에서 height 768px, 다음 28개 viewport snapshot을
연속 적용했다.

- 1023↔1024px 10회 왕복
- 767, 768, 807, 808, 961, 962, 375, 1280px

모든 snapshot이 실제 `innerWidth`, ruler 표시, editor grid, horizontal/vertical/corner 정렬,
document outer overflow 없음과 ruler 내부 색 다양성 검사를 통과했다.

```text
RULER_RESIZE_SNAPSHOTS_OK 28 snapshots; compositor 전 프레임 검사는 아님
```

검증 과정에서 기존 `ruler-resize.test.mjs`가 현재 Puppeteer에서 `Control+Home`을 단일 key 이름으로
보내 실행 전에 실패하는 것을 확인했다. 이를 `keyboard.down('Control')` → `press('Home')` →
`keyboard.up('Control')`로 바로잡고 headless DPR 1 assertion을 추가했다. 제품 동작 변경은 아니다.

이 검사는 browser의 실제 viewport를 바꾸고 각 정착 화면을 읽지만 OS 창 드래그의 모든 compositor
frame을 보증하지 않는다. 연속 zoom 중 geometry/ruler는 기존 Stage 5 A/B와 사용자의 직접 조작으로
별도 확인했다.

## 3. 실제 browser image 실패/fallback

새 좁은 E2E는 Canvas2D·34%에서 `test-image.hwp`를 열고 embedded data-image `decode()`를 실제
Chromium 안에서 `EncodingError`로 실패시킨다. 문서를 연 뒤 별도 zoom/resize를 만들지 않고
PageRenderer의 1.5초 fallback과 후속 decode/render가 정착할 때까지 기다렸다.

```text
PAGE_VIRTUALIZATION_IMAGE_FAILURE_OK attempts=3 failures=3 fallbackRenders=3
```

최종 상태는 다음과 같다.

- decode 시도 3회, 의도한 실패 3회, 관찰된 PageRenderer 재렌더 3회
- 실패한 decode의 완료 서명 0개
- 연결된 page Canvas 존재, flow image layer `ready`
- `pendingImages=0`, `pendingPrefetch=0`
- visible/prefetch scheduler queue 0
- probe error 0, browser `pageerror` 0

실패를 완료 서명으로 캐시하거나 scheduler 완료를 거짓으로 앞당기지 않고 실제 fallback 뒤 정착한다.
단위 계약인 `completeImagePrefetch(false)`·failed observation과 실제 browser 수명 경로를 함께 고정한다.

## 4. 사용자 직접 조작 결과

사용자는 `:4191`과 `:4193`에서 cursor가 없는 쪽으로 스크롤하는 전후 화질을 직접 비교했고, 이어서
scroll 중 surface 고정과 scroll 종료 뒤 visible 화질 승격 보정을 적용한 현재 서버를 다시 조작했다.
그 결과 앞서 제보한 스크롤 버벅임이 사라진 것 같다고 보고했다. 이번 `원래 계획대로 다음 작업` 지시는
장치별 절충안을 보류하고 해당 보정과 기존 Stage 5 수용 기준으로 진행한다는 최종 범위 확인으로 기록한다.

기계 근거는 [scroll 정착 화질 보정 보고](task_m100_6042_stage5_scroll_quality_correction.md)의 클릭 없는
DPR 1→2 회복, scroll callback p50/p95 동등, page box·scroll·focus 불변 결과다. 사용자 관찰을 시간
개선률로 일반화하지 않고, 자동화 계측과 함께 핵심 체감 비회귀 근거로만 사용한다.

## 5. 최종 Stage 5 게이트

- `npm test`: 1,422개 중 1,421 통과, 1개 스킵, 실패 0
- `npm run build`: TypeScript·Vite production build 통과
- `ruler-resize.test.mjs --mode=headless`: 실제 DPR 1, 28/28 snapshot 통과
- `page-virtualization-image-failure.test.mjs --mode=headless`: 실제 decode 실패/fallback 통과
- 두 E2E의 `node --check`: 통과
- E2E manifest: 새 E2E를 포함해 동기화
- Rust source/test/fixture 변경 없음. Rust lint bundle 대상이 아니다.

Vite의 CanvasKit `fs`/`path` externalization과 500kB chunk 경고는 기존 경고이며 build 실패가 아니다.

## 6. 다음 경계

다음은 Stage 6 제출 준비다. 전체 범위 감사, 최종 report, PR 본문 초안과 stack/base 확인을 수행한 뒤
커밋한다. branch push와 #6467 위 native Draft PR 생성은 그 뒤 별도 승인으로만 진행하며, Ready 전환과
merge는 하지 않는다.
