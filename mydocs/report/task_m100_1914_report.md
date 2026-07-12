# 최종 결과보고서 — Task M100 #1914

## 이슈

[#1914 파일 열기: 확장자-실체 불일치 시 매직 바이트 스니핑 부재 — .hwpx 명명 HWP5 49/4580건 로드 실패 (한글은 열림)](https://github.com/edwardkim/rhwp/issues/1914)

## 요약

이슈 전제를 정정한다: **제품 열기 경로는 이미 매직 바이트 스니핑으로 동작**하며
서베이 49건 전수가 `rhwp info`(= `HwpDocument::from_bytes`)로 정상 로드된다
(49/49, 버전 5.1.0.1). 서베이의 LOAD_FAIL("ZIP EOCD")은 **확장자를 신뢰하던
roundtrip 게이트 CLI 경로**의 오분류였고, 두 게이트에 실체 판별(FORMAT_SKIP)을
추가해 해소했다. #1891 의 'EOCD 단서'와 #1892 의 도구 축 후속(어댑터 미경유
HWP3 오도성 IR_DIFF)이 같은 뿌리로 함께 해소된다.

## 판별

| 경로 | 판별 방식 | 위장 파일 동작 (수정 전) |
|---|---|---|
| `parse_document` / `detect_format` | 내용(매직 바이트) | 정상 파싱 ✓ |
| `DocumentCore::from_bytes` / `HwpDocument::from_bytes` (`rhwp info` 등) | 내용 | 정상 파싱 ✓ (49/49 실측) |
| `render-diff` (20k/서베이 렌더 경로) | 내용 | 정상 ✓ (#1891 에서 확인) |
| **`hwpx-roundtrip`** | 확장자(parse_hwpx 직행) | **PARSE_FAIL "ZIP EOCD"** ← 서베이 오류 문자열 |
| **`hwp5-roundtrip` 단일 모드** | 필터 없음 | HWP3 실체 → **오도성 IR_DIFF** (#1892 도구 축) |

## 수정

두 게이트의 `roundtrip_one` 진입부에 `detect_format` 스니핑:

- 실체가 게이트 대상이 아니면 **`FORMAT_SKIP`** 상태 (하드 실패 아님, exit 0)
  + 실체 포맷명 + 올바른 게이트 안내를 error 컬럼에 기록.
  - hwpx 게이트: `확장자 위장: 실체 HWP5(OLE/CFB) — HWP5 는 hwp5-roundtrip 사용`
  - hwp5 게이트: HWP3 → `HWP3 저장 검증은 convert/export 경로 사용` (plain
    serialize 는 어댑터 미경유라 SectionPageDef 소실 등 도구-전용 diff 오표기)
- `Unknown`(빈 파일 2건·DRM 래퍼 2건)은 종전대로 파싱 실패 유지 — 정당한 거부.

## 검증

- 서베이 EOCD 실패 .hwpx **53건 전수 재검**: OLE 실체 49건 → **FORMAT_SKIP 49/49
  (exit 0)**, 잔여 4건(빈 파일 2·DRM 래퍼 2) = 비-OLE 로 종전 거부 유지 —
  이슈의 분해(49+2+2)와 정확히 일치.
- 제품 로드: 동일 49건 `rhwp info` **49/49 정상** (내용 스니핑 기존 동작 핀).
- HWP3 `.hwp` → hwp5-roundtrip: 오도성 IR_DIFF=1 → **FORMAT_SKIP** 전환.
- 동일 바이트를 `.hwp` 로 hwp5-roundtrip: **PASS** (진짜 HWP5 게이트 정상 진행).
- `tests/issue_1914.rs` 2건: 제품 스니핑 핀(3포맷) + 게이트 FORMAT_SKIP 핀
  (CARGO_BIN_EXE 로 실제 CLI 구동).
- cargo test 전 스위트 (195 바이너리): **PASS**

## 산출물

- 수정: `src/diagnostics/hwpx_roundtrip_batch.rs`, `src/diagnostics/hwp5_roundtrip_batch.rs`
- 테스트: `tests/issue_1914.rs`
- 문서: plans/task_m100_1914.md, 본 보고서
