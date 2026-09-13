# #7091 검토 증거 — 초기 보류와 메인터너 보정

독립 정본은 `samples/hwp3-sample11-hwp5.hwp` 및
`tests/fixtures/issue_4680/german-legislative-system-hancom-2020.hwp`이다.
원본 HWP3는 각각 `samples/hwp3-sample11.hwp`와
`tests/fixtures/issue_4680/german-legislative-system.hwp`다. 모두 기존 Git blob이다.

- `sample11-before.hwp`: devel `1ae5ca295` 런타임의 HWP3→HWP5 검사 산출물.
- `sample11-candidate.hwp`, `german-candidate.hwp`: #7091을 포함한 초기 통합
  `9a39b8f6b` 런타임의 같은 변환 산출물. `c1a4d3345` 빌드와도 byte-identical 확인.
  초기 후보에서 되돌린 뒤 `64605f37d`로 재적용했다. 현재 변환 출력도 이 두 파일과 byte-identical이다.
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
구조·SVG 검사와 별도로, 아래 개선된 서버에서 실제 PDF 출력 성공과 전후 동일성을 확인했다.

초기 증거에서는 후보의 한컴 PDF 출력 실패가 반복됐다. 아래 서버 개선 후 재검증에서는 성공했다. HWPX 저장 성공 때문에 일반적인
문서 개방 실패로 확대하지 않는다. 내부 한컴 예외의 정확한 원인은 아직 미확정이다.
원 PR의 Hancom 2024 COM 개방 주장을 이번 engine 2020 PDF 검증 결과로 대체하지 않는다.
#7091·#4680을 close하거나 전체 HWP3 저장 호환성 해결로 기록하지 않는다.

보류 해제: 같은 원본·한컴 버전으로 수정 전/후 PDF 출력 회귀를 해소하고, 깊이 0~9의
독립 census·실제 최상위 레코드·HWP5 재저장·가시 출력까지 함께 재검증한다.

## MCP 서버 개선 후 재검증 (2026-09-13)

동일 `sample11-candidate.hwp`를 수정 없이 engine 2020 / Hancom 12.0.0.4605로 재변환했다.
Job `03325817-45fb-4d2b-8012-c54ea9d891a9`는 148초에 성공했고 PDF는 151쪽이다.
5·34·67·109·140초 status는 running/converting, 최종은 succeeded/completed였다.
출력은 로컬 `pdf/pr7091-sample11-candidate-2020.pdf`(아직 미커밋), 25,612,183 bytes,
SHA-256 `5adf04ec8f15c6e9c8d28f3d8561c75d6ec1d27a4b9e21620945799331a7ba38`다.
서버·로컬 해시 일치 및 151쪽 전체 래스터화 성공을 확인했다.
초기 worker 종료는 재현되지 않았다. PDF 내용의 독립 정본 대조와 최종 PR 수용 검토는 별개다.


## 메인터너 보정 검증

`64605f37d`에서 원 변경을 재적용하고 최상위 레코드 4-byte 오프셋, fixture/record 누락 실패,
독립 한컴 census 전수 비교와 HWP5 재저장 테스트를 보강했다. 보강 Rust 테스트 3개를 실제 실행해 통과했다.
현재 바이너리의 HWP5 산출물도 위 sample11/german candidate와 byte-identical이다.

새 sample11 PDF는 기존 `pdf/pr7091-sample11-before-2020.pdf`와 151쪽 전체 72dpi 픽셀·추출 텍스트가
같으므로 새 이름으로 중복 커밋하지 않는다. 기존 PDF를 재사용하고 새 변환 job과 해시만 기록한다.
독립 한컴 정본과의 기존 151쪽 raster·33쪽 text 차이는 수정 전에도 같았으며 전체 충실도 해결을 주장하지 않는다.
새 코드의 HWP3 SVG 151+261쪽도 수정 전과 동일했다. 최종 검토는 `mydocs/pr/archives/pr_7091_review.md`를 따른다.
