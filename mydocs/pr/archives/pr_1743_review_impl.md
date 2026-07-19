# PR #1743 처리 계획 — #1692 SO-SUEOP HWP3/HWPX 렌더링 정합 보정

## 대상

- PR: #1743
- 작성자: @jangster77
- 관련 이슈: #1689, #1692, #1693, #1694, #1695, #1696, #1697, #1698, #1699
- 문서 작성 시점 PR head: `8e2f89ae9c721de18881551ed114efc6eea2e6d0`
- 처리 판단: GitHub Actions 통과 후 admin merge 완료
- merge commit: `f50aa4ef7a011817d8ae0ae0e41b817d42f4b030`

## 커밋

1. `7393203034a92fdbb241812458b539753eb01985`
   - `task 1692: HWP3 글자색 보존`
   - HWP3 글자색 보존, 샘플/기본 회귀 테스트 추가
2. `74b78f26ecdc9da5e3d4d93d3124c6ca6e4091e5`
   - `task 1692: SO-SUEOP 시각 검증 재분석`
   - HWP/HWPX/PDF 기준 페이지 단위 시각 차이 재분류
3. `d9c7bdfd8cf2145044704c1fe1293a35aed74f61`
   - `task 1692: 하단 빈 문단 페이지 밀림 보정`
   - vpos reset 직전 빈 문단으로 인한 페이지 증가 완화
4. `1a8a88fe791734c9e9a1d8aad6f3903fd980e82e`
   - `task 1692: SO-SUEOP HWP3 line box와 미주 흐름 보정`
   - HWP3 line box 폭, 미주 흐름, Outline 복원 보정
5. `6d70ad462001ad3094651e7ef9957ba70ca77a4a`
   - `task 1692: SO-SUEOP 미주 페이지 경계 보정`
   - 미주 페이지 경계와 후속 흐름 보정
6. `8d49615c3aefd31a7f6bfd027002a8042506e73b`
   - `task 1692: SO-SUEOP 머리말과 HMapsi OLE 보정`
   - 머리말과 OLE preview fallback 보정
7. `16656f5a6d90c085376e368fec3291cded4e9b72`
   - `task 1692: SO-SUEOP HWP3 외부 이미지와 글상자 흐름 보정`
   - HWP3 외부 이미지 경로와 글상자 흐름 보정
8. `a09c321d616ed84d5a042dd8085b4f697e26298f`
   - `task 1692: SO-SUEOP 잔여 도식 차이 분석`
   - p22 관계도 잔여 차이 분석
9. `336746fe66ef9685eb13f573f899c0ab306b0f24`
   - `task 1692: SO-SUEOP p22 관계도 선문자 복원`
   - 관계도 선문자와 원문자 표시 보정
10. `9e789e3e8236b34c38e9ba4495d834e9a8e13fc5`
    - `task 1692: SO-SUEOP HWP3 머리말 선 복원`
    - HWP3 머리말 밑줄 복원
11. `d58a590a235a722eeb2fd7c7569a7703448e9fbc`
    - `task 1692: SO-SUEOP p22 본문 기호 복원`
    - p22 본문 기호와 원문자 표시 복원
12. `3c608e8994d687ef18e4ab03b3300796b8628cb7`
    - `task 1692: SO-SUEOP p22 미주 표지 위치 보정`
    - p22 미주 번호 표지 위치와 스타일 보정
13. `aa8b47c333287c7d1112f6256fa0a7d67a7ede6a`
    - `task 1692: HWP3 p22 하드코딩 제거`
    - 문서명/페이지 전용 fixup 제거, 일반 경로로 재구성
14. `8e2f89ae9c721de18881551ed114efc6eea2e6d0`
    - `task 1692: SO-SUEOP 미주 구분선 흐름 보정`
    - 미주 separator line, prefix style, HWP/HWPX 흐름 정합 보정

## 검토 단계

### Stage 1. PR 메타 확인

- base branch: `devel`
- draft: false
- mergeable: `MERGEABLE` (작성 시점 참고값)
- mergeStateStatus: `CLEAN` (작성 시점 참고값)
- 규모: 41 files, +3329/-307

### Stage 2. 변경 내용 검토

완료.

- HWP3 파서 내부에서 HWP3 전용 보정을 처리하는 원칙을 유지했다.
- 공통 렌더러 변경은 HWP3 전용 분기가 아니라 empty item, note separator, pagination 흐름처럼 공통 모델 의미에 맞는
  범위로 제한했다.
- p22 하드코딩 함수는 제거했고, 선문자/원문자/미주 prefix 처리는 일반 경로로 옮겼다.
- 샘플 HWP/HWPX/PDF와 `tests/issue_1692.rs`가 함께 추가되어 회귀 검증 경로가 남아 있다.

### Stage 3. 로컬 검증

완료.

