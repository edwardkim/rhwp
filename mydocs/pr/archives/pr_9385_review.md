# PR #9385 한컴 MCP 대기열 조회와 클라이언트 0.10.0 셀프 리뷰

제출 커밋 `0c4f1c8918a87c7251156d15565dcf59d3354987`을 `devel` 기준 `2c0c1d205a54b8d5bd262241bac1391cffe0d801`과 대조했다. [PR #9385](https://github.com/edwardkim/rhwp/pull/9385)

## 검토 범위

- `npx`로 `queue-status`를 호출하고, 같은 서버가 `idle`을 반환할 때만 새 변환을 제출한다. [HWP 2024 MCP 사용법](../../manual/mcp_hwp2024Convert_usage.md#새-변환을-시작하기-전-대기열-확인)
- 클라이언트 0.10.0을 추가하고 기존 배포 패키지를 제거했다. 기준 PDF 갱신 도구의 기본 패키지 경로도 함께 변경했다. [기준 PDF 갱신 도구](../../../tools/refresh_oracle_pdfs.py)
- 새 패키지와 매뉴얼에서 VS Code 설정 예제를 제거했다. 최종 패키지의 SHA-256과 파일 구성이 매뉴얼 및 배포 증적과 일치했다.

## 로컬 및 배포 검증

- HTTP·HTTPS·stdio 테스트 14개와 기준 PDF 갱신 도구 테스트 3개가 통과했다.
- 배포 서버에서 한컴 2020·2024 각각 `npx`로 상태 조회, 변환, 다운로드를 실행했다. 두 엔진 모두 `idle` → `busy` → `idle`을 반환했고, 1쪽 PDF의 `Creator`·크기·SHA-256이 일치했다.
- 최종 패키지를 새 npm 캐시에서 실행해 상태를 다시 조회했다. 변경한 매뉴얼의 링크·메타데이터 검증을 통과했다.

## 제출 커밋의 GitHub 검사

검사 32개가 성공했고 6개가 실행 조건에 따라 건너뛰어졌다. 실패와 대기는 0개였다. 최종 `CI Impact Policy` 상태는 성공이었다. [최신 제출 검사](https://github.com/edwardkim/rhwp/pull/9385/checks)

| 검사 | 결과 |
| --- | --- |
| Build & Test | [성공](https://github.com/edwardkim/rhwp/actions/runs/45347722677) |
| CodeQL | [성공](https://github.com/edwardkim/rhwp/actions/runs/45347722756) |
| Adapter inter-diff | [성공](https://github.com/edwardkim/rhwp/actions/runs/45347722713) |
| Proptest roundtrip | [성공](https://github.com/edwardkim/rhwp/actions/runs/45347722556) |

이 기록을 포함한 최신 커밋의 필수 검사와 병합 가능 상태를 확인한 뒤 병합한다.
