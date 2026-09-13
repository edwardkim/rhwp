# #3587 Stage 11 — C1 고정 양식과 범위 선택

- 승인: 메인테이너 「다음 절차 진행을 승인합니다」, 2026-09-13.
- 기준: `0fb093c4d`, `task_m100_3587`.
- 계획: [C 상세계획](../plans/task_m100_3587_impl_c.md).
- 상태: 고정 양식 native 채우기·이름/셀 좌표 선택 구현과 집중 검증 완료. C 종료 통합 판정은 아니다.

고정 양식의 `fill_template_native`는 명시한 section/문단 범위에서 내용을 변경한다.
문단 추가나 ID 재발급 없이 대상 소유 문단만 staging하며 clipboard는 사용하지 않는다.
소유 경로·범위 검사와 뒤에서 앞으로 값 쓰기·대상 시작 서식 상속은 복사본 채우기와 공유한다.
반영 전 오류 검사를 마친 뒤 원본에 대입하고 한 번의 `TemplateFilled` 이벤트를 남긴다.
교체된 소유 트리의 표 재계산 표시도 새 포인터로 이관한다.

이름/셀 좌표는 읽기 전용 편의 API가 명시 경로 target으로 해석한다.
이름 occurrence는 지정 범위 안의 0 기반 소유 트리 순서이며, 생략 시 유일한 이름만 허용한다.
셀은 표 소유 경로·실제 행/열 앵커·셀 내부 문단·문자 범위를 지정한다.
병합에 덮인 좌표, 겹친 셀, 없는 문단을 첫 매치로 대체하지 않는다.
반환 경로는 해당 입력의 주소이므로 편집 요청에서 다시 검증하며 영구 ID로 취급하지 않는다.

초기 고정 양식도 B의 지원 가능한 닫힌 소유 트리·참조 검사를 보수적으로 공유한다.
내부 1회 검사 예산을 사용하는 것이 실제 복사/삽입을 뜻하지 않으며 공개 요청에는 copy count나
삽입 위치를 넣지 않는다. 값이 없는 binding/record 쌍은 유효 scope 검사 후 무변경 성공이다.
고정 양식의 빈 요청은 B의 count=0과 달리 scope 지원 검사도 수행한다.

검증 범위: 집중 기존 100건과 고정 양식 계약, 기존 필드 계약, Rust lint 3종.
실물 저장 검사는 메모리 저장·재열기이며 한컴 시각 판정이 아니다.
C2/C3, D/Gym, #7065, 원격 push·PR·close는 이번 범위 밖이다.

## 검증 기록

- 첫 구현 `53f456016`: 106건 중 63건 실행, 62 PASS / 1 FAIL, fail-fast로 43건 미실행.
  합성 병합 셀의 `row_span`을 명시하지 않아 default 0인 입력이 만들어졌고, 유효한 앵커가
  없어 `fillCell` 오류가 발생했다. `contracts.log`에 보존했다.
  이것은 제품에서 정상 문서를 거부했다는 증거가 아니며 합성 fixture 구성 오류다.
  테스트의 `row_span=1`을 명시했으며 제품의 잘못된 좌표 거부 규칙을 약화하지 않았다.
- `ac72838fc`: 고정 양식 8건 + 기존 100건 = **108 PASS**, 비대상 1,617 skipped.
  `contracts-final.log`에 저장했다. 두 번째 실행은 `--no-fail-fast`로 전 대상 결과를 확인했다.
- 셀/글상자 내부를 채웠을 때 바깥 문단의 저장된 control line을 본문 폭으로 다시 계산하지 않도록
  본문 직접 텍스트 편집에만 `reflow_paragraph`를 적용한다. 소유 표의 재계산 표시는 별도로 전달한다.
- review worktree에서만 준비한 문단의 대입을 생략한 음성 대조: **8건 중 4 FAIL / 4 PASS**,
  exit 100 (`negative-no-commit.log`). 본문·필드·글상자·실물 셀 값 누락을 검출했다.
  거부/무변경 및 기존 복사 API 계약은 통과했다. 검증용 변이는 작업 브랜치에 적용하거나
  커밋하지 않았으며 `ac72838fc`와 동일한 원본으로 복원 후 `git diff --exit-code`를 확인했다.

