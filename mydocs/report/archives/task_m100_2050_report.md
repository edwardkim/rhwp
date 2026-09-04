# 최종 결과보고서 — task_m100_2050

- **이슈**: #2050 [vscode] 뷰어 좌측 사이드바 — 페이지 썸네일 + 목차(개요) + 북마크 바로가기
- **브랜치**: `local/task2050` (분기 기준: `upstream/devel`)
- **마일스톤**: M100 (v1.0.0)
- **PR 전략**: 단일 PR / 목차 기본 모드 `auto`

## 1. 목표

PDF 뷰어 확장처럼 rhwp-vscode 뷰어 좌측에 네비게이션 사이드바(썸네일/목차/북마크)를 추가하고, 클릭으로 해당 위치 페이지로 이동한다. 작업 중 추가 요청으로 **하단 쪽번호 클릭 이동**도 포함했다. (2쪽 보기는 별도 이슈 #2051로 분리)

## 2. 구현 요약

### Stage 1 — Rust: `getStructure` WASM 바인딩
- `src/document_core/queries/structure.rs`에 `get_structure_native(mode)` 추가 (기존 `build_structure` 재사용).
- `src/wasm_api.rs`에 `#[wasm_bindgen(js_name = getStructure)]` 노출.
- `cargo build`/`cargo test --lib structure` 통과.

### Stage 2 — WASM 재빌드
- **Docker 미사용**(작업지시자 지시), 네이티브 `wasm-pack build --target web`로 pkg 재생성.
- `getStructure(mode: string): string`가 `pkg/rhwp.d.ts`에 노출됨을 확인.
- `pkg/`는 gitignore 대상(빌드 산출물) → 커밋하지 않음. 메인테이너는 Rust 소스에서 재빌드.

### Stage 3 — vscode: 사이드바 레이아웃 + 썸네일
- `hwp-editor-provider.ts`: `#scroll-container`를 `#app-shell`(flex row)로 감싸 `#nav-sidebar` 추가. **편집 영역 내부 좌표계 불변** → 스크롤/줌 회귀 방지. CSS 접두어 `nav-` 신설.
- `viewer.ts`: 썸네일 IntersectionObserver 지연 렌더, 클릭 이동, 현재 페이지 강조. 탭 전환, 상태바 토글.

### Stage 4 — vscode: 목차 + 북마크 + 쪽번호 이동
- 목차: `getStructure("auto")` 트리 렌더, 노드 클릭 → `getPageOfPosition` → 페이지 이동.
- 북마크: `getBookmarks()` 목록, 클릭 이동.
- 하단 쪽번호(`#stb-page`) 클릭 → 인라인 입력 → 페이지 점프.

## 3. 변경 파일

- Rust: `src/document_core/queries/structure.rs`, `src/wasm_api.rs`
- vscode: `rhwp-vscode/src/hwp-editor-provider.ts`, `rhwp-vscode/src/webview/viewer.ts`
- (커밋 제외 산출물: `pkg/`, `rhwp-vscode/dist/`, `*.vsix` — 모두 gitignore)

## 4. 검증

- ✅ `cargo build` / `cargo test --lib structure` 통과
- ✅ 네이티브 WASM 빌드 성공, `getStructure` pkg 노출
- ✅ `npm run compile`(webpack) 성공, 타입 체크 통과
- ✅ 산출 번들이 `getStructure`/`getBookmarks`/`getPageOfPosition` 참조
- ✅ vsix 패키징 성공(`rhwp-vscode-0.7.17.vsix`, 9.94MB)
- ⏳ **macOS 실기기 시각 판정 — 작업지시자 확인 필요** (썸네일/목차/북마크/쪽번호 클릭 이동, 사이드바 접기, 기존 스크롤·줌 회귀 없음)

### 시각 판정 방법
```bash
code --install-extension rhwp-vscode/rhwp-vscode-0.7.17.vsix
```
또는 VSCode → 확장 → "Install from VSIX..." → HWP 문서 열어 좌측 사이드바 확인.

## 5. WASM 빌드 트러블슈팅 기록

WASM 재빌드 중 **다른 Claude 세션이 동시에 Docker 빌드를 실행**하여 공유 `target/wasm32`·`pkg`가 반복 오염됨(크레이트 링크 실패, deps 디렉터리 삭제 등 flip-flop). 근본 원인 규명 후 다른 세션 Docker 빌드 종료 + 네이티브 단독 빌드로 해결. 디스크 여유가 빠듯해(`target/wasm32` ~33GiB) `CARGO_PROFILE_RELEASE_DEBUG=false`로 빌드(기능 동일, wasm-opt가 디버그 정보 제거).

## 6. 리스크 / 회귀

- 사이드바는 `#scroll-container` 바깥 래퍼에만 추가 → 기존 스크롤/줌 좌표계 불변.
- 파서/렌더/레이아웃 무변경 (읽기 전용 질의 노출만).
- 목차 `auto` 모드가 빈 문서에서는 "목차 정보가 없습니다" 안내.

## 7. 상태

구현·빌드·패키징 완료. macOS 실기기 시각 판정 후 단일 PR merge(작업지시자 승인). 이슈 클로즈는 판정 통과 후.
