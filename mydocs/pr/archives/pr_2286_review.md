# PR #2286 검토 정정 — 일반 문서 최근 목록 기록

- **PR**: [#2286](https://github.com/edwardkim/rhwp/pull/2286)
- **관련 이슈**: [#2285](https://github.com/edwardkim/rhwp/issues/2285)
- **작성일**: 2026-07-16
- **작성자**: `planet6897`
- **base**: `devel`

## 검토 결론

이전 검토는 이슈 본문의 handle-only 설명을 제품 요구로 잘못 확정했다. 실제 Chrome에서 일반 문서를
열면 최근 문서에 항목 한 건이 표시되어야 한다. 따라서 handle 없는 URL/드롭/input 로드를 메타-only
항목으로 기록하는 `d0242ec35`의 방향이 현재 제품 요구에 맞다. 단, 문서 바이트를 IndexedDB에
보관하지 않고, handle 없는 항목은 파일 선택기로 다시 선택하게 해야 한다.

## 재현과 범위 대조

- 실제 Chrome 프로필에서 `saved_single_line_spacing_after.hwpx`가 최근 문서에 표시되는 것을
  확인했다. 이는 정상 기대값이다.
- 같은 Chrome 프로필의 새 탭에서 URL로 동일 HWPX를 `codex-url-probe-2285.hwpx`라는 새 이름으로
  열었고, 두 항목이 함께 최근 문서에 표시됐다. 신규 URL 로드가 목록에 기록됨을 확인했다.
- `src/recent/recent-store.ts`는 handle 유무와 관계없이 파일명·형식·시각만 기록하며, handle이
  있으면 라이브 재열기에 사용한다. structured clone 실패도 메타-only로 저장해 목록 표시를 유지한다.
- `src/recent/recent-open.ts`와 `file:open-recent`는 handle 없는 항목을 선택하면 안내 후 파일 선택기를
  열고, handle 있는 항목만 권한 확인 뒤 라이브 파일을 재연다.
- 렌더러, 문서 파서, WASM 출력에는 변경이 없으므로 visual sweep 대상은 아니다.

## 수용 조건

1. 모든 일반 문서 열기(URL, 드롭, input, File System Access)는 최근 목록에 한 건을 기록한다.
2. 저장 대상에는 문서 바이트가 없고, handle이 없는 항목은 파일명·형식·시각만 가진다.
3. handle 있는 항목은 라이브 파일 재열기, handle 없는 항목은 파일 다시 선택으로 분기한다.
4. 메타-only 기록, 동일 핸들 최신화, 동명 다른 handle 공존, 권한 거부/파일 이동/목록 지우기,
   최대 8개 상한 회귀를 검증한다.
5. 실제 Chrome URL-load와 최신 Studio/Chrome 확장 build를 다시 확인한다.

## 보정 검증

- `node --test tests/recent-store.test.ts tests/recent-open.test.ts` — 12/12 통과.
- `npx tsc --noEmit` — 통과.
- `rhwp-studio`의 `npm run build` 및 `rhwp-chrome`의 `npm run build` — 통과.
  CanvasKit browser externalization, chunk-size 및 Chrome build의 기존 정적 자산 경고만 발생했다.
- 실제 Chrome에서 URL 로드 후 최근 문서가 표시되고 서브메뉴가 활성화됨을 확인했다.

## 배포 표면 확인

- Chrome/Edge 확장: `rhwp-chrome`의 실제 확장 viewer build가 통과했고 manifest, viewer, service worker,
  WASM 산출물이 모두 존재한다. Studio 보정이 확장 viewer에 정상 포함된다.
- VS Code: Studio를 import하지 않는 독립 webpack webview/extension host이며 `npm run compile`이 통과했다.
- npm: `@rhwp/editor` SDK 테스트 15/15와 `@rhwp/editor`/`pkg`의 `npm pack --dry-run`이 통과했다.
- Chrome viewer의 `apple-touch-icon` `/icons/icon-256.png`는 현재 `upstream/devel`에도 있는 기존 패키징
  누락 후보다. 최근 문서 보정과 무관하며 manifest/action과 viewer 실행 자산은 완전하다.

이전 S5의 IAB 결과는 격리 프로필의 빈 IndexedDB를 실제 Chrome 결과처럼 기록한 오류였으며,
handle-only 보정 커밋 `2d6884b`는 최종 이력에서 제외한다. 정정 경위는
[구현 기록](pr_2286_review_impl.md)에 남긴다.

현재 [#2285](https://github.com/edwardkim/rhwp/issues/2285) 본문의 “핸들 없는 파일 제외” 문구는
실제 제품 요구와 다르다. 이 PR을 close 근거로 사용할 때는 해당 설명도 meta-only 기록 계약으로
정정해야 한다.

## 처리 계획

이 검토 기록과 구현 정정 기록, 오늘할일은 옵션 1로 현재 PR head에 함께 포함한다. PR 본문과
관련 이슈의 범위를 meta-only 기록 계약으로 정정한 뒤, 최신 head의 CI가 모두 통과하면 merge하고
이슈 close 및 PR 후속 코멘트를 확인한다.
