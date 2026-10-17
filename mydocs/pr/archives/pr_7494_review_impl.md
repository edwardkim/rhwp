# PR #7494 메인터너 통합·보정 실행 기록

## 처리 경로와 승인

사용자가 #7494 체리픽 검토·필요한 메인터너 보정과 #7751 동시 해결을 요청했다.
PR 생성·CI 모니터링·같은 PR trailing review/오늘할일·정상 merge·후속 처리는 승인된 범위다.
source fork에는 push하거나 이력을 다시 쓰지 않고 `upstream/devel` 기반 통합 branch를 사용한다.

- branch: `review/ldj-diquest-7494-20261016`
- 원 source: `a3f06bedb3bbd5402082349b955fdf1312a2f4bf`
- 통합 PR: [#7964](https://github.com/edwardkim/rhwp/pull/7964), code candidate: `e8d7cbdeb5901e9b345abcc13e6670666a6a4057`
- 최종 검토 판정·실제 입력/검증: [개별 review](pr_7494_review.md)

## 적용 커밋과 출처

원 기여의 기능6커밋은 author를 보존했다. 원 fork의 devel merge는 적용하지 않았다.
84847e1/488fa93에서 workflow를 추가·제거하던 부분만 사용자 요청으로 제외했으며
나머지 source/test/문서는 보존했다. CI는 기존 Native PDF 진입점에서 검사한다.

| 통합 SHA | 원 PR SHA | 작성자 | 변경 |
| --- | --- | --- | --- |
| `73d8446df` | `6de13e368` | ldj-diquest | Preserve adjusted images in direct PDF exports |
| `498d55958` | `84847e1a7` | ldj-diquest | fix(pdf): 일반 그림 보정과 미지원 워터마크 경계 보존 |
| `1bbd67951` | `427c7e9b7` | ldj-diquest | Prevent adjusted image colors from saturating in the current Skia replay |
| `c7e279c0f` | `785a73624` | ldj-diquest | Keep adjusted native images consistent with the existing SVG color semantics |
| `2549886a7` | `25fd73636` | ldj-diquest | Make the direct PDF color support and remaining limits reviewable |
| `33c699ef7` | `488fa9325` | ldj-diquest | test(pdf): 기존 Native PDF 진입점에 픽셀 검사 연결 |
| `64db01744` | 메인터너 보정·증적 | jangster77 | fix(pdf): 이미지 필터 순서와 미지원 흑백 조합 보정 |
| `f944bcdc3` | 메인터너 보정·증적 | jangster77 | test(pdf): 쪽 배경의 이산 흑백 보정 경계도 검사 |
| `d8e8e93e0` | 메인터너 보정·증적 | jangster77 | docs: #7751 편람 전쪽 보정 기준과 조사 범위 기록 |
| `eedb22fa5` | 메인터너 보정·증적 | jangster77 | fix(layout): 이어받기 행 간격과 통째 표 종료 여백 보정 |
| `063ee3c2b` | 메인터너 보정·증적 | jangster77 | fix(render): 한컴 그림 색조와 묶음 글상자 반각 흐름 보정 |
| `5f743d80c` | 메인터너 보정·증적 | jangster77 | docs: 불필요한 CI 테스트 호출 추가를 막는 지침 보완 |
| `25f284c5e` | 메인터너 보정·증적 | jangster77 | test: 편람 완료 페이지와 한컴 그림 색조 회귀 보강 |
| `79e67763f` | 메인터너 보정·증적 | jangster77 | docs: KoPub 주의표 폭의 독립 근거와 보정 계획 기록 |
| `d4990c437` | 메인터너 보정·증적 | jangster77 | fix(font): KoPub 주의표의 원본 전진폭 보존 |
| `0453474dd` | 메인터너 보정·증적 | jangster77 | docs: 편람375쪽 서식 괘선 원점 보정 계획 기록 |
| `f7a696f87` | 메인터너 보정·증적 | jangster77 | fix(검증): raw 실루엣으로 90% gate와 98% px 기준 통일 |
| `aacb17280` | 메인터너 보정·증적 | jangster77 | fix(조판): 저장 줄 정렬과 글앞 표의 바깥여백 원점 공유 |
| `3f2fc0cfd` | 메인터너 보정·증적 | jangster77 | fix(변환): HWPX 그림 밝기와 명암의 중복 교환 제거 |
| `05295b3d2` | 메인터너 보정·증적 | jangster77 | fix(렌더): WMF 색 처리와 지연 로딩 바이트를 공유 |
| `257aba27b` | 메인터너 보정·증적 | jangster77 | test(렌더): 편람 완료 경계와 WMF 지연 로딩 보호 |
| `b5ab151da` | 메인터너 보정·증적 | jangster77 | fix(paint): 그림 payload 참조 정리와 독립 기대값 구분 |
| `8c5402628` | 메인터너 보정·증적 | jangster77 | fix(canvas): WMF 원본 픽셀 크기와 저장 crop 공유 |
| `d1c1f8d83` | 메인터너 보정·증적 | jangster77 | fix(layout): 이동한 문단의 저장 흐름 끝 재사용 차단 |
| `077cc3684` | 메인터너 보정·증적 | jangster77 | fix(render): 일반 회색조와 흑백 색조의 Print 픽셀 보정 |
| `519a3dbdf` | 메인터너 보정·증적 | jangster77 | fix(print): 논리 조판과 물리 인쇄 종이 경계 분리 |
| `ad96d00d5` | 메인터너 보정·증적 | jangster77 | docs: Print 전쪽 재검토와 raster 바탕쪽 원인 기록 |
| `bd5207aa0` | 메인터너 보정·증적 | jangster77 | fix(paint): 바탕쪽 raster 그림에 물리 Print 참조 프레임 적용 |
| `8291d5cdd` | 메인터너 보정·증적 | jangster77 | test(render): Print 종이와 바탕쪽 프레임 경계 보호 |
| `843164b5a` | 메인터너 보정·증적 | jangster77 | fix(render): 일반 그림 색조를 DOM 지연 로딩과 공유 |
| `465fd10fd` | 메인터너 보정·증적 | jangster77 | docs: 최신 devel 리베이스와 최종 검증 계획 기록 |
| `5060c4bb2` | 메인터너 보정·증적 | jangster77 | docs: 전쪽·전체 회귀 결과와 묶음 화살표 재검토 근거 보존 |
| `cef058e45` | 메인터너 보정·증적 | jangster77 | fix: 셀 도형의 문단 기준 가로 원점에 문단 여백 반영 |
| `542a8bcbf` | 메인터너 보정·증적 | jangster77 | docs: 셀 도형 원점 보정의 Native 전후 검사 기록 |
| `2ea129901` | 메인터너 보정·증적 | jangster77 | test: 독립 Print 기준 셀 도형과 묶음 화살표 위치 보호 |

| `e8d7cbdeb` | 메인터너 보정·증적 | jangster77 | docs: 편람 셀 도형의 전쪽 시각·최종 검증 증거 보존 |

## 실행 순서와 검증 연결

1. 원 PR metadata·첫 기여·reviewer를 확인하고 기능6커밋을 통합했다.
2. 직접 PDF guard·정규화 offset·효과 순서·alpha 계약을 독립 RGB로 확인했다.
3. 편람 원본·독립 Print를 고정하고 이어받기 높이/간격과 통째 표 후속 여백을 공통 흐름에 연결했다.
4. 실제 한컴 저장·Print 색조 대조군5개를 만들고 parser/serializer 필드 순서와 공통 raster 색조를 보정했다.
5. 반각 문자와 KoPub 주의표 원본 메트릭, 저장 줄 pen·표 바깥여백을 공통 소비 경로에 반영했다.
6. WMF payload 신원·source crop·실제Canvas 호출을 수정 전후로 검증했다.
7. 전체 회귀에서 실제 hwpspec 겹침 증가를 찾아 저장 시작이 달라진 축의 재사용을 막았다.
   기존 #7772를 이유로 새 겹침을 이관하지 않았다. 문서 전체178/181쪽 차이는 별도 범위로 남긴다.
8. 기존 base도90% 미만인 복학원서1검사와 주택임대차2검사는 #7756/#7955에 원문·증적을 보존했다.
   정상 검사와 공용 입력·PDF는 유지했다. #7445의 본문 표·subissue 연결도 실제 확인했다.
9. 논리 조판/물리 Print 종이와 명시적 바탕쪽 raster 프레임을 공통 PagePaintContext로 분리했다.
10. 실제 Studio DOM의 CSS 승산 결함을 발견해 공통 resolved PNG와 lazy source key로 보정했다.
    550Canvas·550DOM 실제 screenshot 패치를 각각 독립 Print와 비교했다.
11. #7953으로 devel이 바뀌자 이전 전체 nextest를 명시적으로 중단하고30커밋을 충돌 없이 리베이스했다.
    이전 실행을 전체 PASS로 세지 않고 새 default feature에서 Native/fresh WASM을 다시 빌드·검증했다.
12. raw 99.6%에도 p289 그룹 화살표가 26px 왼쪽임을 추가 확대 판독으로 발견해 PR을 보류했다.
    셀 문단의 가로 기준에 여백을 반영해 그룹과 일반 사각형의 원점을 같이 보정했고 독립 Print와 재대조했다.
    전 383쪽 재실행에서 135·289쪽만 개선됐고 양쪽 나머지 381개 PNG는 byte 동일했다. 정식 검사는 before 4PASS/2FAIL → after 6PASS였다.
13. [Full CI](https://github.com/edwardkim/rhwp/actions/runs/38993598237)의 정확한 code head 성공 후 이 archive review·구현 기록과 오늘할일을 같은 PR에만 추가한다.
    source/test/fixture/workflow는 trailing commit에 섞지 않는다. 최신 devel의 오늘할일 기록은 최종 merge tree에서 보존한다.
14. trailing head의 CI·required aggregate·MERGEABLE/CLEAN·head 일치를 확인한 뒤 정상 merge한다.
15. merge SHA·devel 반영·duration 갱신 결과를 확인하고 #7751과 원 PR에 실제 결과·이미지를 게시한다.
    #7445 표의 완료 취소선과 closed subissue 연결 해제를 확인한다. #7756/#7955는 OPEN을 유지한다.
16. clean한 이번 작업 worktree2개·임시 upstream head·local 검토 branch를 안전 조건에 따라 제거한다.
    핵심 TSV/log/provenance를 영구 경로에 해시 확인 후 보존하고 root output 전체를 정리한다.
    기본 작업공간·공유 target/pr-review·다른 작업 자료·source fork·backup/stash는 보존한다.

개별 단계의 과거 SHA·실패·기각 실험은 `mydocs/working/task_m100_7751_stage1.md`부터
`task_m100_7751_stage15.md`에 보존한다. 리베이스 뒤 SHA를 과거 실행의 생산 SHA로 바꾸지 않는다.

## 보존·정리 경계와 되돌리기

- 실제 신규 source는1000줄 미만이고 정식 회귀는 `tests/cases/` 또는 기존 PDF 진입점이다.
- source/test 보정 뒤 manifest를 prepare하고 base 고정 policy 검사로 누락을 확인했다.
  파생 suite/manifest는 Git에 넣지 않는다. 기존 정확한 계약을 점수나 CI 목적으로 완화하지 않는다.
- rejected 폰트 축소와 종이 일괄 좌표 변환은 제품 코드에 포함하지 않았다.
- `tests/golden_svg`·원본 HWP/PDF·source fork와 사용자 backup/stash는 삭제하지 않는다.
- rollback이 필요하면 통합 merge를 대상으로 별도 revert PR을 준비한다. contributor fork를 reset하거나
  원 기여의 일부만 무근거로 되돌리지 않는다. 현재 head의 실제 결함과 과거 미달 이관을 구분한다.
- 종료 증적의 영구 경로는 `/home/tsjang/rhwp-review-evidence/pr7494-20261016/`이다.
  실제 cleanup 목록·전후 용량·hash 검증은 그 경로의 postmerge 원장으로 확인한다.
