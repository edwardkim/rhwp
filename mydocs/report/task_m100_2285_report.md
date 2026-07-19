# 최종 결과보고서 — 파일 메뉴 '최근 문서' 서브메뉴 (M100 #2285)

- **이슈**: [edwardkim/rhwp#2285](https://github.com/edwardkim/rhwp/issues/2285)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task2285` (from `local/devel`)
- **완료일**: 2026-07-15
- **계획서**: [`plans/task_m100_2285.md`](../plans/task_m100_2285.md) · [`plans/task_m100_2285_impl.md`](../plans/task_m100_2285_impl.md)

## 1. 요약

rhwp-studio 파일 메뉴에 **"최근 문서" 서브메뉴**를 추가했다. 최근 열었던 문서(최대 8개, 최신순)를 표시하고, 항목 클릭 시 저장된 `FileSystemFileHandle`로 권한 재확인 후 재열기한다. "최근 문서 목록 지우기"도 제공한다.

## 2. 결정 사항 (작업지시자 확인)

- **경로 툴팁 취소**: 브라우저 File System Access API는 파일 절대경로를 노출하지 않아(파일명만 획득) 실현 불가 → 기능 제외.
- **재열기 방식**: `FileSystemFileHandle`을 IndexedDB에 저장(권장안).
- **UI 위치**: 파일 메뉴 서브메뉴(권장안).
- **핸들 없는 파일의 표시**: 드롭·`input[type=file]`·URL로 연 파일도 파일명·형식·시각만 메타-only로
  기록한다. 선택 시 자동 재열기 대신 파일 선택기를 다시 연다.

## 3. 구현 내용

| 파일 | 변경 |
|------|------|
| `src/recent/recent-store.ts` (신규) | IndexedDB `rhwpStudioRecent`/`recent` 저장소. `addRecentDoc`/`listRecentDocs`/`removeRecentDoc`/`clearRecentDocs`. 중복 제거(`isSameEntry`→파일명), 상한 8, `openedAt` 내림차순, 메모리 fallback |
| `src/main.ts` | `loadBytes()`에 일반 열기 기록 훅(핸들이 없으면 meta-only) + `renderRecentSubmenu()` 동적 렌더 + MenuBar `onMenuOpen` 연결 |
| `index.html` | 파일 메뉴에 `.md-sub` "최근 문서"(`#recent-docs-panel`) |
| `src/ui/menu-bar.ts` | `MenuBarOptions.onMenuOpen` 콜백(클릭/hover open 시 호출) |
| `src/command/commands/file.ts` | `file:open-recent`(권한 재확인→재열기, 실패 시 목록 제거), `file:clear-recent` |
| `src/command/file-system-access.ts` | `FileSystemFileHandleLike`에 `queryPermission`/`requestPermission` 타입 |

## 4. 동작

1. File System Access로 문서 열기 → 파일 메뉴 "최근 문서"에 등재(파일명 + 형식 badge).
2. 새로고침 후에도 목록 유지(IndexedDB 영속).
3. 항목 클릭 → 읽기 권한 확인/요청 → 재열기(미저장 문서 확인은 기존 open 규약 재사용).
4. 파일 이동/삭제로 접근 실패 → 안내 토스트 + 해당 항목 자동 제거.
5. 드롭/파일선택/URL로 연 파일 → meta-only로 목록 등재. 선택 시 파일 선택기로 다시 고른다.
6. 목록이 비면 "(최근 문서 없음)" + 서브메뉴 비활성.
7. "최근 문서 목록 지우기" → 확인 후 전체 삭제.

## 5. 검증

- **타입체크**: `npx tsc --noEmit` — 각 단계 통과(exit 0).
- **프로덕션 빌드**: `npm run build`(tsc + vite build) — 통과.
- **런타임 스모크**(puppeteer headless, 임시 스크립트, 3회 반복 안정):
  - 빈 상태: 패널 존재, "(최근 문서 없음)", 서브메뉴 disabled ✓
  - IndexedDB 주입 후: 항목 2개, 최신순 정렬, 파일명/형식/`title`, **파일명 HTML 이스케이프**(`<주의>` textContent), 서브메뉴 활성 ✓
  - "목록 지우기" 클릭 → 0개 ✓
  - 콘솔 에러 0건 ✓

> 임시 스모크 스크립트는 미커밋(정리 완료). 실제 핸들 재열기·권한 프롬프트는 브라우저 상호작용이 필요하여 수동 검증 영역(권한 API/`getFile` 경로는 코드상 표준 사용).

## 6. 한계 / 후속 여지

- 절대경로 표시는 브라우저 제약으로 불가(설계 반영).
- 핸들 없는 열기 경로(드롭/input/URL)는 목록에 meta-only로 남으며, 재열기에는 파일 재선택이 필요하다.
- 최초 메뉴 open 시 async 렌더로 항목이 수십 ms 뒤 나타날 수 있음(hover 이동 시간 내 완료, 실사용 무영향).
- 향후: 개별 항목 제거(우클릭/x 버튼), 목록 개수 설정화 등은 별도 이슈 후보.

## 6-1. 검토 정정 (PR #2286, 2026-07-16)

S5 메인터너 검토는 이슈 본문의 handle-only 서술을 제품 요구로 단정해 meta-only 기록을 제거하려 했다.
실제 Chrome에서 문서를 연 뒤 최근 문서에 한 건이 보여야 하는 제품 요구와 다르므로, 이 보정은
철회한다. `2d6884b`는 원격 PR 이력에 포함하지 않는다.

- 모든 일반 열기는 최근 목록에 기록한다. 저장 바이트는 없다.
- handle이 있는 항목은 라이브 파일 재열기, 없는 항목은 파일 다시 선택으로 분기한다.
- 실제 Chrome에서 URL 로드한 `saved_single_line_spacing_after.hwpx`와 별도 probe 문서가 최근 목록에
  함께 표시되는 것을 확인했다.
- IAB의 빈 목록은 격리 프로필 결과이며, 실제 Chrome 결과로 기록하면 안 된다.
- 현재 Studio와 Chrome extension build를 meta-only 소스로 다시 실행해 통과했다. focused 최근 문서
  회귀 테스트도 12/12 통과했다.

## 6-2. 배포 표면 검증 (S6)

- Chrome/Edge 확장: `rhwp-chrome` 실제 viewer build 통과. manifest, viewer, service worker, WASM 산출물
  존재를 확인했다.
- VS Code 확장: 독립 webpack compile 통과. Studio 소스를 import하지 않고 공유 WASM만 사용한다.
- npm: `@rhwp/editor` SDK 15/15 통과, `@rhwp/editor` 및 `pkg` package dry-run 통과.
- Chrome viewer의 `apple-touch-icon` `icons/icon-256.png` 누락은 S5 이전부터 `upstream/devel`에 있는
  packaging 후보이며, 이번 최근 문서 보정의 회귀가 아니다.

## 7. 커밋

- `2ab241fbb` S1 최근 문서 IndexedDB 저장소
- `909ccea63` S2 기록 훅 + 메뉴 동적 렌더
- `5551404f9` S3 재열기/지우기 커맨드
- (본 커밋) S4 빌드·검증 + 최종 보고서
- (정정 기록) S7 실제 Chrome 요구 재정렬 및 검증 문서 보정
