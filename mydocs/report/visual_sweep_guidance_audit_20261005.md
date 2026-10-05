---
kind: investigation
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-10-05
---

# 조판 변경의 Visual Sweep 제출 지침 감사

## 범위와 확인 사실

검토 브랜치는 `review/semanticist21-20261005`, 지침 수정 전 head는
`112f069f6563edcfd48546b8dbf9aecdc5fb1423`이다. 누적 체리픽한 semanticist21의 non-draft
PR 24개 원문과 이번 대화에서 확인한 실패·보정 기록을 대조했다. 개별 PR 판정은 기존 개별 archive
review에 남긴다. 이 문서는 제출 지침의 누락과 모호한 표현을 조사한 기록이다.

24개 PR 본문 모두 `TSV` 또는 `silhouette.tsv` 제출 기록이 없었다. 이것은 본문 증거의 부재이며
기여자가 비공개로 무엇을 실행했는지까지 입증하지 않는다. 원문 조회 JSON은 ignored
`output/pr-review/semanticist21-20261005/source-pr-final-metadata.json`에만 보존한다.

| 원 PR | 확인한 head (앞 12자리) | 원문에서 확인한 검증 범위·생략 사유 또는 변경 동작 |
| --- | --- | --- |
| [#7491](https://github.com/edwardkim/rhwp/pull/7491) | `c4367ec03a28` | 편집 중 문제라 기준 PDF가 없다고 하며 편집 전수 비교로 대체 |
| [#7504](https://github.com/edwardkim/rhwp/pull/7504) | `7dc340284bd8` | 호스트 글꼴 등록 시 출력이 바뀌지만 같은 조건의 한컴 출력이 없어 비해당 |
| [#7508](https://github.com/edwardkim/rhwp/pull/7508) | `97dd139fcd58` | 맨 앞 붙여넣기. renderer 무변경을 Sweep 비해당 사유로 기록 |
| [#7510](https://github.com/edwardkim/rhwp/pull/7510) | `b23337121b6d` | 빈 문단의 글자 모양 보존 |
| [#7517](https://github.com/edwardkim/rhwp/pull/7517) | `b82590c79b80` | batch 글자 모양 복원·재조판 |
| [#7519](https://github.com/edwardkim/rhwp/pull/7519) | `3d67c3efe3d6` | 셀 병합. 문서 열기 경로/renderer 무변경을 비해당 사유로 기록 |
| [#7520](https://github.com/edwardkim/rhwp/pull/7520) | `8d3af6175224` | 셀 목록 캐럿·선택 영역 |
| [#7521](https://github.com/edwardkim/rhwp/pull/7521) | `6f0df1ae38d6` | pasteHtml 경로만 바뀌므로 Sweep 비해당으로 기록 |
| [#7522](https://github.com/edwardkim/rhwp/pull/7522) | `e84767a58b47` | 셀 개요 탐색 주소 |
| [#7527](https://github.com/edwardkim/rhwp/pull/7527) | `34310be3b84c` | 단 정의 저장. renderer/조판 코드 무변경을 비해당 사유로 기록 |
| [#7530](https://github.com/edwardkim/rhwp/pull/7530) | `c3c0c070fa90` | 저장 사본 필드 변환. renderer 무변경으로 미실행, 재열기 쪽 번호 배치 문제도 기재 |
| [#7558](https://github.com/edwardkim/rhwp/pull/7558) | `61875ba4ced2` | 셀 높이 편집 시 너비 보존. model/header 변경이라 Sweep 비해당 |
| [#7560](https://github.com/edwardkim/rhwp/pull/7560) | `96323f506bf5` | 각주 본문 캐럿 원본 주소 보존 |
| [#7561](https://github.com/edwardkim/rhwp/pull/7561) | `3ee6ad8e0222` | 선택 조회의 페이지 tree 캐시 재사용 |
| [#7562](https://github.com/edwardkim/rhwp/pull/7562) | `00fa31e4cc94` | 저장 LineSeg 축 변경. parser/model/serializer 범위로 비해당, rhwp 저장 전후 자기 비교 |
| [#7565](https://github.com/edwardkim/rhwp/pull/7565) | `71c6dc8959f4` | 활성 셀 누름틀 주소와 화면 안내 |
| [#7571](https://github.com/edwardkim/rhwp/pull/7571) | `989e0881e5a2` | 표 생성의 단/쪽 나눔 보존. renderer 무변경·일치 개선 미주장을 비해당 사유로 기록 |
| [#7577](https://github.com/edwardkim/rhwp/pull/7577) | `2e3eeb528925` | 객체 삭제 후 필드 범위·활성 입력 보존 |
| [#7578](https://github.com/edwardkim/rhwp/pull/7578) | `e3268b9bc5ab` | PasswordChar 폼 paint, 변경 전후 이미지 |
| [#7590](https://github.com/edwardkim/rhwp/pull/7590) | `6529a6aaa6e2` | 양식 값 JSON 해석 |
| [#7592](https://github.com/edwardkim/rhwp/pull/7592) | `2e1082980708` | 책갈피 삭제·뒤 필드 원시 위치 |
| [#7595](https://github.com/edwardkim/rhwp/pull/7595) | `520a8ee399f2` | 인라인 객체 삽입·뒤 필드 범위 |
| [#7596](https://github.com/edwardkim/rhwp/pull/7596) | `1e882aae27ed` | 필드 삽입/제거·컨트롤 위치 |
| [#7597](https://github.com/edwardkim/rhwp/pull/7597) | `fe01de7653de` | 필드 끝 뒤 컨트롤 글자 위치. Sweep·한컴·WASM·브라우저 미실행을 명시 |

## 왜 제출 경로에서 빠질 수 있었는가

원문에 반복되는 renderer 파일 무변경·편집 전용·기준 PDF 없음이라는 사유와 다음 지침을 대조했다.
아래 표현은 생략 판단을 유도할 수 있다는 추론이며 기여자의 의도를 단정하지 않는다.

- CONTRIBUTING의 검증 표가 parser/model과 renderer/layout을 파일 경로로 나누고 있었다.
  편집 command와 저장 속성이 실제 조판 결과를 바꾸는 경우의 추가 의무가 표에서 드러나지 않았다.
- 거버넌스의 「선택적 적용」, 보조 도구 안내의 「동작 기준으로 선택」이 실행 여부 선택으로 읽힐 수 있었다.
- PR 템플릿의 「Visual Sweep을 실행했다면」과 미실행 출력 행 삭제 안내는 필수 증적을 선택 사항으로 만들었다.
- 기여자에게 내부 문서를 첨부하지 말라는 안내에 `mydocs/pr/assets/`까지 포함되어 대표 PNG 제출 의무와 충돌했다.
- 기준 PDF 수동 생성 안내에 「PDF 저장 기능」이 있어 Print 출력과 다른 경로를 혼용할 수 있었다.
- 일반 PNG gate는 font 증거가 있으면 미달·누락 페이지까지 예외 처리했다. TSV gate는 이미 엄격했지만 두 경로의 판정이 달랐다.

조회 주소·캐시처럼 실제 조판에 영향을 주지 않는 변경까지 일괄 Sweep 대상으로 바꾸지는 않는다.
비해당 판정에는 변경된 값과 실제 조판 소비 경로의 영향 부재를 설명해야 한다.

## 수정한 제출 계약

1. 실제 조판 영향이면 Native/fresh WASM Sweep·페이지별 TSV를 반드시 실행한다.
   명령과 저장 위치는 [「실루엣 보조값만 빠르게 TSV 산출」](../manual/verification/visual_sweep_guide.md#실루엣-보조값만-빠르게-tsv-산출)에 직접 연결한다.
2. 검증 대상 중 한 페이지라도 **90% 미만** 또는 측정 불가이면 작성자가 자기 branch에서 원인을
   재검토·수정하고 새 head로 재실행한다. **정확히 90%는 통과**한다. 평균·단순 글꼴 추정·CI로 면제하지 않는다.
   사용자의 후속 지시에 따라 올바른 공급으로 해결 불가능한 실제 글꼴 문제는
   [예외 계약](../manual/verification/visual_sweep_guide.md#해결-불가능한-글꼴의-pr-제출-예외)의 근거를 갖춰
   `font_mismatch_exception`으로 PR 제출을 허용한다. 측정 누락·쪽수 불일치·배치 차이는 면제하지 않는다.
3. 쪽수/분할 영향은 전체 문서를 비교한다. 높은 점수의 페이지 선별은 허용하지 않는다.
4. 기준 PDF는 저장 버전에 맞는 한컴 **Print** 경로로 출력한다. MCP는 저장 제품 2024 → engine 2024,
   그 외/미상 → engine 2020을 명시한다. 직접 출력은 실제 제품 버전과 Print 설정을 기록한다.
   편집 동작은 동일하게 편집한 저장본을 인쇄한다. 성공 job·Creator만으로 Print 실행을 입증하지 않는다.
5. TSV·nextest·기타 실행 원 출력은 ignored output에 보존하고 Git에 커밋하지 않는다.
   원본/PDF와 보존용 대표 PNG는 기존 증적 규칙대로 포함하고 PR 본문에 명령·SHA·범위·최저값을 기록한다.

공개 CONTRIBUTING, PR 템플릿, AGENTS/CLAUDE, 시각 거버넌스·Sweep 가이드·review 증적 규칙,
MCP 안내와 contributor skill의 실행 순서/자식 안내를 함께 수정했다. 개별 PR의 기존 미검증을
이 지침 수정만으로 충족 또는 승인으로 바꾸지는 않는다.

## 검증

- `python3 -m unittest scripts.tests.test_visual_sweep -v`: **83 PASS**.
  기존 90.0 PASS/89.99 재검토 경계와 누락 페이지, 추가한 font 증거로 미달·누락을 면제하지 않는 경우,
  높은 페이지와 미달 페이지 혼재를 검사했다.
- 원 출력은 `output/pr-review/semanticist21-20261005/logs/visual-guidance-tests.log`에 보존했다.
- 수정 전 `112f069f6`의 동일 gate 함수에 새 경계 입력을 실행하면 단순 path/hash font 증거만 있는
  미달/누락 모두 `font_mismatch_exception`이었다. 최초 감사 수정 후 두 입력 모두 `re_review_required`로
  판정해 무조건 우회를 검출했다. 후속 사용자 지시의 제한적 예외는 공급 불가·배치/쪽수 일치·대상 쪽
  근거를 추가로 요구하고 source/input/PDF 해시를 고정한다. 단순 증거와 누락 페이지는 계속 보류한다.
- 수정한 안내·보고서 12개의 내부 링크 검사 PASS, front matter 대상 변경 문서 5개 메타데이터 PASS,
  contributor skill 형식 검사 PASS, `git diff --check` PASS. capability의 책임·권위·진입점은 바꾸지 않았다.
- 기존 #7491 Native 캡처 3쌍으로 문서의 `--silhouette-only --png-pair` 명령을 실제 실행해
  TSV 생성과 exit 0을 확인했다(100.00000%, 90.06833%, 100.00000%). 이는 명령 확인용 기존 raster 재사용이며
  새로운 head의 캡처 또는 기존 PDF의 Print 출처 검증으로 승격하지 않는다.
- 전체 장기 문서 메타데이터 검사는 기존 비변경 문서 4개의 필드 누락 16건으로 실패했다.
  이번 변경 문서의 검사는 위처럼 별도로 확인했다. 해당 기존 누락을 지침 수정 범위에서 임의 보정하지 않았다.

### 해결 불가능한 글꼴의 제한적 예외 보완

후속 지시를 반영한 `python3 -m unittest scripts.tests.test_visual_sweep scripts.tests.test_visual_sweep_font_exception -v`는
**89 PASS**다. 해결 불가능한 글꼴의 낮은 점수는 예외 제출, 대상 밖 낮은 쪽·누락 쪽·배치 결함·쪽수 차이·
해결 가능한 글꼴은 재검토로 검사했다. 증거 JSON의 source/입력/PDF 해시 일치와 파일 해시 고정도 확인했다.
단순 비어 있지 않은 글꼴 설명 파일은 예외 증거가 되지 않는다. 원 출력은
`output/pr-review/semanticist21-20261005/logs/visual-guidance-font-exception-tests.log`에만 보존한다.
