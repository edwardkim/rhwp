# #7091 검토 증거 — 머지 보류

독립 정본은 `samples/hwp3-sample11-hwp5.hwp` 및
`tests/fixtures/issue_4680/german-legislative-system-hancom-2020.hwp`이다.
원본 HWP3는 각각 `samples/hwp3-sample11.hwp`와
`tests/fixtures/issue_4680/german-legislative-system.hwp`다. 모두 기존 Git blob이다.

- `sample11-before.hwp`: devel `1ae5ca295` 런타임의 HWP3→HWP5 검사 산출물.
- `sample11-candidate.hwp`, `german-candidate.hwp`: #7091을 포함한 초기 통합
  `9a39b8f6b` 런타임의 같은 변환 산출물. `c1a4d3345` 빌드와도 byte-identical 확인.
  최종 수용 후보에는 #7091 코드를 되돌렸으므로 이 파일들은 **보류 증거**다.
- `pr7091-sample11-candidate-hancom.hwpx`: 위 후보를 한컴 engine 2020에서 실제로
  열고 HWPX로 저장한 산출물. 독립 원본 정본이 아니며 PDF 성공 증거도 아니다.

2026-09-13 MCP 실측, engine 2020 / Hancom 12.0.0.4605 / 32-bit direct worker:

| 입력/출력 | job | 결과 |
| --- | --- | --- |
| sample11 후보 → PDF | `36ac4623-2606-4ce2-a213-ccfdc71a9e14` | 21초, converting_document worker 종료, exit 3762504530 |
| sample11 후보 → PDF 재확인 | `b62cc57a-8aa0-4874-a135-20057e35355d` | 15초, 동일 worker 종료 |
| sample11 수정 전 → PDF | `caf56a2e-7d4b-418e-97ef-ae510c4a4a99` | 148초 성공, 151쪽 |
| sample11 독립 한컴 HWP5 → PDF | `9e0c2859-5677-4894-84eb-5e8d313f918b` | 147초 성공, 151쪽 |
| sample11 후보 → HWPX | `ec46a3db-b91a-46f8-8a2b-5e8ebcd7876d` | 9초 성공 |
| 독일 법령 후보 → PDF | `9aef009a-e72a-41f3-9a30-6cc99f90dd02` | 59초 성공, 326쪽 |

수정 전 후보의 새 PDF는 `pdf/pr7091-sample11-before-2020.pdf`다.
독립 한컴 정본 PDF는 기존 `pdf/hwp3-sample11-hwp-2020.pdf`를 참조한다.
같은 독립 HWP5를 재변환한 control PDF는 새 이름으로 커밋하지 않는다.
새 control도 151쪽·추출 텍스트가 기존 정본과 같지만 raster는 차이가 있으므로 byte/pixel 동일성을 주장하지 않는다.
독일 법령 후보의 새 PDF는 기존 `pdf/german-legislative-system-candidate-2020.pdf`와
326쪽 전체 72dpi 픽셀 및 추출 텍스트가 동일하므로 새 이름으로 중복 커밋하지 않는다.
검토 기록은 기존 경로와 새 변환 job을 참조한다. 326쪽 일치가 원본 완전 충실도를 뜻하지 않는다.

그룹 깊이·행렬 쌍 전수 census는 후보가 독립 한컴 HWP5와 1,392개·358개 각각 일치한다.
기존 sample11은 모든 도형이 (0,1)이었고, 후보/정본은 깊이 0~9다.
최상위 도형의 두 ctrl_id에 대한 +4 byte 오프셋까지 적용하여 25개를 직접 검사했다.
HWP3 원본 SVG도 sample11 151쪽·독일 법령 261쪽 전부 기준/초기 후보가 byte-identical이다.
**이 구조·SVG 결과만으로 PDF 출력 회귀를 통과시킬 수 없다.**

현재 증거는 한컴 PDF 출력 호환성 회귀를 특정한다. HWPX 저장 성공 때문에 일반적인
문서 개방 실패로 확대하지 않는다. 내부 한컴 예외의 정확한 원인은 아직 미확정이다.
원 PR의 Hancom 2024 COM 개방 주장을 이번 engine 2020 PDF 검증 결과로 대체하지 않는다.
#7091·#4680을 close하거나 전체 HWP3 저장 호환성 해결로 기록하지 않는다.

보류 해제: 같은 원본·한컴 버전으로 수정 전/후 PDF 출력 회귀를 해소하고, 깊이 0~9의
독립 census·실제 최상위 레코드·HWP5 재저장·가시 출력까지 함께 재검증한다.
