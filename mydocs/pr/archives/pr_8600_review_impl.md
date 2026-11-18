# PR #8600 통합 적용·메인터너 보정 기록

## 통합 PR의 exact code head CI

- [통합 PR #8857](https://github.com/edwardkim/rhwp/pull/8857), code head `29c74b69851bc6578f10a2c78e23592a05b69a5d`.
- [CI 실행](https://github.com/edwardkim/rhwp/actions/runs/42621490405)과 같은 code head의 CodeQL·Render Diff·Adapter·Proptest·정책 검사를 실제 확인했습니다. 기록 확인 시각: `2026-11-09T22:58:24.689803+09:00`.
- 기본 기능 worker A–E의 실제 nextest는 **12,934 PASS / 0 FAIL / 45 skip**입니다. 기존 로컬 13,105건과 실행 범위·HEAD가 다르므로 합산하지 않습니다.
- source·test·workflow를 추가로 변경하지 않고 이 검토·오늘할일만 단일 부모 trailing commit으로 보완합니다. trailing head의 실제 CI/fast-pass와 mergeability는 push 뒤 별도로 확인하며, 문서만 바뀌었다는 이유로 통과를 미리 가정하지 않습니다.
- [잔여 이슈 #8856](https://github.com/edwardkim/rhwp/issues/8856)은 9문서군의 글리프·간격·도형 및 clip5587 Native 4쪽의 엄격 0.1%p 미충족을 계속 추적합니다. 부분 해결 #8635·#8636과 부모 #7445는 일괄 종료하지 않습니다.


## 최종 source와 재사용 범위

제출 code head `2e262d0d4b2885b130e2db917b86e894710425e4`, base `e00f9b7e26e581896fa17eaff3c8eb3f4889f9d4`입니다. 아래 전체 회귀·전쪽 수치는 원 실측 HEAD의 증거를 재사용한 기록입니다. 최신 집중 검사와 페이지별 producer, 엄격 0.1%p 잔여 한 건은 [재사용·최종 판정](../assets/planet6897_20261106/final-b768bee1-20261109/validation-reuse.md)을 따릅니다. 보호 TSV 표시값 하락은 0입니다.

## 최종 구현·검증 결과

검증 code head `eb876d1416fb558c874f9efec65c5722a6769751`, base `b768bee1c4c0023087687f79a0a95f12108743e9`에서 원 기여와 메인터너 보정을 함께 검증했습니다. 최소 글꼴 전체 회귀 13,105 PASS / 0 FAIL / 45 skip, Native 전용 73 PASS이며, Native/fresh WASM 각각 94조건·2,995쪽의 미달·누락·보호 표시값 하락은 없습니다. 원 PR의 개별 입력·페이지 판독과 보정 내용은 [최종 review](pr_8600_review.md)에 연결합니다. 원격 통합 PR의 최종 CI와 병합은 별도 단계입니다.

## 단계별 기록의 시점

아래 R 단계와 과거 실행의 진행 중·실패·추가 검증 표현은 해당 시점의 원인 추적 기록입니다. 과거 결과를 현재 code head의 통과로 합산하지 않으며, 현재 상태는 위 최종 결과와 연결된 review를 기준으로 판단합니다.

## 대상과 단계

[원 PR #8600](https://github.com/edwardkim/rhwp/pull/8600)의 원 head는 `983933bc448e5ed4f7c8d15be97d894e129a5b9b`다.
[공통 계획](../assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `983933bc448e5ed4f7c8d15be97d894e129a5b9b` | `284682cbe0678f2c97f74ad767908faeeb846c20` | fix(renderer): 도형 그림 채우기 크기와 한컴 윤고딕 250 연결 (#8575) |

## 최종 리베이스 후 적용 상태

검증 base `b768bee1c4c0023087687f79a0a95f12108743e9`, 검증 제품 head
`eb876d1416fb558c874f9efec65c5722a6769751`의 실제 Git 이력을 대조해 위 적용 SHA를 갱신했다.
[고유 56개 최종 적용표](../assets/planet6897_20261106/contributor-commit-map-final.tsv)는 원 기여 commit의
cherry-picked footer와 현재 적용 commit을 일대일로 확인한 결과다. 중복 patch·bridge의 기존 처리는 유지한다.
이번 b768bee1 리베이스에는 실제 파일 충돌이 없었습니다. index.lock 경합이 끝나고 lock이 없는 상태에서 계속했으며 lock을 삭제하거나 todo를 바꾸지 않았습니다. 제품 src/crates/tests/scripts와 지침의 바이트는 이전 후보와 같고 의존성 변경에 대한 새 검증을 수행합니다.
이력 확인과 현재 제품의 기능·회귀·시각 검증 판정은 구분하며, 필수 검증 완료 전에는 승인하지 않는다.

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.

## R29 선행 분석 — 실제 번호 폭과 형식의 예약 폭

고정 base `a7feb849e11642b1438b7e9ee460ff4c8a7c946e` 대비 R28의 `interview.hwp`
1쪽 raw는 96.80%에서 96.01%로 낮아졌다. 한컴 윤고딕 250의 별칭을 제거해 수치를
되돌리는 방식은 사용하지 않는다. 독립 Print 제목의 실제 문자 원점 간격은 `면→접`
약 34.54 CSSpx, `1→0` 약 27.03 CSSpx이며 현재의 34.56·27.04 CSSpx와 일치한다.
이는 소스의 90% 장평과 실제 프로그램의 전진을 소비한 개선이다.

하락 영역은 목록 본문이다. 원본 DocInfo의 두 관련 번호 정의는 `attr=0x8`,
`width_adjust=2000`, `text_distance=0`, 글자 모양 10, 형식 `^1`이다. 사양 표 40의
bit2는 실제 인스턴스 문자열의 너비를 사용할지 결정하며 이 원본에서는 꺼져 있다.
독립 Print는 각 단에서 1~10번 본문을 각각 x86.64pt·344.64pt의 같은 시작점에 둔다.
현재는 번호 1·2·10의 자연 전진이 공통 `ListMarkerGeometry.area_px`에 더해져
같은 원본 본문이 서로 다른 x로 이동한다. 기본 개요에는 참조 형식 폭을 쓰지만,
파일에 명시된 번호 정의에는 이 값이 없어 실제 glyph 폭으로 대체하는 것이 원인이다.

수정은 `번호 정의와 형식 → MarkerHead의 참조 예약 폭 → list_marker_geometry_for →
줄 나눔과 실제 본문 배치`의 공통 경로에서 수행한다. bit2가 꺼져 있으면 한 자리
참조 번호로 형식을 확장해 ASCII 반각·비ASCII 전각의 형식 칸을 예약하고,
실제 번호 glyph는 원래 프로그램·정렬로 그린다. bit2가 켜진 실제 폭 경로는 보존한다.
특정 문서·번호·좌표의 예외를 추가하지 않는다. 관계 검사에서는 1~10번 본문의 같은
시작점·이어지는 줄·원문 보존과, bit2를 켠 반례의 가변 영역을 확인한다.

R28 전체 Rust·실제 전쪽 측정은 기존 검증 소스에서 계속한다. 이 분석을 R29의 구현·
Native/fresh WASM 재측정 완료로 쓰지 않으며, 확정 source의 실제 결과를 뒤에 연결한다.

R29 최소 글꼴 집중 회귀는 관련 기존 검사와 함께 **33건 모두 PASS / exit 0**이다.
새 원본 관계 검사의 수정 전 FAIL과 수정 후 PASS를 확인했다. Native/fresh WASM 전쪽
재측정과 전체 회귀·필수 lint는 별도 게이트이며 아직 완료 판정하지 않는다.
