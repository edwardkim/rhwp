# PR #2286 검토 정정 기록

- **PR**: [#2286](https://github.com/edwardkim/rhwp/pull/2286)
- **관련 이슈**: [#2285](https://github.com/edwardkim/rhwp/issues/2285)
- **기준 커밋**: `d0242ec35` (`최근 문서 메타-only 기록 확장`)
- **작성일**: 2026-07-16

## 정정 사유

S5는 이슈 본문의 handle-only 설명만으로 제품 요구를 단정하고, `d0242ec35`의 meta-only 최근 문서
기록을 제거했다. 이후 실제 Chrome에서 문서를 열었을 때 최근 문서에 한 건이 표시되어야 함을
확인했다. 따라서 S5의 code/test 변경과 IAB 검증 결론은 모두 잘못됐고, `2d6884b`는 최종 PR
이력에 포함하지 않는다.

## 실제 계약

1. 모든 일반 문서 열기는 최근 목록에 파일명·형식·시각을 기록한다.
2. `FileSystemFileHandle`이 있으면 함께 보관해 라이브 파일을 재연다.
3. handle 없는 항목은 바이트를 저장하지 않으며, 선택하면 파일 선택기를 열어 사용자가 다시 고른다.
4. handle structured clone 실패도 meta-only로 기록해 최근 문서 표시를 유지한다.
5. 동일 핸들은 최신화하고, 다른 handle은 동명이어도 공존한다. 목록은 최신순 최대 8개다.

## 실제 Chrome 확인

- `saved_single_line_spacing_after.hwpx`를 열어 둔 실제 Chrome 프로필은 최근 문서에 해당 항목을
  표시했다.
- 같은 프로필의 새 Chrome 탭에서 URL 로드한 `codex-url-probe-2285.hwpx`도 기존 항목과 함께
  표시됐다.
- IAB에서 본 `(최근 문서 없음)`은 격리된 새 프로필의 빈 IndexedDB 상태였고, 실제 Chrome 프로필
  동작을 검증한 결과가 아니다.

## 배포 표면 기록

S6의 VS Code webpack compile, npm SDK 및 package dry-run 기록은 유지한다. 당시에는 잘못된
handle-only 보정 소스를 빌드했으므로, 현재 meta-only 소스로 Studio와 Chrome extension build를
다시 실행했다. 두 build 모두 통과했고 기존 CanvasKit/chunk-size 및 Chrome static asset 경고만 확인했다.

## 정리 원칙

- S5/S6 문서는 history를 숨기지 않고, 실제 요구와 다르다는 정정 사실을 명시한다.
- `2d6884b`와 그 검증 기록만을 근거로 contributor PR head를 수정하거나 merge 보류하지 않는다.
- 사용자 승인에 따라 이 정정 기록과 오늘할일을 현재 PR head에 포함한다. 최신 CI 통과 뒤
  PR/이슈 후속 처리와 merge를 수행한다.
