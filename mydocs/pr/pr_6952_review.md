# PR #6952 검토 기록

## 발견 사항과 최종 판정

**최종 판정: 머지 보류.** #6940과의 충돌은 로컬에서 해소했지만, 그 보정이 포함된 통합 후보의 회귀·Clippy·실물 왕복/시각 검증은 아직 실행하지 않았다. 원 head CI 성공으로 충돌 해소본의 검증을 대체하지 않는다.

### 기존 #6940의 명시적 빈 접미 보존을 유지해야 한다

최신 devel에는 #6940이 이미 반영되어 `ON_PAGE`/`ON_SECTION`과 `deco_chars_from_source`에 따른 빈 접미 보존이 존재한다. #6952의 오래된 base는 suffix fallback을 `USER_CHAR` 여부만으로 선택하므로, 원 PR 쪽을 통째로 선택하면 숫자 형식의 명시적 `suffixChar=""`를 다시 `)`로 바꾸는 회귀를 만들 수 있다.

`src/serializer/hwpx/section.rs`의 두 충돌을 다음 원칙으로 해소했다.

- 이미 적용된 ON_* 토큰은 유지하고 중복 주석 충돌만 제거했다.
- `shape.deco_chars_from_source || shape.number_format == NumberFormat::UserChar`이면 빈 fallback을 사용한다.
- 그 외 미설정 숫자 형식은 기존 `)` fallback을 유지한다. 실제 suffix 문자가 있으면 note_deco_char_attr가 그 문자를 방출한다.
- 새 `decoration_is_user_char`와 인라인 `USER_CHAR ↔ 18` 처리는 보존했다.

이것은 충돌 해소/호환 보정에 대한 정적 설명이다. 보정 코드가 테스트로 확인됐다는 주장이 아니다.

### 시험 범위의 공백

[추가 시험 6개](../../tests/cases/issue_6872_hwpx_footnote_autonum_roundtrip.rs#L72)는 합성 IR을 직렬화하는 계약이다. 원 HWPX의 userChar를 읽는 parser, 미주 Endnote 경로, 인라인 AutoNumber의 USER_CHAR parse/serialize 왕복은 직접 다루지 않는다. 따라서 "네 필드 모두 왕복 검증"을 이 6개 시험만으로 입증할 수 없다.

수용 전에는 명시적 빈 DIGIT/USER_CHAR 접미와 기존 #2742 계약, footNote/endNote의 userChar, 인라인 autoNum 5개 슬롯을 함께 확인해야 한다. 추가적인 확정 런타임 결함은 현재 읽은 변경 범위에서 발견하지 못했지만, 이 공백과 통합 검증 누락은 남아 있다.

## 대상과 적용

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#6952](https://github.com/edwardkim/rhwp/pull/6952), planet6897 |
| 관련 이슈 | [#6872](https://github.com/edwardkim/rhwp/issues/6872), [#6941](https://github.com/edwardkim/rhwp/issues/6941) |
| base / 규모 | devel, 4파일, +271/-7, 1 commit |
| 원 head | `4529c2a0c0104a2783b44a5b798a7badce628042` |
| 검토 기준 | upstream/devel `92f6242af96f51ede912588fa6fe35f5447709bc` |
| 검토 branch | `review/planet6897-6949-6952-20260909` |
| 출처 보존 적용/누적 candidate | `d01f7989b25e5ecff7a6a42a04b1d901ea44c1d6` |
| 충돌 | src/serializer/hwpx/section.rs 2개 구간, 기존 #6940 규칙과 새 USER_CHAR 처리를 함께 유지 |
| reviewer | jangster77 지정 완료 |
| 원격 상태 참고값 | 2026-09-09, 원 PR CONFLICTING / DIRTY. 로컬 해소가 원 PR branch를 갱신한 것은 아님 |

기본 경로는 maintainer 일반 경로이며 intake, multi-PR, local validation, visual fixture, post-merge 지침을 적용했다. PR 본문·commit·변경 파일·시험을 읽었고 #6872의 기존 #6940 수용 범위와 #6941 본문을 대조했다. 원 PR의 일반/inline/review 코멘트는 조회 시점에 없었다.

## CI와 로컬 실행 구분

- 원 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/34345926899)에서 Build & Test, A/B/C/D 기본 회귀, lint, Native Skia가 성공했다. WASM Build 등 정책 skip은 실행 성공으로 세지 않는다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34345926849) 분석, [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34345926557), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34345926883), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34345926925)가 성공했다. CodeQL aggregate는 NEUTRAL, CI Impact Policy는 SUCCESS다.
- 이 결과는 원 head에 귀속되며 이미 #6940이 들어간 최신 devel과 충돌 해소본의 검증은 아니다. 이번 로컬 후보에서 build/회귀/Clippy/새 렌더를 실행하지 않았다.
- 이번 단계에서 원 head를 수정하거나 원격 push/PR 생성/merge/issue close를 수행하지 않았다. 수용 조건은 충돌 해소본 검증, 최신 통합 head CI와 작업지시자 승인이다.

## 실물과 시각 증적 경계

대상 원본은 기존 #6940 검토와 같은 `156584446` 및 `156513948` HWPX다. 기준은 기존 `pdf/pr6940-156584446-source-2020.pdf`, `pdf/pr6940-156513948-source-2020.pdf`를 우선 재사용한다. 이번에는 PDF를 다시 출력하거나 새 증적 파일을 복제하지 않았다.

기존 [#6940 review](archives/pr_6940_review.md)의 68쪽 텍스트/대표 2쪽 raster 일치는 이전 candidate 결과다. #6952 보정본의 결과로 재사용하지 않는다. #6941은 해당 문서의 인라인 USER_CHAR 손실이 한컴 PDF에 나타나지 않았다고 보고하므로, 그림만 같아도 성공으로 판단하지 않고 section3.xml의 인라인 autoNum 5개 type/userChar 및 note 속성을 직접 대조해야 한다.

## Merge 후 contributor PR comment 계획

- 정본: [Visual Sweep GitHub merge comment](../manual/verification/visual_sweep_guide.md#github-merge-comment).
- 현재 통합 후보의 실물 왕복·visual sweep은 미실행이다. #6940의 예전 100% 수치나 PNG를 #6952의 새 증적처럼 게시하지 않는다.
- 검증 뒤에는 번호 토큰, footNote/endNote 속성 이름, 인라인 autoNum 5개 슬롯, 접미 보존의 실제 XML 결과와 대표 PDF 판독을 함께 기록한다. 직접 생성한 대표 PNG만 안정 경로에 보존하고 중간 파일/로그는 output에 둔다.
- 그 결과에 따라 #6872와 #6941의 수용/close 범위를 각각 판단한다. 현재 두 이슈는 close하지 않았다.
- 이후 실제 merge와 asset의 devel 포함, 최신 CI 및 승인이 충족된 뒤에만 merge SHA 고정 이미지와 실제 수치를 `--body-file`로 게시한다. 현재는 승인 코멘트를 작성할 선행 증적이 부족하다.
