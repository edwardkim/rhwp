# planet6897 통합 후보 — PR #8609 반영 전후 검증 계획

## 실제 기준과 보존 범위

기존 통합 브랜치는 `review/integration-20261103-planet6897-open`, 기록 head는
`8d3b6541d1e78e721a3cec41e391033d205e9539`다. 접수한 planet6897 원 PR 39개는
유지하며 새 외부 PR을 추가로 체리픽하지 않는다. PR 생성·push·원격 review·comment·
close·merge는 승인 전 대기한다.

2026-11-05 UTC 00:37에 GitHub ref를 읽어 devel `e9d4a23be4523a97c29ce6f5e407aba6c4cfb26c`
([PR #8609](https://github.com/edwardkim/rhwp/pull/8609))를 확인했다. 이어 fetch 시 실제
`upstream/devel`은 `f529dc92d4`로 전진했다. 검증 기준 `22947f6a90` 이후 변경 중
제품·Studio 코드 32개가 있고 현재 통합 변경과 8개 파일이 겹친다. 문서만 바뀐 리베이스와
구분한다. 이 ref는 적용 전에 전체 SHA와 commit 범위를 다시 고정한다.

겹친 파일은 `src/model/style.rs`, `src/renderer/composer/composition.inc.rs`,
`src/renderer/composer/line_breaking.rs`, `src/renderer/height_measurer.rs`,
`src/renderer/layout.rs`, `src/renderer/layout/paragraph_layout.rs`,
`src/renderer/layout/table_layout/plain_cell_flow.rs`, `src/renderer/mod.rs`다.

## 완료 결과와 종료한 중간 검증

- 기존 제품 `c183a1ac61`: 최소 글꼴 전체 Rust 12,563 PASS·45 SKIP·exit 0,
  Native Skia 4,040 PASS·12 IGNORE와 CI 통합 21개 모듈을 포함한 40개 명령 통과.
- 같은 제품의 Native/fresh WASM 각 80입력·406쪽은 모두 90% 이상이다. 당시 고정
  base의 수치 비교 가능한 396쪽에 0.1%p 초과 하락 0쪽이며 파싱 거부 6쪽·누락 4쪽
  복원은 수치 대조와 구분한다. Native 보호 범위는 원 TSV 2,153행 표시값 하락 0행이다.
- 별도 세로쓰기 보정 `37e5bba300`: 같은 검출 함수의 수정 전 2 FAIL·수정 후 2 PASS,
  기존 대조 5 PASS. Native/fresh WASM 각 6입력·29쪽과 대표 쪽 판독을 완료했다.
  두 문제 입력의 각 5쪽은 양쪽 모두 100.00%, 나머지 27쪽 PNG는 직전 후보와 같다.
- 이 보정 소스의 필수 편람 Native 383쪽은 최저 99.96%·기준 하락 0쪽·fallback 0건으로
  통과했다. 공간 부족에 따른 집계 실패는 canonical resume으로 복구했다. 기대값·metric·
  원 기준을 바꾸지 않았다. 같은 코드의 doctest는 동시 링크만 제한해 8 PASS·3 IGNORE다.
  Rust lint와 workspace 빌드도 통과했다.

새 devel에 실제 코드 변경이 들어와 기존 중간 후보의 남은 전체 검증을 최종 제출 증거로
쓸 수 없다. 실행 중이던 `vertical-axis-final-rust-resume-worker`와
`marker-slot-wasm80-protected-worker`가 소유한 프로세스만 종료했다. 진행 중 명령은
통과나 제품 실패로 집계하지 않고 `superseded`로 구분한다. 완료한 TSV·대표 PNG·원장·
로그·제품은 보존했다. Cargo/Rust 프로세스 0개를 재확인한 뒤 Git 작업을 진행한다.

[세로쓰기 보정의 실제 완료 범위](../pr/assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/README.md)와
[기존 c183 전체 Rust](../pr/assets/planet6897_20261103/rust-validation-c183a1ac61.md)는
최신 리베이스 후보의 완료로 승격하지 않는다.

## 적용 순서와 완료 조건

1. 현재 단계의 완료된 Markdown·TSV·대표 PNG와 이 계획을 일반 commit으로 고정한다.
2. 세로쓰기 보정 `37e5bba300`을 현재 통합 브랜치에 적용한다. 분석 문서의 add/add 충돌은
   현재 브랜치의 더 완전한 완료 기록을 보존하며 제품·새 관계 검사는 빠뜨리지 않는다.
3. 고정한 최신 devel 위에 리베이스한다. 원 기여 39개와 메인터너 보정의 적용 SHA·충돌
   위치를 기록한다. 자동 병합을 기능 보존의 증거로 삼지 않는다.
4. 충돌과 실제 소비 경로를 확인해 기존 기여와 새 base의 기능을 함께 보존한다. 사양·
   원본 저장값·독립 Print에 맞는 공통 조판 결과를 유지하며 한쪽 구현을 통째로 버리지 않는다.
5. 최종 후보에서 필요한 focused test·최소 글꼴 전체 Rust·Native Skia·Rust lint·최적화
   WASM·Studio·실제 출력·편람 gate를 실행한다. 종료된 구 소스의 검증을 반복하지 않으며
   재사용한 자료의 source와 새로 실행한 자료를 구분한다. 충돌 없는 변경의 전쪽 재검사는
   작업지시자의 범위와 실제 소비 경로에 따라 정한다.
6. Native/fresh WASM의 원본·Print·쪽 매핑을 맞추고 90% 미달·누락·쪽수 차이와 변경 영역의
   모양·간격을 확인한다. 새 base 대비 허용 하락폭은 사용자 지정 0.1%p다. 보호 대상
   PR #8389·#8397은 원 TSV의 소수 둘째 자리 표시값이 떨어지면 안 되며 상승은 허용한다.
7. 원 PR별 주장 문서·쪽의 TSV와 review/overlay를 최종 개별 리뷰·본문·후속 댓글 초안에
   연결한다. 중간 분석 JSON·원시 로그는 output에 두고 결과·명령·SHA는 Markdown에 적는다.
8. 최신 head·base·원 PR head·본인 열린 PR·필수 검증·링크를 마지막으로 확인한 뒤
   PR 생성 전 승인 대기 상태로 보고한다. 실제 원격 조치는 승인 뒤 진행한다.
