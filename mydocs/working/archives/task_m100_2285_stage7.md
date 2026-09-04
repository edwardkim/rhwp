# 단계별 계획 S7 - 실제 Chrome 최근 문서 계약 재정렬 (M100 #2285)

- **이슈**: [edwardkim/rhwp#2285](https://github.com/edwardkim/rhwp/issues/2285)
- **대상 PR**: [edwardkim/rhwp#2286](https://github.com/edwardkim/rhwp/pull/2286)
- **브랜치**: `codex/task2285-handle-only-20260716`
- **단계**: S7 / 요구 재정렬
- **작성일**: 2026-07-16

## 계기

S5는 IAB의 격리 프로필 결과를 실제 Chrome 결과처럼 기록했고, URL/드롭/input 경로를 최근 문서에서
제외하도록 보정했다. 실제 Chrome에서 `saved_single_line_spacing_after.hwpx`를 연 뒤 최근 문서에
한 건이 표시되는 것이 제품의 정상 요구임을 확인했다. 따라서 S5의 “최근 문서 없음” 검증 서술과
handle-only 보정은 요구와 맞지 않는다.

## 이번 단계 범위

- 일반 문서 열기(URL, 드롭, input 포함)는 파일명·형식·시각만으로도 최근 문서에 기록한다.
- `FileSystemFileHandle`이 있으면 라이브 파일을 직접 재열고, 없으면 최근 항목 선택 시 파일 선택기로
  다시 선택하도록 한다.
- 문서 바이트 스냅샷은 저장하지 않는다.
- S5/S6의 사실과 다른 IAB 검증 서술을 실제 Chrome 관찰과 위 계약에 맞춰 정정한다.

## 완료 기준

1. 현재 URL 로드 문서가 최근 문서 메뉴에 한 건으로 표시된다.
2. handle 없는 최근 항목은 자동 재열기 대신 파일 다시 선택 경로를 사용한다.
3. handle 있는 항목의 권한 거부·파일 이동/삭제 처리와 최대 8개 상한 회귀 테스트를 유지한다.
4. 문서에는 IAB 격리 프로필 결과와 실제 Chrome 프로필 결과를 구분해 기록한다.

## 결과

1. 실제 Chrome 프로필의 기존 `saved_single_line_spacing_after.hwpx` 최근 항목을 확인했다.
2. 같은 프로필의 새 탭에서 URL로 동일 파일을 `codex-url-probe-2285.hwpx` 이름으로 열었다.
   문서는 1페이지로 로드됐고, 파일 메뉴에는 신규 항목과 기존 항목이 함께 표시됐다.
3. `node --test tests/recent-store.test.ts tests/recent-open.test.ts`는 12/12 통과했고,
   `npx tsc --noEmit`도 통과했다.
4. `rhwp-studio`의 `npm run build`와 `rhwp-chrome`의 `npm run build`가 통과했다.
   CanvasKit browser externalization, chunk-size 및 Chrome build의 정적 자산 경고는 기존 경고다.

## 결론

`2d6884b`의 handle-only 보정은 제품 요구와 반대이므로 최종 이력에서 제외한다. S5의 IAB 결과는
격리 프로필의 빈 저장소 관찰일 뿐 실제 Chrome 검증이 아니었음을 S5/PR review/최종 보고서에 정정했다.
