# PR #446 검토 — set_field 후 저장/재오픈 시 필드 값 유실 정정 (#270)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#446](https://github.com/edwardkim/rhwp/pull/446) |
| 작성자 | [@oksure](https://github.com/oksure) (Hyunwoo Park) — 신뢰 컨트리뷰터, 본 사이클 6번째 PR |
| 이슈 | [#270](https://github.com/edwardkim/rhwp/issues/270) (closes) |
| base / head | `devel` ← `oksure:contrib/fix-set-field-persistence` |
| 변경 규모 | +107 / -3, 1 file, 1 commit |
| mergeable | `MERGEABLE` (BEHIND) |
| 검토 일자 | 2026-04-29 |

## 본질

`samples/field-01.hwp` 의 Clickhere 필드에 `set_field("회사명", "PERSIST_TEST")` 후 `exportHwp()` → 재오픈 시 필드 값이 빈 문자열로 유실되던 결함 정정.

### 원인

`rebuild_char_offsets()` 가 FIELD_END(0x04) 마커 갭만 생성하고 **FIELD_BEGIN(0x03) 컨트롤 갭 누락**.

중간 위치 (start_char_idx > 0) 필드의 경우:
1. `rebuild_char_offsets` 가 FIELD_BEGIN 위치에 8바이트 갭을 생성하지 않음
2. `serialize_para_text` 가 갭이 없어 FIELD_BEGIN 을 텍스트 뒤로 밀어냄
3. 재오픈 시 FIELD_BEGIN ↔ FIELD_END 사이에 텍스트가 없으므로 빈 필드로 파싱

### 정정

`field_begin_at` 배열 추가 + `start_char_idx` 위치에 8바이트 갭 생성. 이중 계산 방지를 위해 `control_idx >= ctrls_before_text && start_char_idx > 0` 조건으로 필터링.

```
변경 전: TEXT("라벨: PERSIST_TEST") | FIELD_BEGIN | FIELD_END | PARA_END
변경 후: TEXT("라벨: ") | FIELD_BEGIN | TEXT("PERSIST_TEST") | FIELD_END | PARA_END
```

### 사용자 영향 정황

이슈 #270 작성자 (@hyoseop1231) 명시:
> "LLM agent / MCP / CLI 가 양식을 자동으로 채우는 시나리오 전체가 작동하지 않습니다."

→ 다운스트림 사용자 시나리오 영향 큼.

## 처리 방향

작업지시자 직접 결정 (중간 정황 변경):
- 1차 결정: **옵션 C** (메인테이너가 통합 테스트 직접 추가 후 머지)
- 2차 결정: **옵션 A** (한컴 검증 실패 정황으로 작성자 보강 요청)
- **3차 결정 (최종): cherry-pick 머지** — 한컴 2010 + 한컴 2020 양 환경 검증 통과

## dry-run cherry-pick 결과

`local/pr446` 브랜치 (`local/devel` 분기) 에서 단일 commit cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `1698a9c` (← `2929f30`) | @oksure | fix: set_field 후 저장/재오픈 시 필드 값 유실 수정 (#270) |

cherry-pick 결과: 충돌 없이 자동 적용.

## 검증 게이트 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1069 passed** (1066 → +3 신규 회귀 테스트) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| **작성자 회귀 테스트 3건** | ✅ 모두 통과 (`field_query::tests`) |
| WASM 빌드 (Docker) | ✅ 1m 19s |

## 메인테이너 통합 e2e 추가 (옵션 C 잔재)

옵션 C 결정 시 메인테이너가 직접 추가한 e2e 검증:

`rhwp-studio/e2e/issue-270-set-field-persist.test.mjs` (155 lines):
- field-01.hwp 로드
- `setFieldValueByName("회사명", "PERSIST_TEST")` 호출
- in-memory 검증 → exportHwp → 재 loadDocument → `getFieldValueByName` 검증
- 산출물 `output/hwp/issue_270_persist_test.hwp` 저장 (한컴 편집기 직접 검증용)

`rhwp-studio/public/samples/field-01.hwp` 추가 — Vite `/samples/` 라우팅 정합 (기존 부재).

### e2e 결과

```
PASS: field-01.hwp 로드 성공 (3페이지)
PASS: 초기 get_field 호출 성공
PASS: 초기 회사명은 PERSIST_TEST 가 아니어야 함 (실제: "")
PASS: set_field 호출 성공
PASS: setField newValue = PERSIST_TEST
PASS: in-memory 값이 PERSIST_TEST 여야 함
PASS: exportHwp 호출 성공 (size: 473,600 bytes)
PASS: 파일 저장 성공
PASS: 재 loadDocument 성공
PASS: 재오픈 후 회사명 값이 PERSIST_TEST 여야 함 — 이슈 #270 회귀 게이트
```

## 한컴 편집기 검증 (작업지시자 직접)

산출물: `output/hwp/issue_270_persist_test.hwp` (473,600 bytes)

| 한컴 환경 | "회사명" 필드 | 결과 |
|----------|--------------|------|
| **한컴 2020** | "PERSIST_TEST" 정상 표시 | ✅ **성공** |
| **한컴 2010** | "PERSIST_TEST" 정상 표시 | ✅ **성공** |
| set_field 안 한 다른 필드 (목차 03~05) | "목차 입력" placeholder 정상 표시 | ✅ 회귀 없음 |

→ **메모리 `feedback_self_verification_not_hancom` 의 핵심 게이트 (한컴 편집기 직접 검증) 양 환경 통과**.

### 검증 정황의 의미

본 검증 사이클은 메모리 `feedback_self_verification_not_hancom` 원칙의 정확한 적용 사례:

1. **자기 라운드트립** (rhwp 자체 e2e) — 통과 ✅
2. **한컴 편집기 직접 검증** (한컴 2010 / 2020) — 양 환경 통과 ✅

자기 라운드트립이 통과해도 한컴 호환은 별도 게이트라는 원칙. 본 PR 은 두 게이트 모두 통과.

## 본 PR 의 좋은 점

1. **정밀 진단**: FIELD_END(0x04) 갭만 생성하고 FIELD_BEGIN(0x03) 갭 누락이라는 비대칭 정확 식별
2. **단위 테스트 3건 추가**: `rebuild_char_offsets` 함수 자체 정합성 검증
3. **이중 계산 방지 가드**: `control_idx >= ctrls_before_text && start_char_idx > 0` 조건으로 회귀 회피
4. **이슈 #270 의 사용자 시나리오 (LLM/MCP/CLI 양식 자동 채움) 직접 해결**
5. **변경 범위 한정**: 단일 파일 (`field_query.rs`), +107 / -3

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1069 + svg_snapshot 6/6 + clippy 0 + WASM + e2e |
| 시각 판정 게이트 (push 전 필수) | (해당 없음 — 텍스트 영역, 한컴 편집기 직접 검증으로 대체) |
| **자기 라운드트립 ≠ 한컴 호환** | ✅ 양 게이트 분리 검증 (e2e 자기 라운드트립 + 한컴 2010/2020 직접) |
| 트러블슈팅 사전 검색 | ✅ 직접 관련 트러블슈팅 부재 (필드 직렬화 신규 영역) |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr446` 에서 커밋 |

## 다음 단계

1. 본 보고서 + e2e 파일 + field-01.hwp + 오늘할일 갱신 commit
2. `local/pr446` → `local/devel` → `devel` 머지 + push
3. PR #446 close + 작성자 댓글 (이슈 #270 자동 close)

## 참고

- PR: [#446](https://github.com/edwardkim/rhwp/pull/446)
- 이슈: [#270](https://github.com/edwardkim/rhwp/issues/270)
- 같은 작성자 머지 PR (본 사이클): [#395](https://github.com/edwardkim/rhwp/pull/395), [#396](https://github.com/edwardkim/rhwp/pull/396), [#427](https://github.com/edwardkim/rhwp/pull/427), [#444](https://github.com/edwardkim/rhwp/pull/444)
