# PR #1169 검토 — chore: repo hygiene + wasm api 분리 (실질: HWPX fragment paste 3차 재제출)

## 1. PR 정보

| 항목 | 내용 |
|------|------|
| 번호 | #1169 |
| 제목 | chore: harden repo hygiene and split wasm api modules |
| 작성자 | [@dragonnite1221-lgtm](https://github.com/dragonnite1221-lgtm) (Lee doyun) |
| **base ← head** | **`main`** ← `public/hwpx-fragment-paste` ⚠️ (devel 아닌 main 직접 타겟) |
| 상태 | OPEN, **CONFLICTING / DIRTY** |
| 변경 | +4957 / -1046, 64 파일 |
| 라벨 | enhancement / v1.0.0 |

## 2. 컨트리뷰터 의도 (정확 파악)

**제목은 chore 지만, 실질 의도는 "HWPX fragment paste 기능 추가"의 3차 재제출**이다.

근거:
- 브랜치명 `public/hwpx-fragment-paste`
- 첫 커밋 `8d25cb82 feat: add HWPX fragment paste support` (이후 chore/refactor 가 뒤따름)
- 누적 이력: **#880 → #950 → #1169** 모두 동일 기능. #880/#950 둘 다 CLOSED.

커밋 순서:
```
8d25cb82 feat: add HWPX fragment paste support   ← 핵심 의도
bebda1fa fix: address HWPX fragment paste review feedback
b7324c8e chore: harden local security hygiene
f4756ab0 chore: remove unmaintained paste dependency
8e2b4d6d chore: refresh npm audit lockfiles
7811fbc6 test: satisfy clippy all-targets warnings
1bb00472 refactor: split wasm api modules
```

→ 컨트리뷰터는 두 번 미머지된 기능을 **이번엔 repo 위생 chore + wasm_api 리팩터를 함께 묶어** 재제출. 제목을 chore 로 둔 것은 부수 정리를 전면에 내세운 것으로, 진짜 목표는 fragment paste 기능 통과.

## 3. 이전 close 사유 (검토 기준)

### #880 (2026-05-17 close)
리뷰 피드백 무응답 → close. "코드 품질·인프라 설계 좋음."

### #950 (2026-05-21 close) — 메인테이너 hands-on 테스트 후
**발견된 동작 결함 2건**:
1. 다이얼로그 확인 버튼 눌러도 안 닫힘 (`yangsik-parts-dialog.ts:101` `onConfirm` false 반환)
2. 빈 HWPX 삽입 실패 — `after_para_idx 1 out of range` (frontend `pos.paragraphIndex=1` vs Rust 0-based semantic 불일치)

**재제출 시 필수**:
- (A) 빈 HWPX end-to-end 동작 테스트
- (B) 확인 버튼 UX 재설계
- (C) `cross_document_migrate.rs` (1011줄) 별도 PR 분리
- (D) 비공개 task 참조 주석 제거 (`task_local_yangsik_paste_*`)

**권고**:
- (E) `yangsik` 명명 일관화 (form-snippet 또는 seosik-jogak)
- 프론트엔드 기능용 의존성 추가는 거버넌스에 반함

## 4. #950 필수/권고 항목 대응 현황 (#1169 점검)

| 항목 | 상태 | 근거 |
|------|------|------|
| (A) 빈 HWPX e2e 테스트 | ✅ 추가 | `tests/fragment_paste_in_document.rs` (`saved/04-blank_hwpx_empty.hwpx`, after_para_idx=0 0-based, inserted=1 검증) |
| (B) 확인 버튼 UX 재설계 | ❌ **미해결** | `yangsik-parts-dialog.ts:101 onConfirm() { return false; }` 그대로 — 확인 버튼 여전히 무동작 |
| (C) cross_document_migrate.rs 별도 PR 분리 | ❌ **미분리** | `src/document_core/commands/cross_document_migrate.rs` 여전히 포함 |
| (D) 비공개 task 참조 제거 | ❌ **잔존** | `fragment_paste_in_document.rs:133/330`, `mod.rs:123` — `task_local_yangsik_paste_composed_refresh` (RCA 2026-04-28 일자 포함) |
| (E) yangsik 명명 일관화 | ❌ **미반영** | `yangsik-fragments/`, `YangsikPartsDialog`, `yangsik-parts-dialog.ts` 그대로 |
| 의존성 제거 (`paste`) | ✅ 일부 | `f4756ab0 chore: remove unmaintained paste dependency` |

→ **빈 HWPX 코드 결함(2)은 0-based 테스트로 보강됐으나, 필수 (B)·(C)·(D) 3건과 권고 (E)가 미해결.**

## 5. 추가 우려

- **base=main**: 다른 PR 은 devel 대상. main 직접 타겟은 release 브랜치를 거치는 정상 흐름과 다름 → devel 로 재타겟 필요.
- **규모 +4957/-1046 64파일**: chore(위생) + refactor(wasm_api 분리) + feat(fragment paste)가 한 PR 에 혼재. CLAUDE.md "기능/포맷 분리" + `feedback_small_batch_release_strategy`(작은 단위 회전) 위배. #950 의 "cross_document 별도 PR" 권고와 같은 맥락.
- CONFLICTING (devel/main 최신 미반영).

## 6. 판단 (잠정)

기능·인프라 설계는 #880/#950 검토에서 이미 호평. 그러나:
- **#950 필수 항목 (B)(C)(D) 미해결** — 동일 결함/잔존으로 재제출됨.
- base=main 오타겟 + 64파일 혼재(chore+refactor+feat) → 거버넌스/작은 단위 회전 정책 위배.

→ **현 상태 머지 부적합.** 처리 방향은 작업지시자 판단 필요 (아래 선택지).

## 7. 작업지시자 확인 필요 사항

1. base main→devel 재타겟 요청 여부
2. (B)(C)(D)(E) 미해결 항목 — 재요청 후 보류 vs 일부 메인테이너 직접 보완 vs close
3. 64파일 혼재 — chore/refactor/feat 분리 요청 여부 (작은 단위 회전 정책)

---

## 8. 처리 결과 (보고)

- **결정: close + 재작업 요청** (작업지시자 승인). #880/#950 패턴 일관.
- 한글 코멘트 등록 (1 base 재타겟 / 2 #950 필수항목 B·C·D + 권고 E 미반영 / 3 64파일 chore·refactor·feat 분리 요청). issue-4575183281
- 빈 HWPX 통합 테스트 추가는 확인·인정.
- 재제출 가이드: (a) repo 위생 / (b) wasm_api refactor / (c) fragment paste(필수항목 반영) 3개 독립 PR, base=devel.
