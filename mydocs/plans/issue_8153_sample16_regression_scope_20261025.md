# #8153: 별도 PR에서 처리하는 sample16 회귀의 범위 분리

## 작업 근거

2026-10-25 사용자는 현재 브랜치의 `hwp3-sample16` 관련 회귀 테스트를 모두 제거하도록
지시했다. 별도 작업은 이슈 [#8160](https://github.com/edwardkim/rhwp/issues/8160),
열린 [PR #8188](https://github.com/edwardkim/rhwp/pull/8188)이다. 읽기 전용으로 확인한
head는 `021c486d1eaee17920a2b3e82ae896115b847669`이며 본문의 세 원본 전쪽 raw100과
중요 영역 회귀 범위도 확인했다. 이 브랜치에서 해당 PR의 구현을 체리픽하거나 수정하지 않는다.

이 제거는 사용자가 정한 작업 범위의 분리다. sample16의 현재 출력을 90% 미달이라고
판정했거나 다른 개인 이슈로 자동 이관한 것이 아니다. 편람 두 원본의 raw100 목표와
검증 기준도 변경하지 않는다.

## 처리 범위

- 원본 HWP3, HWP5/HWPX 변환본, 버전별·암호 변형본을 직접 입력으로 쓰는 전용 검사와
  그 검사만 사용하는 helper를 제거한다. 함수 호출과 상수 참조까지 추적한다.
- 여러 원본을 검사하는 배열에서는 sample16 항목만 제거한다. 다른 원본의 검사와
  독립 기대값은 유지하고 고정 배열 길이·수집 목록도 실제 구성에 맞춘다.
- 공용 corpus·baseline·합성 raster 원장 및 Studio 암호 E2E에서도 해당 입력을 실행
  대상으로 다시 수집하지 않도록 정리한다. 다른 문서의 암호·파서·저장·렌더 검사는 유지한다.
- 원본 HWP/HWPX/PDF, 제품의 parser·layout·renderer 구현, 다른 PR head는 보존한다.
  source 내부의 sample16 전용 unit test만 제거하며 혼합 unit test의 다른 원본 검사는 유지한다.
- 같은 날 사용자의 추가 지시에 따라 보관용 `.txt` 파일과 임시 코드 백업도 제거한다.
  sample16 회귀의 원 코드를 별도 파일로 보관하지 않는다. 실행 로그와 제거 범위만 기록한다.

## 보관 취소 반영

사용자가 "sample16 회귀 테스트는 보관하지 말고 그냥 제거"하도록 명시했다. 이에 따라
보관 디렉터리와 output의 코드 사본을 삭제했고, 활성 검사 제거는 유지한다. PR에는
보관용 코드가 포함되지 않는다. 원본 HWP/HWPX/PDF와 다른 문서의 검사는 유지한다.

## 검증

1. 실제 문자열과 함수·상수 의존성, 공용 배열·corpus 참조를 검사하고 삭제 범위를 대조한다.
2. 별도 validation worktree에서 suite를 준비하고 자동 수집·manifest·정책 검사를 수행한다.
   파생 `tests/generated/`와 manifest는 source commit에 포함하지 않는다.
3. fmt·관련 focused test·source unit tier 및 필수 lint를 실행하고, 미실행 검증을 통과로
   기록하지 않는다. Cargo/Rust 검증은 기존 `target/pr-review`에서 순차 실행한다.
4. sample16 입력과 PDF의 Git blob이 바뀌지 않았는지 확인한다. 새 sample16 기대값이나
   sweep을 만들지 않는다. 필요 회귀의 복원과 시각 검증은 별도 작업에서 처리한다.
