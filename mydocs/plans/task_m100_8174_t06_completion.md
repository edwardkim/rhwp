# #8174 T06 완료 범위와 종료 조건

2026-10-28. 기준은 T05 완료 `f7ede1d54eb018bea2f49405b3d1cba030e9360d`다.
작업지시자의 「T06 역시 나누지 않고 한 단위로 작업 범위를 잡고 진행해주세요」를 계획·
구현·검증·보고·로컬 커밋까지의 승인으로 기록한다. E01–E05 전체를 완료하고 중간 승인을 반복하지 않는다.

## 한 단위의 범위

| ID | 처리할 내용 | 완료 조건 |
| --- | --- | --- |
| E01 | S06에서 좁힌 chacha20 0.10.1 → 0.10.2 갱신, root/독립 소비 graph와 공식 archive 검증 | 불필요한 다른 의존성 갱신 없이 lock·feature·checksum 대조. 기존/새 seal·password/PQ trailer의 양방향 호환, 오암호·키 불일치·변조 거절을 Native/WASM에서 실제 실행 |
| E02 | mixed progressive JPEG 6건의 최초 실패 원인을 공통 decoder에서 보정 | cjpeg/djpeg의 독립 입력·픽셀·성분 sampling 기하와 대조. 기존 52개·같은 경계의 baseline/작은 입력 보존, 요청 예산의 정확한 경계/부족·sticky 실패·정상 복구. 기존 손상 입력 guard를 제거하지 않음 |
| E03 | JPEG가 들어간 실제 PDF의 Native/CLI·최적화 fresh WASM/Worker/Studio 소비 | 원본 그림의 크기·내용·alpha·clip과 HWP/HWPX 저장·재열기 보존. 실패 후보의 기존 편집/draft/recents/저장 핸들 보존과 정상 재생성. codec 감사 모듈 성공으로 제품 실행을 대신하지 않음 |
| E04 | 필수 lint/test·변경 영향 전쪽·비용·패키지 소비 확인 | 공용 target 순차 Cargo, 전체 nextest·Native Skia·Native/WASM/workspace lint/build·TypeScript/Node·실제 브라우저. 영향 입력/독립 기준의 전쪽 Native/fresh WASM TSV와 compare/overlay/review 확대 판독. 현재 제품과 source hash 고정 |
| E05 | 가이드·의존성/codec patch 고지·TODO·완료 보고·로컬 커밋 | E01–E04 필수 잔여 0건. 기존 T04/T05 출력/보안 경계를 보존하고 실측한 비용·지원 한계를 기록. 최종 전체 이슈/PR 준비와 구분 |

## 선행 이력과 구현 근거

[S06](../working/task_m100_8174_v2_stage6.md)의 52개 입력과 6개 거절, 의존성 archive/source 조사,
[S05](../working/task_m100_8174_v2_stage5.md)의 기존 포장·소비와 yank 경고를 출발점으로 한다.
원 PDF 도입 PR #7913의 실제 통합은 `4662c069e557e928ed7968d061121fde90b1f027`이며
이때 들어온 `vendor/rhwp-zune-jpeg/src/misc.rs`의 공통 component setup을 현재와 대조한다.
후속 요청 예약 패치는 유지한다. #8174는 현재 open·assignee edwardkim이며 기존 브랜치를 이어 쓴다.

현재 최초 실패 가설은 모든 성분의 최종 Hmax/Vmax를 알기 전에 앞 성분의 stride/MCU 크기를
확정한다는 것이다. 이를 실제 입력의 SOF sampling·MCU/coefficients·IDCT 소비 위치와 대조해
확정한 뒤 수정한다. `Invalid slice` guard의 제거·clamp·문서별 예외나 다른 backend로 우회해
수용 여부만 맞추지 않는다. 공통 기하의 수정은 baseline에도 영향을 줄 수 있어 같은 입력군을 함께 대조한다.

chacha20은 공식 crates.io metadata/archive와 upstream 보정
[PR #580](https://github.com/RustCrypto/stream-ciphers/pull/580)를 다시 확인한다.
선택하지 않은 RNG/legacy 경로에 대한 정적 설명을 소비 graph 전체의 안전성 보증으로 바꾸지 않는다.
새 테스트는 기존 `tests/cases/` 자동 수집 경로에 둔다. 렌더링 회귀/fixture 추가는 독립 기준과
Native/fresh WASM 전쪽 90% 이상·직접 판독 이후에만 수행하며 baseline·한도를 완화하지 않는다.

## 실행·증적·비범위

증적은 ignored `output/pr-review/issue8174/t06-completion-20261028/`에 보존한다.
기준 Native/WASM 제품·현재 source·실제 archive/graph·입력/독립 픽셀·최종 제품·명령/exit·
전쪽 TSV/대표 PNG와 비용을 서로 연결한다. 비공개 문서는 계속 로컬 검증만 한다.

T07의 14개 전체 최종 게시 인계와 T08의 기존 누적 unit 정책 이관·최종 통합 성능,
T09의 최신 devel 보존/PR 제출은 별도 잔여다. 이번에 바뀐 의존성과 decoder의 실제 소비는
T06 안에서 완료하며 그쪽으로 넘기지 않는다. 원격 push·PR/comment·registry 게시·토큰 설정·
태그는 이번 범위에 포함하지 않는다. 전체 heap/RSS·동기 main thread 선점의 비보장도 유지한다.

## 완료 판정

2026-10-28. E01–E05를 한 단위로 완료했고 필수 잔여는 0건이다.
[T06 완료 보고](../working/task_m100_8174_t06_completion.md)에 좁힌 의존성·양방향 암호 호환,
공통 JPEG 기하·예산/픽셀, 실제 Native/Worker/Studio·독립 Rust/WASM 소비·전체 회귀·전쪽
TSV/직접 판독·비용·source/제품 대조를 기록했다. 추가 vendor 단독 Clippy의 기존 진단 128건과
이번 신규 진단 0건도 필수 root lint 결과와 구분했다. T07–T09·원격 PR/registry/태그는 별도다.
