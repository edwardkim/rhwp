# Stage 3 단계별 완료보고서: Reflow on-demand + WASM API + 모달 UI

- **타스크**: [#177](https://github.com/edwardkim/rhwp/issues/177)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task177`
- **일자**: 2026-04-18
- **단계**: Stage 3 / 4

## 1. 수행 범위

구현계획서 Stage 3 + 작업지시자 추가 지시(UI 모달 포함, 자동보정 기본 선택) 반영:

1. **`DocumentCore::reflow_linesegs_on_demand`** — 사용자 명시 요청에 의한 전체 reflow
2. **WASM API 2개** — `getValidationWarnings()` · `reflowLinesegs()`
3. **rhwp-studio 모달 UI** — 문서 로드 시 경고 있으면 모달, 기본 선택 = 자동 보정

## 2. 산출물

### 2.1 Rust 측 수정

**`src/document_core/commands/document.rs`**:
- `needs_reflow_broadly(para)` 신규 — `needs_line_seg_reflow` + 빈 line_segs 케이스 포함
- `reflow_linesegs_on_demand(&mut self) -> usize` 신규 — 전체 reflow + 재구성 + paginate
- 단위 테스트 4개 추가 (`needs_reflow_broadly_covers_*`)

**`src/wasm_api.rs`**:
- `getValidationWarnings() -> String` (JSON) 신규 — count/summary/warnings 구조
- `reflowLinesegs() -> usize` 신규 — reflow 실행 + 처리 개수 반환

**`src/wasm_api/tests.rs`**:
- 3개 단위 테스트 추가 (JSON 형태, 빈 문서 케이스)

### 2.2 rhwp-studio 측 수정

**`src/core/wasm-bridge.ts`**:
- `ValidationReport` 타입 정의 (export)
- `getValidationWarnings(): ValidationReport` 메서드
- `reflowLinesegs(): number` 메서드

**`src/ui/validation-modal.ts`** (신규, 200줄):
- `ValidationModal` 클래스 — 3버튼 커스텀 모달 (자동 보정 · 그대로 보기 · ×)
- `ValidationChoice` 타입 — `'auto-fix' | 'as-is' | 'cancel'`
- `showValidationModalIfNeeded(report)` 헬퍼 — count=0 이면 모달 미생성
- **기본 포커스 = 자동 보정 버튼**, Enter 키 = 자동 보정 실행
- 상세 보기 `<details>` 토글 (최대 50건 표시)

**`src/main.ts`**:
- `showValidationModalIfNeeded` import
- `initializeDocument` 말미에 검증 훅 추가:
  1. `wasm.getValidationWarnings()` 호출 + 콘솔 로그
  2. count > 0 이면 모달 표시 + 사용자 선택 대기
  3. 선택이 `'auto-fix'` 이면 `reflowLinesegs()` + `canvasView.loadDocument()` 재렌더
  4. 상태바에 `(비표준 lineseg N건 자동 보정됨)` 표시

### 2.3 WASM 바인딩

`pkg/rhwp.d.ts` 에 새 API 노출 확인:

```
getValidationWarnings(): string;
reflowLinesegs(): number;
```

WASM 빌드 성공 (1분 16초, rhwp_bg.wasm 3.72MB).

## 3. 검증 결과

### 3.1 Rust 단위 테스트

- `document_core::commands::document::validate_linesegs_tests`: **10 passed** (기존 6 + Stage 3 신규 4)
- `wasm_api::tests::test_get_validation_warnings_*`, `test_reflow_linesegs_*`: **3 passed**
- 전체 라이브러리: **871 passed, 0 failed, 1 ignored** (Stage 2의 864 대비 +7, 회귀 0건)

### 3.2 TypeScript 빌드

```bash
cd rhwp-studio && npx tsc --noEmit
# (출력 없음 = 에러 0)
```

### 3.3 WASM 빌드

```bash
docker compose run --rm wasm
# Finished in 1m 16s
# rhwp_bg.wasm 3.72MB
```

### 3.4 통합 테스트 (HWPX 라운드트립)

```
10 passed, 0 failed
```

Stage 2의 8개 + `task177_lineseg_preserved_on_roundtrip_ref_*` 2개 유지.

## 4. 완료 기준 대조

구현계획서 Stage 3 완료 기준:

| 기준 | 상태 | 근거 |
|---|---|---|
| `DocumentCore::reflow_linesegs_on_demand()` 추가 | ✅ | +스타일 재해소 + composed/paginate 재수행 |
| `DocumentCore::validation_report()` 접근자 | ✅ | Stage 1에서 완료 |
| WASM API `getValidationWarnings()`, `reflowLinesegs()` 추가 | ✅ | `pkg/rhwp.d.ts` 730/958행 |
| rhwp-studio 모달 UI 신규 | ✅ | `validation-modal.ts` 200줄 |
| 문서 로드 훅에서 모달 트리거 | ✅ | `initializeDocument` 말미 |
| 기본 포커스 = 자동 보정, Enter = 자동 보정 | ✅ | `bindKeyboard` + primaryBtn focus |
| 경고 0건 시 모달 미생성 (비침습) | ✅ | `showValidationModalIfNeeded` 에서 `count === 0` → 즉시 `'as-is'` |
| Rust 단위 테스트 5개 | ✅ | 실제 7개 (document 4 + wasm 3) |
| `reflow_zero_height_paragraphs` 동작 유지 | ✅ | 변경 없음 |
| WASM 빌드 통과 | ✅ | Docker 빌드 성공 |

## 5. 주요 설계 결정

### 5.1 자동 보정이 기본 선택 (작업지시자 지시)

모달 표시 시:
- **포커스**: 자동 보정 버튼 (`.dialog-btn-primary`)
- **Enter 키**: 자동 보정 실행
- **Escape / ×**: `cancel` 반환 (그대로 보기 유지)
- **오버레이 클릭**: `cancel`

대부분의 사용자가 "비표준이면 보정해주는 게 좋다"를 기대하므로 기본값을 자동 보정으로. 다만 **명시적 선택**은 여전히 모달로 사용자에게 요청함 (Discussion #188 원칙).

### 5.2 3버튼 대신 2버튼 + 상세 토글

초기 설계는 `[자동 보정] [그대로 보기] [상세 보기]` 3버튼이었으나, 실제 UX에서는:
- [자동 보정] [그대로 보기] **2 버튼**
- 본문에 `<details>` 를 두어 펼치기 버튼으로 상세 내용 인라인 표시

→ 공간 효율 + 사용자 선택 혼란 방지. 상세 보기는 "참고" 수준.

### 5.3 로드 훅 위치 — `initializeDocument` 말미

모든 로드 경로(파일 업로드 / URL 파라미터 / Chrome 확장 / 내부 메시지)가 이 함수를 공통으로 거침. 여기에서 한 번만 훅 추가하면 모든 경로에서 동작.

### 5.4 reflow 후 렌더 재계산

`reflow_linesegs_on_demand` 가 `self.paginate()` 를 호출하므로 WASM 측에서는 이미 재계산 완료. TypeScript 측은 `canvasView?.loadDocument()` 로 페이지 캐시 재로드.

### 5.5 JSON 직렬화는 수동

`serde_json` 의존 추가를 피하고 한국어 경고 메시지의 단순 escape 로 충분. 키 정렬로 결정론적 출력.

## 6. 알려진 제한

1. **대량 경고 시 상세 목록 스크롤**: 현재 최대 50건만 표시 후 "외 N건". 대량 문서에서는 전체 다운로드 기능이 없음. 후속 이슈에서 고려.
2. **사용자 선호 저장**: "다시 표시하지 않음" 체크박스 없음. 매번 로드 시 모달 표시. 향후 UX 이슈로 분리.
3. **모달 스타일 다국어**: 현재 한국어 전용. i18n 은 후속.
4. **경고 아이콘 상태바 표시**: 모달 닫은 후 상태바에 아이콘으로 지속 표시하는 기능 없음. 모달 놓치면 사용자가 경고를 다시 보기 어려움.

## 7. 다음 단계 (Stage 4)

**통합 검증 + 문서화**:
- `samples/hwpx/hwpx-02.hwpx` (작업지시자 제공) 회귀 테스트
- 대형 실문서 4건 false positive 측정 + 문서화
- `mydocs/tech/hwpx_lineseg_validation.md` 기술문서 작성
- 최종 결과 보고서 `mydocs/report/task_m100_177_report.md`

## 8. 승인 요청

본 Stage 3 단계별 완료보고서 검토 후 승인 시 Stage 4 착수.

### WASM 빌드 후 수동 검증

지금 WASM 빌드가 완료되었으므로 작업지시자가 다음을 확인할 수 있습니다:

1. rhwp-studio 에서 HWPX 파일 열기 (예: `samples/hwpx/hwpx-02.hwpx`)
2. 문서 로드 직후 **경고 모달 팝업** 확인 (경고 있을 경우)
3. Enter 키 또는 [자동 보정] 클릭 → 보정 후 재렌더 확인
4. 브라우저 콘솔에서 `[validation] ... warnings` · `[validation] user choice: auto-fix` · `[validation] reflowed N paragraphs` 로그 확인

모달이 예상대로 동작하지 않거나 false positive 가 보이면 Stage 4 에서 규칙 조정 가능합니다.
