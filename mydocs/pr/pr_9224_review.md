# PR #9224 검토 — feat(doc-info): 문서 요약을 읽고 쓰고, 그림 파일 목록·저작권 표시·개체 수를 준다

## 최종 판정

**머지 보류.** 제목만 변경해도 별도 메타데이터의 CDATA 값이 소실된다. 달력에 없는 날짜도 저장 형식별로 다르게 처리된다.

## 접수와 검토 범위

- 작성자 및 브랜치 `semanticist21/pr/doc-info`
- 검토 head `527ceb0bc3374aa0d6e173142d6b0d840f27d689`
- 리뷰어 `edwardkim`, 요청 시각 `2026-11-20T10:37:33Z` (UTC), GitHub 요청 목록 재확인
- 최신 devel `0a437a7eefb19bfe81fc12866887e4548ea9554d` 기준 개별 및 번호순 누적 병합 시뮬레이션 성공
- 원 head의 Build & Test 및 CI Impact Policy 성공. [CI 집계](https://github.com/edwardkim/rhwp/actions/runs/44666605507/job/136068314907)

제목만 바꿔도 Contents/content.hpf의 전체 metadata를 재구성한다. metadata_entries는 CDATA를 읽지 않는다. 날짜 유효성 검사에서 월별 일수를 확인하지 않아 HWP FILETIME과 HWPX 문자열이 서로 다른 날짜를 나타낼 수 있다.

## 메인테이너 실행 결과

누적 적용 5번째, 적용 직후 `4dbd79d12c`. 충돌 없이 적용됐다. 처음 10건을 합친 `c6407bf83c`에서 이 PR의 집중 검사 4개가 통과했다.

HWPX의 `Contents/content.hpf`에 값이 `editorial note`인 CDATA 메타 항목을 넣고 제목만 수정했다. 저장 ZIP에서 원래 항목의 텍스트가 비었다. `2026-02-30T12:00:00Z`를 등록하는 명령도 성공했다. HWP로 다시 열면 3월 2일이고 HWPX로 다시 열면 2월 30일이다. 관련 없는 메타 값의 왕복 보존과 유효하지 않은 날짜의 일관된 거절을 확인해야 한다.

공개 API 대조 대상은 `getDocumentSummary, setDocumentSummary, getBinDataList, getLicenseMark, getObjectCount`이다. 직접 빌드한 WASM 패키지에서는 메서드 존재와 TypeScript 컴파일을 확인했다. 추적 중인 `rhwp-studio/public/rhwp.d.ts`를 import한 같은 소비자 코드는 타입 검사에서 실패했다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9224](https://github.com/edwardkim/rhwp/pull/9224)의 head는 `527ceb0bc3374aa0d6e173142d6b0d840f27d689`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

제목만 바꾼 HWPX에서 별개의 CDATA 메타데이터가 사라진다. 2월 30일을 입력하면 HWP와 HWPX가 서로 다른 날짜를 반환한다. 알 수 없는 메타데이터를 보존하고 유효하지 않은 날짜를 거절하는지 두 형식 저장 후 재열기로 검증한다.

## 메인테이너 보완 검증 중간 결과

제목만 바꾼 뒤 중첩 확장 XML과 CDATA를 HWPX 재열기에서 확인했다. 윤년 2000년 2월 29일은 수용되고 1900년 2월 29일·2026년 2월 30일은 다른 요약 칸 변경 전 거절됐다. [쪽별 비교 원장](assets/semanticist21-20261121/maintainer-comparison.tsv)
