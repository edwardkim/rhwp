# PR #1922 검토 — Task #1914: roundtrip 게이트 확장자 위장 파일 FORMAT_SKIP 분류

- 작성일: 2026-07-05 / 검토자: Claude (메인테이너 대행 검토)
- PR: planet6897 → devel / MERGEABLE, CI 11 pass / 생성 2026-07-04 23:34 (시간순 4번째)
- 연결 이슈: #1914 — 본문 `Closes #1914`

## 1. PR 요약

**이슈 전제 정정 + 도구 분류 수정** (제품 코드 무변경):

- 제품 열기 경로(`parse_document` 등)는 이미 매직 바이트 스니핑으로 동작 — 서베이 49건
  전수 재검 결과 49/49 정상 로드. 이슈의 LOAD_FAIL은 **확장자를 신뢰하던 roundtrip 게이트
  CLI의 오분류**였음을 판별 (#1891 'EOCD 단서'의 실체 — #1913의 클러스터 판별과 연결).
- 수정: `hwpx-roundtrip`/`hwp5-roundtrip`의 `roundtrip_one` 진입부에 `detect_format`
  스니핑 — 실체가 게이트 대상 포맷이 아니면 **FORMAT_SKIP**(하드 실패 아님) + 실체
  포맷명 + 올바른 게이트 안내. `Unknown`(빈 파일/DRM)은 종전대로 파싱 실패 유지.
- HWP3→hwp5-roundtrip의 오도성 IR_DIFF(어댑터 미경유 SectionPageDef 소실, #1892 도구 축)도
  FORMAT_SKIP으로 정리.

변경: `src/diagnostics/hwp{5,x}_roundtrip_batch.rs` + 핀 2건(`tests/issue_1914.rs`,
CARGO_BIN_EXE 실 CLI 구동) + 문서 3건.

## 2. 코드 검토

- 제품 경로 무접촉 — 진단 CLI의 분류 체계만 변경. `is_hard_fail`에서 FORMAT_SKIP 제외
  (exit 0)는 게이트 의미론상 옳다: 위장 파일은 "해당 게이트의 검증 대상 아님"이지 회귀가
  아님.
- 검증 분해(49 OLE + 2 빈 파일 + 2 DRM = 53)가 이슈의 분해와 일치 — 전수 재검 기반.
- `samples/hwpx` 전수 회귀 게이트(`hwpx_roundtrip_baseline`)와의 상호작용: 위장 파일이
  샘플에 없으면 무변동 — 전체 테스트로 확인.

## 3. 게이트 결과 (devel `bf5228df` + PR 테스트 머지)

| 게이트 | 결과 |
|---|---|
| GitHub CI | 11 pass / 1 skip |
| cargo fmt --check | 통과 (결합 게이트) |
| cargo clippy --profile release-test --all-targets | 경고 0 (결합 게이트) |
| cargo test --profile release-test --tests | **2,875 통과 / 실패 0** (devel+4PR 결합) |

- OVR/시각 판정 미적용 — 렌더 경로 무접촉 (진단 도구 분류만).

## 4. 판단 (작업지시자 승인 대기)

- 낮은 위험, 도구 정확성 개선 + #1891/#1892/#1914 3개 이슈의 조사 축을 정리하는 가치.
- #1913과 상호 보완(같은 위장 클러스터의 도구 축) — 독립 머지 가능(4 PR 파일 겹침 0 확인).
