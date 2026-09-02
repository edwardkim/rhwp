# PR #4816 검토 - HWP/HWPX 글꼴 웹폰트 전수 조사 확장

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4816](https://github.com/edwardkim/rhwp/pull/4816) |
| 관련 이슈 | [#4812](https://github.com/edwardkim/rhwp/issues/4812) |
| 작성자 | `jangster77` (`Taesup Jang`) |
| 검토 방식 | collaborator 셀프 리뷰 |
| base / head | `devel` / `task_m100_4812_font_webfont_survey` |
| code candidate | `1d8d0e5e038cac72aba5383fdbd3a5fc3407b7d4` |
| 규모 | 2 commits, 6 files, +34,039 / -3 |
| 작성 시점 상태 | `MERGEABLE`, CI 시작 전 `BLOCKED` |

작성 시점에 reviewer 요청은 없다. collaborator 셀프 PR 정책에 따라 reviewer를 지정하지 않았으며,
최종 merge 전에는 trailing head의 GitHub Actions와 mergeability를 다시 확인한다.

## 변경 범위와 판단

- `rhwp batch info --json`이 DOCINFO의 첫 글꼴군만 읽던 범위를 7개 글꼴군 전체로 확장한다. 따라서
  분석 도구가 문서에 선언된 글꼴을 누락하지 않고 수집할 수 있다.
- `scripts/survey_korea_downloads_font_jsdelivr.mjs`는 HWP/HWPX 단일 파일 또는 디렉터리를 입력받아
  글꼴 선언을 모으고, Fontsource, jsDelivr, Google Fonts CSS API, Noonnu, OnlineWebFonts 응답을 실제로
  조회한다. 웹폰트 사용 가능, 다운로드 가능, 라이선스 검토 필요와 조회 오류를 분리한다.
- 장식 기호, CSS 이스케이프, 일부 인코딩 깨짐, 휴먼컴퓨터 A 계열 접두사를 외부 검색용 이름에만
  정규화한다. 원문 선언명과 검색명은 TSV에 함께 남기므로 검색 편의성 때문에 원 문서 정보를 잃지 않는다.
- 보고서, TSV, 실행 로그는 2026-08-15 코퍼스 실행의 재현 가능한 스냅샷이다. 공급자 응답과 라이선스는
  변경될 수 있으므로 결과를 영구적인 배포 허가로 해석하지 않는다.

## 완료된 검증

- 전체 입력 코퍼스 조사 스크립트를 완료했다. 1,379개 고유 선언 글꼴 중 다운로드 가능 319개,
  웹폰트 사용 가능 59개, 라이선스 검토 필요 260개, 조회 오류 1개를 기록했다. 개별 문서 파싱 실패
  52건은 실행 로그에 파일별로 보존했다.
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`를
  실행해 6,055건 통과, 38건 skip을 확인했다.
- `cargo fmt --check`를 통과했다.
- `cargo clippy --all-targets -- -D warnings`를 통과했다.
- `git diff --check upstream/devel...HEAD`를 통과했다.

## 위험과 후속 범위

- 공급자 검색 결과는 네트워크 가용성, 검색 색인, 각 서비스의 라이선스 표기에 의존한다. `웹폰트 사용 가능`은
  조사 시점에 CSS 또는 웹폰트 자원이 확인되었다는 뜻이며, 실제 제품 배포 전에는 해당 글꼴의 사용 조건을
  별도 확인해야 한다.
- 52건의 파싱 실패와 대체 불가능한 손상 글꼴명은 원본 문서 또는 파서 지원 범위의 후속 조사 대상이다.
  이번 변경은 실패를 숨기지 않고 로그와 보고서에 남긴다.
- 대용량 조사 로그와 TSV는 의도된 증적 산출물이다. 이후 재조사 시에는 새 실행일 기준 산출물로 갱신한다.

## 최종 권고

현재 코드와 증적 범위는 merge를 권고한다. 다만 이 review·오늘할일 trailing head를 push한 뒤,
GitHub Actions가 완료되고 최신 head가 `MERGEABLE`인지 확인한 후에만 merge를 진행한다.
