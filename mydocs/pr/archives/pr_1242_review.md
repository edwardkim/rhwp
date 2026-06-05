# PR #1242 검토 — HWPX masterpage idRef 연결 보강

- **작성일**: 2026-06-02
- **PR**: #1242 (OPEN)
- **제목**: `Task #1201: HWPX masterpage idRef 연결 보강`
- **컨트리뷰터**: @postmelee
- **연결 이슈**: #1201
- **base/head**: `devel` ← `task-1201-hwpx-masterpage`
- **Head SHA**: `658a0308ae29405e031dcc96b7e71ecd24e2f352`
- **현재 local/devel**: `a9c49dd6`
- **규모**: 10 files, +1105 / -31, 1 commit
- **GitHub 상태**: `MERGEABLE`, `BEHIND`
- **CI**: PR head 기준 `Build & Test`, `CodeQL` 통과. `WASM Build`는 skip.
- **PR 댓글/리뷰**: 없음

## 1. PR 요약

PR #1242는 HWPX 문서의 바탕쪽(masterpage)을 manifest 순서 추정이 아니라 section XML의 명시적 참조로 연결하도록 보강한다.

핵심 흐름:

```text
section XML의 <hp:masterPage idRef="...">
-> content.hpf manifest item id
-> masterpage href
-> Contents/masterpage*.xml
-> SectionDef.master_pages
```

기존 구현은 `content.hpf`의 manifest 순서로 section별 masterpage 묶음을 추정했다. 이 방식은 샘플에 따라 우연히 맞을 수 있지만, manifest 순서와 section 참조 순서가 달라지면 홀짝 바탕쪽이 뒤집히거나 다른 section의 바탕쪽이 붙을 수 있다.

## 2. 주요 변경 범위

| 파일 | 변경 |
|---|---|
| `src/parser/hwpx/content.rs` | `PackageInfo.master_page_items` 추가, masterpage manifest id/href 보존 |
| `src/parser/hwpx/mod.rs` | section `idRef`를 manifest id로 resolve해 masterpage XML 연결, 실패 시 기존 fallback 유지 |
| `src/parser/hwpx/section.rs` | section XML의 `masterPage@idRef` 수집 helper 추가, `masterPage@type` 표기 정규화 |
| `mydocs/plans/task_m100_1201*.md` | 계획/구현 계획 문서 |
| `mydocs/working/task_m100_1201_stage*.md` | 단계별 검증 문서 |
| `mydocs/report/task_m100_1201_report.md` | 완료 보고서 |

## 3. 타당한 부분

### 3.1 HWPX와 HWP5의 규칙을 분리한다

HWP5 raw parser는 기존대로 `Both, Odd, Even` 순서 기반 LIST_HEADER 해석을 유지한다.

이번 PR은 HWPX XML parser에만 적용되며, HWPX에서는 명시적 `idRef`와 `masterPage@type`을 우선한다. 형식별 규칙을 섞지 않는 방향이라 타당하다.

### 3.2 fallback을 유지해 기존 샘플 위험을 낮춘다

`idRef` 연결이 하나도 성공하지 못한 경우에만 기존 `section_master_page_files` fallback을 사용한다.

따라서 기존 manifest 순서 기반으로 우연히 동작하던 문서도 곧바로 깨질 가능성은 낮다.

### 3.3 type 정규화가 실제 샘플과 공식 표기를 모두 커버한다

`EVEN`/`Even`/`even`, `ODD`/`Odd`/`odd`, `LAST_PAGE`/`LastPage`/`lastPage`, `OPTIONAL_PAGE`/`OptionalPage`/`optionalPage`를 같은 의미로 처리한다.

HWPX XML 생성기별 대소문자 차이를 흡수하는 보강이다.

## 4. 주의 사항

### 4.1 PR은 현재 devel보다 뒤처져 있다

GitHub 상태는 `BEHIND`이다. 다만 `git merge-tree HEAD pr/1242` 기준 충돌은 없다.

현재 `local/devel`에서 검증 브랜치를 만들어 병합하면 될 것으로 보인다.

### 4.2 원본 재현 샘플이 저장소에 없다

PR 본문과 이슈 #1201은 대상 샘플을 다음 파일로 설명한다.

