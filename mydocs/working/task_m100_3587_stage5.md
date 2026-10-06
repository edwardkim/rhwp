# #3587 Stage 5 — B2 문단 블록 반복 삽입

- 일자: 2026-09-12
- 승인: 메인테이너 「다음 절차를 진행하는 것을 승인합니다」.
- 근거: [B 계획](../plans/task_m100_3587_impl_b.md), [B1 검증](task_m100_3587_stage4.md).
- 상태: **B2 반복 삽입 핵심 경로 구현·집중 검증 완료. B 전체 완료 또는 PR 준비 완료가 아니다.**

## 구현 범위

`repeat_paragraph_block_native`가 B1 사전검사를 통과한 본문 문단 블록을 반복 삽입한다.
전체 원문 ID 예약은 요청당 한 번, 내부 참조 대응표는 사본마다 별도로 만든다.
원형은 staging 동안 불변 참조하므로 별도의 snapshot 복제로 메모리를 중복 사용하지 않는다.
사본·결과 경로·표 재조판 출처를 준비하고 회복 가능한 실패를 검사한 후 한 번만 삽입한다.
기존 문단과 표 Box 신원, clipboard, 외부 batch 상태는 유지한다.
새 문단의 연결에는 기존 구역 vpos 재계산·재구성 경로를 사용한다. 새 조판 정책은 추가하지 않는다.

전체 원문의 ID 예약에서 모델에 존재하는 OLE chart fallback 소유 트리의 예약 누락도 보완한다.
이는 B에서 OLE 원형 복제를 허용한다는 의미가 아니다. 원형 밖 ID 충돌 방지 범위다.
OOM abort·프로세스 종료까지 rollback을 보장하지 않는다.

## 검증 및 남은 범위

집중 테스트·native Clippy 결과는 아래 동일 제품/테스트 SHA 검증 기록을 따른다.
B3 HWP/HWPX 저장·재열기, 실물 배치 판정, 1/10/100회 비용 실측,
B 종료 통합 게이트와 C 내용 채우기 API, 후속 Gym 시나리오는 아직 완료하지 않았다.
원격 push·PR·댓글은 이번 승인 범위가 아니다.

## 집중 계약 테스트

`tests/cases/issue_3587_paragraph_block_repeat.rs`의 10건을 기존 A/B1 64건과 함께 실행한다.

1. 앞/원형 시작/원형 끝/본문 끝 삽입 × 1/2/5회. 기존 문단과 빈 문단 수 및 원형 이동 주소 보존.
2. count=0 및 내부 삽입·overflow·예산 초과 거부 시 원문·clipboard·이벤트 불변.
3. 문단 간 필드 시작/끝 참조가 연속 요청에서도 사본별로 연결되고 이름은 보존됨.
4. 중첩 글상자 경로 대응과 사본 내부 내용의 독립성.
5. 두 실물 블록 반복에서 원형 표 Box 신원·전체 표 데이터 보존, 사본별 common/raw ID 일치·독립성.
6. 외부 batch 안에서 성공 이벤트 1건, clipboard 보존 및 batch 종료 후 본문 구성.
7. 내부 연결선 대상이 원형이나 다른 사본이 아니라 같은 사본의 도형 ID를 참조.
8. 원형 밖 OLE fallback 소유 ID와 신규 사각형 ID의 충돌 방지.
9. 반복 성공·실패 뒤에도 기존 snapshot 복원이 가능하고 snapshot 번호를 소비하지 않음.
10. 공개 셀 편집 API로 원형 편집 → 복제 → 한 사본 편집을 수행해 원형·다른 사본·clipboard 보존.

5번 실물 원형은 `hwp_table_test.hwp`의 pi=3 및 `labnote-001.hwp`의 pi=12 한 문단이다.
표 프레임/내용의 구조 보존을 검사하지만, 호스트 문단의 y/페이지가 불변이라는 주장은 하지 않는다.
10번은 재조판 출처 승계 경로를 실행한다. private 출처 set의 모든 중첩 항목 및 최종 시각 결과를
직접 판정한 테스트는 아니다. 전체 B2 지원 조합·B3 저장 결과 검증을 대신하지 않는다.

## 초기 검증에서 발견한 테스트 오류

- `333ad5c149`: OLE fallback 예약 테스트에서 `ShapeObject::Ole`의 Box를 빠뜨려 E0308 컴파일 실패.
  `91b20b4236`에서 생성 타입을 바로잡았다. 이 시점 실행은 **73 PASS / 1 FAIL**이었다.
- 남은 실패는 이벤트 API를 배열 자체로 잘못 읽은 테스트였다. 실제 응답은 `{ok, events}`이며
  `begin_batch_native`가 기존 이벤트를 비운다. `3cfeae4a2d`에서 batch 시작 후 기준값을 잡고
  `events` 배열을 비교하도록 수정했다. 제품 동작을 바꾸거나 검사를 제거하지 않았다.
- nextest 0.9.137 / 저장소 권고 0.9.140 및 새 설정 키 무시 경고가 있다. 버전 변경은 하지 않았다.

## 최종 집중 검증 결과

- 제품 최초 구현: `333ad5c149`. 최종 제품/테스트 기준: **`3cfeae4a2d`**.
- 검증 위치: `/home/edward/mygithub/rhwp-review-3587`, detached 동일 SHA.
  공유 target: `/home/edward/mygithub/rhwp/target/pr-review`.
- manifest `--prepare` 뒤 `cargo fmt --all -- --check`: PASS.
- `deriveManifest`/`buildCaseIndex`로 `issue_3587_` source **8개**가 속한 target을 동적으로
  선택하여 release-test nextest 실행: **74 PASS / 0 FAIL**, 비대상 **1,094 skipped**.
  skip은 이번 집중 filter로 제외된 target 내 다른 테스트이며 전체 회귀 결과가 아니다.
- `cargo clippy --locked --target-dir <공유 target> -- -D warnings`: **PASS, 40.55초**.
- manifest `--check`: PASS. 파생 suite·manifest는 PR source에 추가하지 않았고 review worktree는 clean.
- 증적: `output/3587/b2/focused-3cfeae4a2.log`, `clippy-3cfeae4a2.log`,
  `manifest-3cfeae4a2.log` (마지막 두 파일도 같은 `output/3587/b2/` 아래).

다음 순서는 B2 잔여 지원 조합·실제 생성 비용 확인 후 B3 저장·재열기 및 메인테이너 사용 결과
확인이다. 현재 증거를 시각 판정으로 대체하지 않는다. WASM/workspace Clippy·workspace build·
전체 nextest와 Docker WASM은 이번 절편에서 실행하지 않았으며 B 종료/PR 전 검증에 남아 있다.
