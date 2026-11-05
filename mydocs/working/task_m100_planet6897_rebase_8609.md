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

## 세로쓰기 보정 적용

`37e5bba300`의 제품·새 관계 검사 6파일은 현재 브랜치의 `b9a63a95ca`에 바이트 단위로
같게 적용됐다. 충돌은 분석 문서의 add/add 한 건이었고, 기존 브랜치의 더 완전한 결과
기록을 보존했다. 충돌 정리 명령 오류로 남은 문서 표시를 바로 제거했으며 제품 소스나
기대값을 추가 변경하지 않았다. 다음 일반 commit에 문서 보정을 고정한다.


## 리베이스 적용 결과와 최종 실행 계획

2026-11-05 UTC 00:51에 `f529dc92d4d172bf32390d7749743a45d9e41bfc` 위로
168개 커밋의 리베이스를 완료했다. 결과 head는
`dda217dba06e33eb0749a24872a60e99e9e63e8a`이며 충돌은 0건이다.
Git range-diff 168행이 모두 `=`로 대응하고 비동일 패치 행은 0개다.
세로쓰기 공통 수정은 새 SHA `4a0af7e867`에 보존됐다. 앞 절의 `b9a63a95ca`는
리베이스 전 SHA이며 두 SHA를 별개의 수정으로 세지 않는다.

최종 실행은 위 head의 Rust·Cargo·내장 글꼴·빌드 입력 해시를 고정해 수행한다.
Cargo는 기본 작업공간의 `target/pr-review`에서 빌드 잡 8개로 순차 실행한다.
회귀 실행에는 공개 글꼴 5개만 노출하는 기존 격리 wrapper를 적용한다.
manifest·정책·세 Clippy·workspace build·관계형 집중 검사·전체 nextest·Native Skia
필수 묶음·doctest와 최적화 WASM 제품을 확인한다. doctest만 링크 피크를 줄이도록
실행 스레드를 1개로 제한하며 전체 nextest는 12개로 유지한다.

작업지시자의 충돌 없는 리베이스 후 전쪽 비교 생략 지시에 따라 기존 80문서 TSV를
최신 소스의 신규 산출물로 바꾸어 표시하지 않는다. 새 base의 실제 변경 소비 경로인
표 높이 제약, 구역별 탭, 머리말·꼬리말 번호와 이미지, 장식선·테두리 및 세로쓰기
공통 경로를 선별 검증한다. 제출 전 편람 gate는 별도로 최종 제품에서 실행한다.
보호 PR의 완료·미완료 실행도 구분하며 부분 결과를 전쪽 완료로 집계하지 않는다.
실행 계획과 원시 로그는 `output/pr-review/planet6897-20261103/`에 보존하고,
공식 결과·명령·SHA·제한은 Markdown으로 요약한다. 원격 변경은 승인 전 수행하지 않는다.


## 최신 Native 제품과 주장 쪽의 실제 확인

최신 source에서 Native 제품을 고정하고 현재 보정의 경계 10입력·43쪽을 완료했다.
세로쓰기 새 관계 2개와 기존 대조 5개·개요·저장 쪽 경계의 집중 검사도 통과했다.
Native 전쪽 최저 raw는 94.92448%, 직전 c183/37e 후보 대비 0.1%p 초과 하락은 0쪽이다.
표시용 두 자리 TSV만 비교하지 않고 실제 manifest의 다섯 자리 raw와 표시 일치까지 확인했다.

기존 직접 판독에서 차이를 기록했던 PR #8512·#8535도 원래 39개 범위 안의 확인 대상에
추가했다. PR #8512의 해당 쪽 Native PNG는 기존 4b 자료의 출처 해시와 바이트가 같고,
PR #8535는 c183의 실제 원시 PNG와 바이트가 같다. 점수 통과로 기존 차이를 삭제하거나
새 개선 성과로 바꾸지 않는다. PR #8535의 입력 XML은 각주 자동 번호를 본문 뒤에 두며
독립 PDF는 뒤에, Native는 앞에 표시한다. 원 PR ee3fd560의 제품 변경은 scaffold 표
속성 6줄이고 번호 전체 일치를 해결한다는 주장이 아니다. 표·각주 영역의 비겹침과
기존 번호 차이를 개별 리뷰에서 구분한다.

최신 Native의 fit/초과 높이 5쪽과 두 기존 차이의 review·별도 overlay를 실제로 열어
관측을 기록했다. 새 base 자체와 fresh WASM 대조·최종 회귀가 끝나기 전에는 전체
수용 판정으로 확대하지 않는다. 중간 JSON·원시 로그는 output에 보존하고 제출 자료는
원본 TSV·대표 PNG·결과 Markdown으로 제한한다.
