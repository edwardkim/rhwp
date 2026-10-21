---
kind: snapshot
status: active
canonical: mydocs/working/task_m100_7445_rebase_repair_stage26.md
last_verified: 2026-10-21
---

# #7445 보정26 — 머리말 그림 토글의 독립 줄 점유 검증

Issue: #7445, 관련 #3080. 부모9ff65a9e07, 고정base2e3b4f880bd11d35398e6ea0100846611f13a72d 위 로컬 검사 보정이다. 제품 로직은 바꾸지 않았으며 PR 준비 완료가 아니다.

원PR#3081과 통합#3218/3bc17ee95577623f119133248f93b8f37a6c4526의 본문·논의·diff를924에서 확인했다. 원 계약은 머리말/꼬리말 setter도 본문과 같은 TAC 토글 마이그레이션을 호출해 줄 점유를 갱신하는 것이다. 당시 검사의 본체 높이 단독 기대값과 현재 본체+여백 공통 점유가 달랐다. 곧바로3016을 복사하지 않고 같은 편집 저장본의 독립 한컴2020 Print151쪽과 재저장 HWPX를 만들었다.

독립 재저장본은 그림2448HU와 위/아래284HU씩, 줄/text3016HU·기준선2564HU를 보존했다. 생산(`inline_picture_occupied_size_hu`)→공개 setter migration의 LineSeg→측정/최종 머리말 표시가 같은 점유를 쓴다. 이 단계에서 마이그레이션을 본체2448로 되돌리거나 그림 크기·여백·원본을 바꾸지 않았다.

같은 수정 HWP의 Native/fresh WASM 전151쪽은 각각 최저93.14319%/93.14843%(50쪽), 각151/151·미달/누락0이다. 각쪽 TSV와2·50·151쪽을 직접 판독한 대표PNG를 보존한다. 원본 HWP3와 새 수정 HWP5의 입력SHA·생성 recipe·별도 독립 PDF를 구분한다. 기존50쪽 Print의 면 표시와 미세 자형/선 차이는 남아 있으므로 모든 시각 차이를 해결했다고 표현하지 않는다.

기존 정규 검사1개 안에서 false→true 경로·실제 source 그림/문단 소유를 유지했다. line/text 전체 점유, 독립85% 기준선, 본체·여백 및 머리말/본문 구역·문단 텍스트 보존을 추가했다.980의 기존 기대값은 실제1FAIL, 최종981은1PASS(0.307초)다. manifest/prepare→fmt→prepare→자동 nextest, manifest/check·unit-tier/check·fmt/diff 모두 exit0이다. 이전 전체 회귀의 알려진 실패 한 건을 독립 근거로 교정한 것이며 새 전체 nextest 통과가 아니다.

[입력·PDF·producer SHA와 정규 검사, 전151쪽 TSV·review/overlay](../pr/assets/issue7445/completed-20261009/stage26-20261021/README.md)

#8064 Native 품질 하한은973에서 현재 코드로 다시 확인했고 PrEP1과 법령2·13·36쪽의0.1%p 이상 하락은 남는다. 법령 파일은 확장자hwpx지만 실제HWP5/한컴2018 저장본이다.977의 별도 명시TTF 진단은36쪽에서0.02%p 개선에 그쳐 해법으로 채택하지 않았다. 진단 환경으로 수용 조건을 바꾸지 않는다. 과학 기준선·품질 하한 보정, 최신 전체 corpus/base/head TSV·nextest/필수gate·actual36·PR 직전upstream 확인은 계속 남는다. PR/push·외부 게시에는 사용자 승인이 필요하다.
