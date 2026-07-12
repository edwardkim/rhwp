# PR #1924 검토 — Task #1916+#1917: 표 CTRL_HEADER 빈 데이터 방출 + BinData 64MB 상한 (배치)

- 작성일: 2026-07-05 / 검토자: Claude (메인테이너 대행 검토)
- PR: planet6897 → devel / MERGEABLE, CI 11 pass / 시간순 2번째
- 연결 이슈: #1916, #1917 (본문 `Closes` 선언) — 작업지시자 지시의 배치 PR (커밋은 이슈별 분리)

## 1. PR 요약 (커밋 2건)

### 커밋 1 — #1916: `serialize_table` CTRL_HEADER 빈 데이터 방출
`raw_ctrl_data` 부재 시 빈 값을 방출해 재파스 표의 **CommonObjAttr 전체**(treat_as_char/
text_wrap/flowWithText/크기)가 기본값으로 붕괴하던 결함. 부재 시
`serialize_common_obj_attr(&table.common)` **합성 방출**로 수정 — 다른 GSO 컨트롤과 동일
계약. 실경로 = raw 없는 IR의 plain 저장(HWPX 파스 IR, 편집기 신설 표).

- 판별 기록의 가치: 서베이 12건은 전부 .hwp 명명 HWPX(역방향 위장 — #1914/PR #1922 축)로
  분류하고, 어댑터 경로는 Stage 2 합성으로 정상임을 실측(hwp5-anchor-trace) — 수정 범위를
  plain 저장 경로로 정확히 좁혔다.

### 커밋 2 — #1917: HWPX BinData 상한 64MB → 512MB
비압축 BMP/TIF 대형 이미지 실문서(최대 103.7MB, 한글 정상 열람)를 zip-bomb 방어 상한이
거부 — 그림 소실 + 왕복 pic 드롭. 512MB 상향(엔트리당 상한 유지, 무제한 read_to_end 차단
목적 보존). 서베이 4건 전수 PASS 전환.

## 2. 코드 검토

- #1916: raw 우선 + 부재 시 합성의 분기 구조가 명료. raw 보존 경로·어댑터 경로 불변.
  attr=0이어도 `pack_common_attr_bits` 경유로 flow bit 13 포함 — flowWithText 축(#1911
  후속 계열)과 정합. 핀 `tests/issue_1916.rs`.
- #1917: 상수 1줄 + 사유 주석. 512MB 초과 실문서의 pass-through 보존은 실수요 확인 시
  후속으로 기록 — 적절한 범위 절제. 핀 `tests/issue_1917.rs`(인메모리 70MB 왕복).
- 두 커밋 모두 렌더 경로 무접촉, 리팩토링 v2 §1 금지 목록 비저촉.

## 3. 게이트 결과 (devel `5d5a635e` + #1923 + #1924 결합)

| 게이트 | 결과 |
|---|---|
| GitHub CI | 11 pass / 1 skip |
| cargo fmt --check | 통과 |
| cargo clippy --profile release-test --all-targets | 경고 0 |
| cargo test --profile release-test --tests (issue_1916/1917 핀 + roundtrip baseline) | **2,878 통과 / 실패 0** |

- OVR/시각 판정 미적용 — 파서 상수 + 직렬화기 전용.

## 4. 판단 (작업지시자 승인 대기)

- 두 커밋 모두 건전. 배치 구성도 지시 이력과 부합(검증·CI 1회). #1923과 파일 겹침 0.
- 머지 시 #1916/#1917 close 여부는 승인에 따름 (`Closes` 선언 — devel 대상이라 자동
  close는 동작하지 않으므로 수동 처리).
