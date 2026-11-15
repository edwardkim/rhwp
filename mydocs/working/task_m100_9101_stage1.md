# 이슈 #9101: 기여자 공통 시각 검사 준비

Issue: https://github.com/edwardkim/rhwp/issues/9101

## 범위와 선행 분석

정본 보호 TSV source `4e0b20d2922953ee7ab9a1716ac819b921fe484b`의 18개 원본을 조사한다.
정본은 비공개 글꼴을 포함한 통합 환경의 보호 기준이므로 일반 기여자의 재현 가능성을 별도로 확인한다.
기존 `visual_sweep_handbook.py`의 해시 검증·웹폰트 공급·전쪽 판정과
`scripts/visual_sweep_resume.py`의 공식 checkpoint를 재사용한다. 측정 알고리즘과 기존 정본은 변경하지 않는다.

공개 시스템 글꼴과 등록 웹폰트만 노출한 Linux 환경과 Windows OS 기본 글꼴 19개를 추가한 환경을 구분한다.
OS 글꼴은 원 호스트에서 해시를 확인한 실행용 사본이며 Git·배포물에 포함하지 않는다.
Windows/macOS에서 직접 실행한 결과로 표현하지 않는다.

## 계획

1. 원본·기준 PDF SHA와 페이지 수를 고정하고 두 backend의 전쪽 TSV를 산출한다.
2. 높은 점수여도 실제 글자 모양이 다른 입력은 공통 대상에 넣지 않는다.
3. 공개 환경에서 재현되는 대상만 데이터 manifest로 등록하고 별도 공통 gate를 추가한다.
4. 누락·변조·쪽수 불일치·backend 혼용·점수 하락·실행 실패가 통과하지 않는지 검사한다.
5. 최종 자동 실행 결과와 대표 review·overlay를 보존하고 기여자·로컬 검증 지침에 연결한다.

## 적용 경로

기본 경로: collaborator_self_merge.
보조 경로: intake_and_review, local_validation, visual_fixture_evidence, post_merge.
PR 생성 전 본인 열린 PR과 시뮬레이션하며 reviewer 지정 PR은 제외한다.
현재 renderer·Rust·Studio 실행 코드는 바꾸지 않는다. Python 검사와 문서 검증을 수행한다.

## 진행 중 관측

- 공개 웹폰트의 Helvetica 대체는 body-list에서 raw 100.00%여도 굵기·장평이 다르다.
  이 입력을 공개 공통 검사로 수용하지 않는다.
- Arial 기본 글꼴을 추가한 control-styles는 두 backend 100.00%다. OS 의존 결과로 구분한다.
- 라틴·좁은 셀·그림 자르기·그룹 도형 4개는 공개 Native에서 100.00%이며 직접 판독했다.
- 18개 전체 검증과 공개 fresh WASM은 진행 중이다. 완료 결과는 아래에 추가한다.

## 1단계 구현·검증 결과

공개 4개 원본의 Native/fresh WASM 8건 자동 실행을 완료해 모든 쪽 raw 100.00%를 확인했다.
동일 PNG로 review·overlay를 생성해 글자 모양·줄바꿈·자르기·그룹 윤곽을 직접 판독했다.
공통 검사 CLI·고정 manifest·실패 계약 검사를 추가하고 기여자·로컬 검증·시각 검증 정본에 연결했다.
기존 CI의 `test_visual_sweep*.py` 자동 수집에 포함되므로 개별 CI 호출은 추가하지 않았다.

- `venv/bin/python -m unittest discover -s scripts/tests -p 'test_visual_sweep*.py'`: 267건 통과.
- 공통 CLI 실제 실행: 8/8 통과. 공식 checkpoint를 사용한 재개도 8/8 통과했다.
- 변경한 매뉴얼 metadata: 오류 0건. 전체 metadata 검사는 기존 5개 문서의 누락 20건을 보고했으며,
  해당 파일이 source `4e0b20d`와 동일함을 확인했다. 이번 수정의 오류로 분류하지 않는다.
- `git diff --check`: 통과. 상대 링크 검사는 진행 중이며 다음 단계에서 종료 결과를 기록한다.
- Rust·Studio·렌더러·기존 정본·기준 PDF·CI 실행 정의는 변경하지 않았다.

증적은 `mydocs/pr/assets/issue_9101/`에 보존한다. 18개 환경 조사 중 Native는 18개 전쪽을 완료했고
WASM은 16개 완료, 나머지 편람 HWPX·sample10은 진행 중이다. 그 결과를 공통 4개 통과와 구분한다.