집중 실행은 기존 review worktree `/home/edward/mygithub/rhwp-review-3587`에서
`resolveCase`로 `issue_3587_*` 13개 원본의 suite를 해석하여
`cargo nextest run --locked --test <suite들> -E 'test(issue_3587_)' --no-fail-fast
--cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review`로 수행했다.
음성 대조는 같은 명령에서 `issue_3587_template_form::` 8건만 실행했다.

증적 로그는 로컬 `output/3587/c1-form/` 아래에 보존한다.
integration 원본은 `tests/cases/issue_3587_template_form.rs`이며 generated suite/manifest는 제출하지 않는다.

## 최종 검증과 남은 절차

제품·테스트 SHA는 `ac72838fc`다. 음성 대조 복원 후에도 같은 SHA의 source와 byte-identical임을
확인했다. 이후 commit은 계획·결과 기록만 변경한다.

| 검사 | 결과 | 로그 |
| --- | --- | --- |
| #3587 집중 계약 | 108 PASS | `contracts-final.log` |
| 값 반영 생략 음성 대조 | 4 FAIL / 4 PASS, 예상한 값 누락 검출 | `negative-no-commit.log` |
| 원본 복원 후 고정 양식 계약 | 8 PASS | `restored-green.log` |
| 기존 필드 계약 | rhwp 14 PASS, 다른 member는 필터로 0건 | `field-legacy.log` |
| fmt check | PASS | `fmt.log` |
| native Clippy | PASS | `clippy-native.log` |
| WASM lib Clippy | PASS | `clippy-wasm.log` |
| workspace build | PASS | `workspace-build.log` |
| workspace/all-target Clippy | PASS | `clippy-workspace.log` |
| integration manifest | PASS, 1,276 source / 48 target | `manifest.log` |
| 변경 문서 5개 링크·diff check | PASS | `docs-links.log` 및 Git 검사 |

Cargo 명령은 고정 target에서 순차 실행했다. Rust lint는 AGENTS.md의 세 Clippy와 workspace build를
각각 실행했으며, 기존 필드 검사는 `cargo test --locked --profile release-test --lib field_query::
--target-dir /home/edward/mygithub/rhwp/target/pr-review`다.
nextest 0.9.137/권장 0.9.140 및 해당 버전의 JUnit 설정 경고는 기존대로 남는다.

다음은 승인된 C 계획의 **C2 표 내부 행 복제·채우기**다. 이어서 C3 공개 API와 실행 경로,
동일 staging 준비를 사용하는 dry-run, 전체 nextest·Native Skia·Docker WASM·실물 시각 판정과
C 비용 계측을 진행한다. 이번 집중 검사를 C 전체 회귀 또는 Studio/한컴 시각 통과로 확대하지 않는다.
WASM 배포 파일과 dev 서버는 갱신하지 않았고 원격 push·PR·댓글도 하지 않았다.

## 파일 내보내기 후 메인테이너 판정과 별도 결함

후속 요청으로 같은 값·대상을 실제 파일로 저장했다.

- `output/3587/c1-form/labnote-001-stage11-filled.hwp`
- `output/3587/c1-form/labnote-001-stage11-filled.hwpx`

메인테이너는 **HWP/HWPX 모두 한컴편집기와 rhwp-studio에서 정상 열림**을 확인했다.
이는 해당 두 산출물의 열림 판정이며, 모든 렌더링 속성이나 C 전체 통합 검증 통과를 뜻하지 않는다.

동시에 rhwp-studio에서 삽입한 `😀`의 너비가 좁게 잡혀 가로로 압축되어 보이는 문제를 관측했다.
지시에 따라 선행 검색 후 [별도 이슈 #7084](https://github.com/edwardkim/rhwp/issues/7084)로 등록했다
(`bug`, `rhwp-studio`, `rendering`, `font`; milestone `v1.0.0`).
이슈 본문·metadata를 게시 후 재조회했으며 한글 본문 일치와 BOM 없음을 확인했다.
재현 파일 주소·해시·값·소유 경로와 후속 조사 항목을 남겼다.
회귀 여부와 원인은 미확정이며, #3587의 채우기/저장 결함으로 단정하거나 이번 범위에 추가하지 않는다.
Studio에 실제 로드된 WASM SHA는 생성 코드 SHA와 별도로 후속 조사에서 확인한다.