- `cargo build`: 통과
- `cargo test issue_1692 --test issue_1692 -- --nocapture`: 통과
- `cargo test issue_1293_clean_visual_sweep_targets_keep_page_counts_and_shape_profiles --test issue_1139_inline_picture_duplicate -- --nocapture`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --all-targets`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `git diff --check`: 통과

### Stage 4. 시각 검증

완료.

- `mydocs/manual/verification/visual_sweep_guide.md` 기준 sweep 사용
- HWP3/HWPX/PDF 모두 SO-SUEOP 46쪽 확인
- p22 관계도/미주 표지/선문자 확인
- p43~p46 미주 범위 확인
- p45 하단 footer overlap 재발 없음

### Stage 5. GitHub Actions 확인

문서 작성 시점에는 다음 상태다.

- preflight, Render Diff, Canvas visual diff, CodeQL 계열 대부분 통과
- `Analyze (rust)`: success
- `Build & Test`: success

최종 merge 전 PR head와 merge state를 다시 확인한다.

## merge 전 필수 조건

1. PR #1743 최신 head가 `8e2f89ae9c721de18881551ed114efc6eea2e6d0` 또는 그 이후 의도한 head인지 확인
2. GitHub Actions required checks 전체 통과 확인 완료
3. 작업지시자 merge 승인 확인
4. #1695와 #1699 후속 정책 확인

## merge 후 후속 처리

`mydocs/manual/pr_review_workflow.md` 기준으로 처리한다.

1. merge 직전 최신 GitHub Actions와 head SHA 재확인: 완료
2. PR #1743 admin merge: 완료, `f50aa4ef7a011817d8ae0ae0e41b817d42f4b030`
3. `upstream/devel` fetch: 완료
4. 자동 close 여부 확인: #1692 포함 관련 이슈 모두 open 상태였음
5. #1692/#1693/#1694/#1696/#1697/#1698 수동 close 완료
6. #1695는 실제 close 여부 확인 결과 open 유지
7. #1699는 font fallback 후속으로 open 유지
8. #1689 parent 이슈는 하위 이슈 상태를 본 뒤 별도 판단
9. PR 후속 코멘트와 오늘할일 갱신은 문서-only PR로 반영

## 수동 close 결과

- #1692: https://github.com/edwardkim/rhwp/issues/1692#issuecomment-4864806076
- #1693: https://github.com/edwardkim/rhwp/issues/1693#issuecomment-4864806208
- #1694: https://github.com/edwardkim/rhwp/issues/1694#issuecomment-4864806357
- #1696: https://github.com/edwardkim/rhwp/issues/1696#issuecomment-4864806486
- #1697: https://github.com/edwardkim/rhwp/issues/1697#issuecomment-4864806638
- #1698: https://github.com/edwardkim/rhwp/issues/1698#issuecomment-4864806850

close 시 최초 코멘트의 브랜치명 표기가 shell quoting 문제로 깨져, 위 정정 코멘트를 추가로 남겼다.

## open 유지 판단

- #1695
  - #1743에서 페이지 수 46쪽, p43~p46 미주 범위, p45 footer 겹침은 해결됐다.
  - 하지만 #1695 본문 확인 기준은 일반 본문 6/23/28쪽과 미주 42/44/47쪽의 원본 LINE_SEG vpos reset/rewind가
    페이지 또는 단 경계 힌트인지 검증하고, 반영 가능한 최소 규칙을 정의하는 것이다.
  - 현재 회귀 테스트는 미주 내부 vpos=0 normalize와 페이지/미주 범위를 고정하지만, LINE_SEG reset/rewind 일반
    규칙 검증까지 포함하지 않는다.
  - 따라서 #1695는 부분 반영으로 보고 open 유지한다.
- #1699
  - #1743은 구조적 레이아웃 차이를 우선 해소했다.
  - #1699의 확인 기준인 로컬 폰트 설치 여부, fallback 결과, `--font-style`, `--embed-fonts`, `--font-path`
    옵션별 차이 비교는 아직 별도 검증하지 않았다.
  - 따라서 #1699는 font fallback 후속 검증 이슈로 open 유지한다.

## 후속 코멘트 요지

- PR #1743은 초기 HWP3 글색상 보존에서 시작했지만, 최종 head 기준 SO-SUEOP HWP3/HWPX 구조적 렌더링 정합
  보정으로 범위가 확장됐다.
- 로컬 검증은 `clippy --all-targets`, `test --all-targets`, `release-test --tests`, issue 전용 테스트, 시각 sweep까지 통과했다.
- SO-SUEOP 기준 PDF/HWP/HWPX 모두 46쪽이며, p22 관계도와 p43~p46 미주 흐름을 확인했다.
- PR #1743은 `f50aa4ef7a011817d8ae0ae0e41b817d42f4b030`으로 merge 완료됐다.
- #1692/#1693/#1694/#1696/#1697/#1698은 수동 close 완료됐다.
- #1695 LINE_SEG reset/rewind 일반 규칙과 #1699 폰트 fallback은 별도 후속으로 남긴다.
