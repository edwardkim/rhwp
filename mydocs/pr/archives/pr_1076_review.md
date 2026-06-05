# PR #1076 검토 — set_field_text_at 메타데이터 불일치 (ClickHere 필드 파일 손상)

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #1076 |
| 제목 | fix: set_field_text_at 메타데이터 불일치 — ClickHere 필드 값 설정 시 파일 손상 (#838) |
| 작성자 | oksure (Hyunwoo Park) — 기존 컨트리뷰터 (#971/#1074/#1075) |
| base ← head | `devel` ← `contrib/fix-field-text-corruption` |
| 연결 이슈 | Closes #838 (Bug, 한컴 파일 손상), assignee 본인 지정 완료 |
| mergeable | MERGEABLE / BEHIND (cherry-pick 으로 해소) |
| CI | Build & Test ✅ / Analyze ✅ / CodeQL ✅ |
| 커밋 | 1 (599862ad) |

## 2. 배경 (이슈 #838)

기존 텍스트(안내문) 있는 ClickHere 필드에 `setFieldValue` 호출 →
HWP 저장 후 한컴에서 "파일 손상" 경고. 빈 공간에 설정은 정상,
**기존 텍스트가 있는 경우만** 손상.

## 3. 변경 내용

**`src/document_core/queries/field_query.rs` 단일 Rust 파일.**

### Before (버그)
```rust
para.text = format!("{}{}{}", before, value, after); // 텍스트만 직접 교체
// + field_ranges 수동 시프트
```

### After (안전)
```rust
para.delete_text_at(start_idx, count);   // char_shapes/line_segs/range_tags/char_count 자동 시프트
para.insert_text_at(start_idx, value);   // field_ranges 도 자동 처리
// 현재 필드의 start/end 만 재설정
```

## 4. 검토 의견

### 4.1 트러블슈팅 사전 검색 (feedback_search_troubleshootings_first)

한컴 호환 작업 — 관련 문서 다수:
- `table_paste_file_corruption.md`
- `cell_split_save_corruption.md`
- `picture_save_hancom_compatibility.md`
- `task178_second_attempt_hancom_rejection.md`

→ 모두 동일 패턴(직접 텍스트 조작 → 메타데이터 시프트 누락 →
한컴 파일 손상)의 사례들. 본 PR 진단이 이 패턴과 정합 —
과거 함정의 동일 root cause 를 새 경로(필드)에서 해소.

### 4.2 강점

- **root cause 정밀**: `char_shapes`/`line_segs`/`range_tags`/
  `char_count` 시프트 누락 → PARA_CHAR_SHAPE 파싱 실패. 명확 진단.
- **재사용 패턴**: 기존 `Paragraph::delete_text_at` + `insert_text_at`
  (paragraph.rs:239/382) 위임. 두 메서드는 char_offsets + UTF-16
  까지 견고하게 시프트 — 신뢰 가능한 안전 경로.
- **수동 시프트 로직 제거**: field_ranges 수동 보정 코드 제거,
  `delete_text_at/insert_text_at` 자동 처리에 위임. 코드 단순화 +
  버그 표면 감소.
- 주석에 #838 명시 — 추후 회귀 추적 용이.

### 4.3 한컴 스펙 교차 검증 (보조)

작업지시자 지시로 `mydocs/tech/한글문서파일형식_5.0_revision1.3.md` 교차
검증 시도. **단 스펙에 잘못된 정보 기록 가능 — PR 의 실제 관찰이 권위.**
참고 일치 사항만 기록:

| PR 가 시프트하는 메타데이터 | 스펙 매칭 (보조 참고) | 관찰 권위 |
|---------------------------|---------------------|---------|
| `char_count` | 표 58 PARA_HEADER "문단의 글자 수" | PR 관찰: 텍스트 길이 변경 시 갱신 필요 |
| `char_shapes` | 표 61 PARA_CHAR_SHAPE "시작 위치 + ID" 8×n | PR 관찰: 시프트 누락 시 파싱 실패 |
| `line_segs` | 표 62 PARA_LINE_SEG "텍스트 시작 위치" + 9 필드 | PR 관찰: 시프트 누락 시 레이아웃 불일치 |
| `range_tags` | 표 63 PARA_RANGE_TAG "영역 시작/끝" 12×n | PR 관찰: 시프트 누락 시 마킹 무효 |

→ 스펙도 같은 방향을 시사하나, **PR 의 관찰("ClickHere 필드에서
파일 손상 재현 + delete_text_at/insert_text_at 위임 후 해소") 가
1차 권위**. 본 검토는 PR 의 실제 동작 관찰을 신뢰.

### 4.4 검토 포인트

- **자매 PR #1080** (`set_cell_field_text` 같은 파일 수정, 같은
  #838 분리 해결): 동일 파일 `field_query.rs` 의 다른 함수
  (`set_cell_field_text`). #1076 과 #1080 이 같은 패턴(직접 텍스트
  교체 → delete+insert 위임)을 다른 진입점에 적용. cherry-pick
  시 충돌 가능 → **#1076 먼저 처리 후 #1080 검토** 권고.
- **단위 테스트 부재**: PR 본문에 `cargo test --lib 1335 passed`
  만 명시. 신규 회귀 가드 테스트 없음. 그러나 #838 자체가 한컴
  실측 재현이라 테스트화 부담은 별개 — 보고서 기록.
- 본질이 시각 판정 아닌 **파일 무결성 (한컴 호환)** —
  `feedback_self_verification_not_hancom`: 자체 검증만으로
  충분하지 않을 수 있음. **한컴 수동 검증 게이트 권고**
  (재현 시나리오: 안내문 있는 ClickHere 필드 → setFieldValue →
  저장 → 한컴 열기 → 손상 경고 없음 확인).

## 5. 검증 계획

- [ ] cherry-pick (`599862ad` → 최신 devel)
- [ ] 전체 `cargo test` + `cargo clippy -- -D warnings` +
      `cargo fmt --all -- --check`
- [ ] WASM 빌드 (field_query 변경 — WASM 영향)
- [ ] **한컴 수동 검증 게이트 권고** (작업지시자 판단):
      ClickHere 필드 + 안내문 + setFieldValue → 저장 →
      한컴 오피스 열기 → 파일 손상 경고 없음 확인

## 6. 판단 (잠정)

root cause 정밀 (HWP5 메타데이터 시프트 누락) + 안전 메서드
위임 + 수동 시프트 코드 제거. 트러블슈팅 사례들과 동일 패턴
해소. 자체 검증(cargo test/clippy/CI) 통과는 PR 본문 명시.

본질이 한컴 호환 파일 무결성 → **한컴 수동 검증**이 판정 핵심.
PR #1080 자매 관계 인지하고 본 PR 우선 처리 권고.

검증 결과 + 한컴 수동 검증에 따라 `pr_1076_report.md` 작성.