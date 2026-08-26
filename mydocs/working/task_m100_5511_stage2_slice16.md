# #5511 Stage 2 열여섯 번째 수직 절편 — watermark CLI 계약 선행 고정

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 테스트 커밋: `d57bfe4ba`
- 수행일: 2026-08-19
- 상태: 완료 — watermark handler 이동 절편 승인 대기

## 1. 절편 선정과 중단 판단

열여섯 번째 후보는 `inspect watermark` handler와 `inspect_watermark_scan_unit` helper의
물리 이동이었다. 그러나 이동 전 계약을 조사한 결과 탐지 코어
`document_core::queries::stego_scan`에는 세 축의 단위 테스트가 충분한 반면, 공개 CLI를 실제로
실행하는 전용 integration contract는 없었다. 기존 보호는 비밀번호 지원 목록과 상위 `inspect`
subcommand·MCP 선언의 일부에 그쳤다.

이 상태에서 handler를 먼저 옮기면 인자 파싱, HWP/HWPX 문서 순회, 중첩 location, 사람용·JSON
출력, provenance, 암호 및 exit code가 바뀌어도 코어 단위 테스트만 통과할 수 있다. 계획서의
중단 조건에 따라 이번 절편에서는 source 이동을 하지 않고 characterization contract만 독립
변경 단위로 추가했다.

절편 시작·종료 시 `upstream/devel`은 `1a6ce79fd`로 동일했다. 활성 PR 중 `src/main.rs`,
`src/cli/queries/security_inspection.rs`, 신규 watermark 계약, CLI catalog와 이 보고서에 겹치는
변경은 없었다.

## 2. 추가한 보호 계약

새 `tests/cases/watermark_inspection_contract.rs`의 8개 테스트가 다음 공개 동작을 고정한다.

1. 정상 HWP5·HWP3의 완전한 빈 JSON 봉투와 실제 스캔 문자 수
2. 실행 중 합성한 HWP5와 실제 변환한 HWPX에서 hidden·homoglyph·whitespace 세 축 탐지
3. 제로폭 비트열 `Hi` 복호, 심각도·축 집계와 표 셀 중첩 location
4. `--kind hidden|homoglyph|whitespace|all`의 정확한 분할
5. 사람용 출력의 탐지 건수·복호 근거·중첩 위치
6. 누락·잘못된 인자와 파일 오류의 exit 1/2 및 빈 stdout
7. JSON·사람용·필터 스캔 전후 입력 문서 byte 불변
8. 암호 없음·오류·정답 경로와 capabilities·MCP schema·annotation·outputFields

합성 payload는 테스트 source에 검토 가능한 Unicode escape와 문자열로만 두고, 실행 중 정상
HWP5 표본에 삽입한다. 생성된 HWP/HWPX는 각 테스트 종료 시 제거하며 저장소에 공격 문서를
커밋하지 않는다.

## 3. 조사 중 발견한 별도 탐지 정책 위험

정상 공공문서 표본 `2025 행정업무운영 편람(최종)`의 HWP와 HWPX를 같은 CLI로 교차검사하자
양쪽 모두 다음과 같이 동일했다.

| 항목 | HWP | HWPX |
|---|---:|---:|
| 전체 탐지 | 22 | 22 |
| hidden_char | 0 | 0 |
| homoglyph | 0 | 0 |
| whitespace | 22 | 22 |
| medium / low | 15 / 7 | 15 / 7 |

대상은 표 셀과 본문의 정렬용 후행 공백 8~52자다. HWP와 HWPX가 같은 결과이므로 변환이나
parser 차이가 아니라, 공공기관 HWP 편집 습관과 현재 `WS_TRAIL_MIN=8` 휴리스틱의 충돌로
판정한다. 이는 받은 문서의 은닉 추적을 찾는 도구가 정상 공문서에서 반복 경보를 내어 신뢰를
잃을 수 있는 실제 오탐 위험이다.

이번 #5511 절편은 CQRS 경계의 move-only 작업이므로 임계값이나 판정 알고리즘을 바꾸지 않았다.
또한 이 22건을 정상 기대값으로 새 테스트에 고착하지 않았다. 공공문서 코퍼스의 후행 공백 길이·
위치·탭 혼합 여부를 별도로 계측하고 기능 탐지식 완화 규칙을 설계하는 후속 이슈가 필요하다.

## 4. 지표 변화

| 항목 | Stage 2 절편 15 | Stage 2 절편 16 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 39,239 | 39,239 | source 이동 없음 |
| `src/cli/queries/security_inspection.rs` | 382 | 382 | source 이동 없음 |
| watermark 전용 integration source | 없음 | 1 | 신규 |
| watermark CLI 계약 | 0 | 8 | +8 |
| Rust test source | 750 | 751 | +1 |
| static test attribute | 3,699 | 3,707 | +8 |
| `main.rs` CC>25 함수 | 19 | 19 | 변화 없음 |
| `main.rs` 최대 CC | 68 | 68 | 변화 없음 |
| security inspection 모듈 CC>25 함수 | 0 | 0 | 변화 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | 변화 없음 |
| `rhwp::model` 직접 참조 | 62 | 62 | 변화 없음 |
| `rhwp::parser` 직접 참조 | 72 | 72 | 변화 없음 |
| `rhwp::renderer` 직접 참조 | 24 | 24 | 변화 없음 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 신규 `watermark_inspection_contract` focused nextest | 8/8 통과 |
| release-test 전체 nextest | 7,756/7,756 통과, 3 slow, 38 skipped, 170.705초 |
| `cargo check --all-targets` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 3 ignored |
| Rust test suite manifest | 통과, 751 sources / 3,707 static test attrs / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| test policy 자체 계약 | integration 16/16, unit tier 12/12 통과 |
| `scripts/tests/test_ci_impact_workflow.py` | 30/30 통과 |
| `git diff --check` | 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, #5511 신규 오류 없음 |

최초 focused 컴파일은 테스트 기대 문자열의 quote escape 오류로 실패했다. 제품 구현 문제는
아니었으며 해당 기대값을 Rust raw string으로 고친 뒤 동일 focused 8건과 전체 게이트를 처음부터
통과했다.

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. Cargo가 이름이 인접한 두 verifier package의 lockfile 순서만 재정렬했으나
#5511과 무관한 파생 변화이므로 추적 변경에서 복원했다. 테스트만 추가했으므로 시각 검증과 WASM
빌드는 추가하지 않았다.

Markdown 검사는 기준선에도 존재하는 `agent_capability_registry.md`의 중복 ID·진입점 링크
무결성 오류 16건을 보고했다. 이번 절편이 추가한 보고서 링크 오류는 없다.

## 6. 다음 절편 관문

다음 절편은 이제 보호된 `inspect_watermark`와 `inspect_watermark_scan_unit`만
`security_inspection.rs`로 물리 이동한다. 이동 전후 출력 비교에는 정상·세 축 양성·각 필터·
사람용·오류·암호 경로와 공식 편람 HWP/HWPX의 현재 결과를 포함하되, 마지막 항목은 동작
동등성만 확인하고 장기적으로 올바른 clean 판정이라고 선언하지 않는다.

이동 절편에서는 watermark 탐지 임계값·코어 알고리즘·service 이행을 바꾸지 않는다. 공공문서
후행 공백 오탐 개선은 별도 이슈·계획·승인으로 처리한다. 다음 절편은 메인테이너 승인 전 시작하지
않으며 remote push도 별도 승인 전 수행하지 않는다.
