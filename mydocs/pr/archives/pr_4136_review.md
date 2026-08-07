# PR #4136 검토 기록

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#4136](https://github.com/edwardkim/rhwp/pull/4136) |
| 작성자 | `planet6897` |
| 대상 | `devel` |
| 원 코드 head | `cdde6b8151464caf7318f2e3b36e107214589d9b` |
| 보정 뒤 code head | `7f4c92c361a25e6f7ad3ddfe9710d2a69a426858` |
| 작성 시점 merge 상태 | `MERGEABLE` / `CLEAN`, merge 전 최신 상태 재확인 필요 |

## 변경 검토

이 PR은 한글 2018·2020·2022·2024의 페이지네이션 차이를 수집하는 PowerShell 오라클과
측정 보고서를 추가한다. `src/` 렌더러, 저장 포맷, 기준 PDF는 변경하지 않는다. 따라서 이 PR 자체는
시각 sweep 대상이 아니다.

기여자 원 변경의 범위는 `mydocs/manual/verification/`, 측정 보고서, `tools/hangul_version_oracle/`다.
원 contributor commit은 재작성하지 않았고, maintainer가 다음 세 보정을 원 head 뒤에 별도 commit으로
추가했다.

| commit | 구분 | 내용 |
| --- | --- | --- |
| `c2d8a7039` | 도구 보정 | `hwp.Version`의 dotted/comma 형식에서 major를 안전하게 추출하도록 worker를 보정하고, Windows PowerShell 5.1 ASCII 규칙에 맞춰 BOM을 제거했다. |
| `8c702735d` | 기여 안내 | 외부 기여자가 `mydocs/orders/YYYYMMDD.md`를 PR에 포함하지 않도록 `CONTRIBUTING.md`에 명시했다. |
| `7f4c92c36` | 범위 정리 | 기여자 원 변경에 섞인 오늘할일 항목을 제거했다. 기존 메인터너 운영 기록은 보존했다. |

`page_oracle_worker.ps1`은 기존에 `12.0.0.4547`처럼 점(`.`)으로 구분된 `hwp.Version` 값을 정수로 직접
변환하려 했다. 저장소의 기존 Windows 증적에 있는 이 형식에서는 worker가 실제 버전과 무관하게
version mismatch로 종료할 수 있으므로 차단 결함으로 판단했다. `Get-HwpMajor`는 첫 숫자 구간만 읽어
`12.0.0.4547`, `12,0,0,535`, `13.0.0.3901`을 각각 12, 12, 13으로 처리한다.

## 수행한 검증

- Linux: `git diff --check upstream/pr4136-head..HEAD`를 통과했다.
- Linux: `python3 scripts/check_markdown_links.py CONTRIBUTING.md mydocs/manual/verification/README.md mydocs/manual/verification/hangul_version_oracle.md mydocs/report/hangul_version_oracle_r1_20260807.md`를 실행해 내부 상대 링크 오류가 없음을 확인했다.
- Linux: `tools/hangul_version_oracle/`의 `.ps1` 파일에 non-ASCII 바이트가 없고, `scan_appversion.ps1`이 BOM 없이 `#`로 시작함을 확인했다.
- Windows 10 / Windows PowerShell 5.1: 원 PR의 PowerShell 스크립트 8개를 `Parser::ParseFile`로 검사해 구문 오류 0건을 확인했다.
- Windows 10 / Windows PowerShell 5.1: `list_hangul_versions.ps1` 실행에서 2018·2022·2024 설치 정보를 확인했다. 기존 `Hwp.exe`가 실행 중이어서 도구의 안전 규칙에 따라 실제 COM probe는 건너뛰었다. 다른 작업의 한글 프로세스를 종료하지 않았다.
- Windows 10 / Windows PowerShell 5.1: 보정한 major parser에 `12.0.0.4547`, `12,0,0,535`, `13.0.0.3901`을 입력해 각각 12, 12, 13 결과를 확인했다.

원본 10k 코퍼스 전체와 한글 COM 실측은 해당 Windows 환경의 실행 중인 한글 프로세스 때문에 이 검토에서
재실행하지 않았다. PR 보고서는 2018·2020 3자 원시값의 재현 검증이 미완료임을 이미 명시하며, 이 상태는
새 오라클 도구의 설치·정적 검토 결과와 구분한다.

## CI와 결론

보정 code head `7f4c92c361a25e6f7ad3ddfe9710d2a69a426858`의
[GitHub Actions CI #4136](https://github.com/edwardkim/rhwp/actions/runs/31198985288)와 CodeQL이 성공했다.
Native Skia, 세 archive build, regular shard 3개, slow shard까지 모두 성공했고, WASM Build와 frontend unit
gate는 변경 범위상 skip됐다. 이 문서와 implementation 기록만 뒤이어 추가한 head는 review-only fast-pass
조건을 적용한다.

권고는 **수용**이다. 조건은 다음과 같다.

1. 이 review-only 기록을 추가한 최신 head의 fast-pass aggregate가 성공해야 한다.
2. merge 직전에 PR head SHA, mergeable 상태, required check를 다시 확인해야 한다.