```text
[2027] 온새미로 1 본교재.hwpx
[2027] 온새미로 1 본교재.pdf
```

하지만 PR 커밋에는 원본 HWPX/PDF가 포함되어 있지 않다. 따라서 PR 작성자가 기록한 PDF 4~7쪽 기준 시각 검증은 현재 저장소만으로는 재현할 수 없다.

자동 테스트와 구조 테스트는 가능하지만, 최종 시각 판정은 다음 중 하나가 필요하다.

```text
1. 메인테이너가 대상 샘플을 로컬에 확보해 직접 판정
2. 컨트리뷰터에게 샘플 첨부 요청
3. 저장소 내 다른 HWPX masterpage 샘플로 제한적 가드 판정
```

### 4.3 idRef 부분 성공 케이스

현재 설계는 일부 idRef만 연결되고 일부는 누락된 경우 fallback을 섞지 않는다.

이는 중복/오연결 방지 측면에서는 적절하지만, 손상된 manifest 문서에서는 일부 바탕쪽이 빠질 수 있다. warning이 출력되므로 진단 가능성은 남아 있다.

## 5. 권장 검증

현재 `local/devel` 기준 검증 브랜치를 만들고 PR #1242를 병합한 뒤 다음을 실행한다.

```text
git diff --check HEAD
cargo fmt --all --check
cargo test --lib test_parse_content_hpf_master_pages_by_manifest_order
cargo test --lib test_collect_hwpx_section_master_page_refs
cargo test --lib test_resolve_master_page_hrefs_uses_id_ref_order_and_dedups
cargo test --lib test_parse_hwpx_master_page_type_accepts_official_and_sample_spellings
cargo test --lib test_parse_master_page_mixed_case_type_attrs
cargo test --lib hwpx
cargo test --test issue_1100_exam_social_hwpx_header
cargo test --test issue_1113_header_autonum_placeholder
cargo test --test hwpx_roundtrip_integration
docker compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
```

시각 판정은 샘플 확보 상태에 따라 진행한다.

대상 샘플이 확보된 경우:

```text
target/debug/rhwp export-svg "[2027] 온새미로 1 본교재.hwpx" -p 3 -o output/poc/pr1242-masterpage-idref/page4
target/debug/rhwp export-svg "[2027] 온새미로 1 본교재.hwpx" -p 4 -o output/poc/pr1242-masterpage-idref/page5
target/debug/rhwp export-svg "[2027] 온새미로 1 본교재.hwpx" -p 5 -o output/poc/pr1242-masterpage-idref/page6
target/debug/rhwp export-svg "[2027] 온새미로 1 본교재.hwpx" -p 6 -o output/poc/pr1242-masterpage-idref/page7
```

확인 포인트:

```text
PDF 4/6쪽: 짝수쪽 바탕쪽
PDF 5/7쪽: 홀수쪽 바탕쪽
rhwp SVG에서 홀짝이 반전되지 않는지 확인
```

## 6. 권장 처리

권장안: **수용 후보로 진행한다.**

근거:

- 변경 범위가 HWPX parser에 제한되어 있다.
- HWP5 raw parser 순서 규칙을 변경하지 않는다.
- 기존 fallback을 유지한다.
- 연결 모델이 HWPX XML의 명시적 `idRef`/`type` 의미와 맞다.
- PR head CI는 통과했다.

단, 원본 재현 샘플이 커밋에 없으므로 최종 완료 게이트는 다음 중 하나를 선택해야 한다.

```text
권장: 검증 브랜치에서 자동 테스트 + WASM build 후, 메인테이너가 대상 샘플로 시각 판정
대안: 샘플이 없으면 컨트리뷰터에게 샘플 첨부 요청 후 PR 완료 보류
```

## 7. 다음 승인 요청

다음 단계로 진행하려면 작업지시자 승인이 필요하다.

권장 절차:

```text
1. `local/pr1242-verify` 브랜치를 현재 `local/devel`에서 생성
2. PR #1242를 병합
3. HWPX masterpage 관련 단위/통합 테스트 실행
4. WASM/Studio 빌드
5. 대상 샘플 확보 상태에 따라 SVG 산출 및 메인테이너 시각 판정
6. 판정 통과 후 local/devel 반영
```
