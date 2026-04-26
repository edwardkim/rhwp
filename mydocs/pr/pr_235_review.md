# PR #235: 표 셀 긴 숫자 텍스트 겹침/셀 폭 미사용 해결 — 수행 계획서

## PR 정보

| 항목 | 내용 |
|------|------|
| PR 번호 | [#235](https://github.com/edwardkim/rhwp/pull/235) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| 관련 이슈 | [#229](https://github.com/edwardkim/rhwp/issues/229) |
| base → head | **devel ← planet6897:devel** (Fork 기반) |
| 변경 | +1,990 / -725 (31 파일, 16 커밋) |
| Mergeable | ✅ **clean** |

## 수행 목표

PR #235를 검토·검증하여 `devel`에 merge한다.

## 해결 대상 문제 (Issue #229)

표 셀 내 긴 숫자 텍스트가:
1. **음수 자간(letter_spacing)으로 인한 글자 겹침**
2. **셀 폭을 충분히 활용하지 못하는 underflow 현상**

## PR 변경 범위 (작성자 설명)

1. **오버플로우 케이스 자간 압축** 적용
2. **narrow glyph 역진 방지**
3. **셀 underflow 시 자간 확장**
4. `form-002` 골든 SVG 재생성 (분기 추가로 좌표 변경 반영)

## 검증 상태 (작성자 체크)

- [x] `cargo test` 통과 (svg_snapshot 3 passed)
- [ ] `cargo clippy -- -D warnings` 통과 — **미확인**
- [x] 샘플 파일 SVG 내보내기 확인
- [ ] 웹(WASM) 렌더링 확인 — **미확인**

## 검토 항목

1. **코드 품질**: 이상적 구현인지 확인
2. **Clippy 경고**: -D warnings 통과 여부 직접 검증
3. **회귀 테스트**: 기존 893+ 테스트 영향 없음 확인
4. **E2E 테스트**: 표 렌더링 관련 E2E 통과 확인
5. **문서**: `mydocs/report/task229/` 첨부된 비교 이미지 확인

## 브랜치

`local/pr235` (검토 전용 브랜치)

## 구현 계획서

`mydocs/plans/task_m100_pr235_impl.md` 참조

## 예상 단계: 4단계
