# PR #8326 검토 — fix(hwpx): 내보낸 HWPX의 칸 배경 그림과 HWP 그림 글머리표가 다른 그림을 가리키지 않게 한다

## 최종 판정

**메인터너 보정 후 수용 가능 — 로컬 필수 검증 완료.** 원 기여 commit을 보존한 누적 후보에서 기능·전쪽 Native/fresh WASM·필수 회귀·lint를 완료했다. 원 PR의 오래된 head를 직접 강제 merge하는 판정이 아니며, 통합 후보의 원격 처리 승인과 해당 head CI를 선행한다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8326, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `32c2a1e9febb303ac0662021279cf9171b346cb8`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:26:44.159666+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: CONFLICT (content): Merge conflict in src/model/style.rs; CONFLICT (content): Merge conflict in src/parser/hwpx/mod.rs; CONFLICT (content): Merge conflict in src/serializer/doc_info.rs; CONFLICT (content): Merge conflict in src/serializer/hwpx/header.rs.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: 동일 원 head의 제출된 check에 failure/cancelled 없음.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 기존 해결과 실제 저장 계약

HWP 그림 글머리표의 기본 참조는 #8438 → #8517과 `c97709` 계열에서 이미 보정됐다. 최신 `Bullet::image_reference`와 누락 참조 오류·그림 효과·Storage 예산을 유지하며, 이번 후보의 새 기능은 HWPX `header.xml`의 `img`/`image` 참조를 본문과 동일하게 manifest ID에서 정규화하는 것이다. 글꼴 참조 ID는 변경하지 않는다. 원 PR의 오래된 HWP 모델 표현을 복구하지 않았다.

실제 HWP → HWPX 내보내기에서 `--verify --verify-pages`를 base/후보 양쪽의 17쪽·3쪽 문서에 실행해 모두 성공했다. 저장본도 같은 독립 Print의 전체 20쪽과 비교했다. 칸 배경 참조와 서로 다른 원본 그림 바이트의 보존은 `hwpx_export_header_image_refs`의 세 검사로 확인한다. NewYear 그림 글머리표는 기존 해결 출력이 유지된다. 추가 byte-guard 입력은 데이터 참조 검사이며 별도 전체 문서 시각 일치를 주장하지 않는다.

원본 17쪽의 N/W 최저는 각각 99.62%, 3쪽은 98.43%다. 저장 재열기 17쪽은 base의 N/W 96.68/96.75%에서 후보 99.62/99.62%로 개선됐고 3쪽은 양쪽 98.43%를 유지했다. 실제 사용한 원본·PDF·저장본의 Git blob은 [입력 검증](assets/semanticist21-20261105/input-commit-verification.json)에 있다.

최종 WASM의 실제 `exportHwpx` → 재열기 → 재저장도 실행했다. 첫 저장은 정식 증거에 포함된 Native 저장본과 두 파일 모두 바이트가 같다. 두 번째 저장은 ID가 다시 정규화돼 파일 hash가 달라져도 header/body의 참조 10개·2개가 해소하는 그림 바이트는 각각 모두 같다. [실제 binding 결과](assets/semanticist21-20261105/final-wasm-image-save.json).

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·새 통합 PR·merge·후속 댓글은 현재 아직 실행하지 않았다. 승인 뒤 해당 제출 head의 CI와 최신 base의 충돌 여부를 다시 확인한다.
