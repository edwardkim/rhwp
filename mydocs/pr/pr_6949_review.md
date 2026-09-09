# PR #6949 검토 기록

## 발견 사항과 최종 판정

**최종 판정: 머지 보류.** 원 head CI는 성공했지만 아래 두 소스 결함과 통합 후보의 직접 시각 검증 누락 때문에 수용하지 않는다. 아래 결함은 정적 제어 흐름 검토 결과이며 실행 재현을 완료했다는 뜻이 아니다.

### P1: 완성된 ROP 관용구를 확인하기 전에 DPA를 삭제한다

[drops_monochrome_mask](../../src/wmf/converter/svg/ternary_raster_operator.rs#L91)는 앞 PATINVERT로 만든 AwaitDpa 상태와 출력 요소 수만 확인한다. [observe의 DPA 분기](../../src/wmf/converter/svg/ternary_raster_operator.rs#L59)도 같은 조건에서 즉시 true를 반환해 중간 DPA를 삭제한다.

마지막 PATINVERT의 존재·브러시·좌표 일치는 나중에 확인하며, 그때 불일치하더라도 이미 삭제한 DPA를 복원하지 않는다. DPA 자체의 목적 사각형도 앞 PATINVERT와 대조하지 않는다. 따라서 `PATINVERT(A) → DPA(1bpp, B) → EOF`처럼 다른 영역의 독립된 DPA가 이어지거나, 동일 영역이라도 마지막 PATINVERT가 없는 입력에서 기존에 표시하던 패턴이 사라진다. 마지막 PATINVERT의 사각형/브러시만 달라져도 앞서 삭제한 출력은 돌아오지 않는다.

현재 음성 대조는 앞 PATINVERT가 전혀 없는 단독 DPA뿐이다. [합성 시험](../../tests/cases/issue_6865_wmf_monochrome_mask_rop.rs#L214)은 위 불완전·다른 영역의 두 연산 접두부를 다루지 않는다.

해제 조건: 세 연산과 같은 목적 영역/유효 DC 조건을 확인한 뒤 삭제를 확정하거나, 대기 중 DPA를 보존해 미완성/불일치 시 원래 출력을 복구한다. EOF, 다른 영역, 다른 마지막 브러시와 끼어드는 draw에 대한 음성 대조가 필요하다.

### P2: 1bpp만으로 흑백 마스크라고 판단한다

[brush_is_monochrome_mask](../../src/wmf/converter/svg/ternary_raster_operator.rs#L111)는 DIBPatternPT의 bit_count만 검사한다. 실제 팔레트 색, 색 테이블 사용 방식, 패턴 용도는 검사하지 않는다.

1bpp는 색 테이블의 두 색을 선택하는 형식이지 흑백 또는 마스크 전용 형식이 아니다. [MS-WMF BitCount 정본](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/792153f4-1e99-4ec8-93cf-d171a5f33903)은 0/1 비트가 각각 색 테이블의 첫째/둘째 색을 선택한다고 규정한다. 따라서 유효한 빨강/파랑 1bpp 패턴도 마스크로 분류되어 해당 DPA가 사라질 수 있다. "색 정보를 싣지 않는다"는 코드 주석도 이 조건에서는 성립하지 않는다.

해제 조건: 실제 확인한 흑백 마스크 범위로 좁히고, 유색 1bpp 및 별도의 흑백 그림 입력을 보존하는 음성 대조를 추가한다. 위 P1의 시퀀스 완결성도 함께 해결해야 한다.

## 대상과 적용

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#6949](https://github.com/edwardkim/rhwp/pull/6949), planet6897 |
| 관련 이슈 | [#6865](https://github.com/edwardkim/rhwp/issues/6865), Closes #6865 |
| base / 규모 | devel, 7파일, +343/-9, 1 commit |
| 원 head | `8fbfd865b1190fdf95b5a98d80467a53c944ce59` |
| 검토 기준 | upstream/devel `92f6242af96f51ede912588fa6fe35f5447709bc` |
| 검토 branch | `review/planet6897-6949-6952-20260909` |
| 출처 보존 적용 commit | `32554ff8bf193e4da64e7855854dcb3b80bffa64`, 충돌 없음 |
| 누적 code candidate | `d01f7989b25e5ecff7a6a42a04b1d901ea44c1d6` |
| reviewer | jangster77 지정 완료 |
| 원격 상태 참고값 | 2026-09-09, MERGEABLE / CLEAN; merge 전 최신 head 재확인 필요 |

기본 경로는 maintainer 일반 경로이며 intake, multi-PR, local validation, visual fixture, post-merge 지침을 적용했다. 원 PR 본문·commit·diff·기존 합성 시험·보고서·#6865 본문과 코멘트를 읽었다. 원 PR의 일반/inline/review 코멘트는 조회 시점에 없었다.

## CI와 로컬 실행 구분

- 원 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/34339873622)는 Build & Test, A/B/C/D 기본 회귀, lint 성공이다. Native Skia와 WASM Build는 SKIPPED였으며 통과로 세지 않는다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34339873660) 분석, [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34339873605), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34339873758)가 성공했다. CodeQL aggregate는 NEUTRAL, CI Impact Policy는 SUCCESS다. Render Diff run은 해당 head의 조회 결과에 없었다.
- PR 본문이 언급한 로컬 golden/flake 실패와 원격 최종 CI는 별개의 실행 결과다. 어느 쪽도 이번 통합 candidate의 실행 결과로 옮겨 적지 않는다.
- 이번 단계에서는 로컬 build, Cargo 회귀, Clippy, 새로운 렌더/visual sweep을 실행하지 않았다. 실행하지 않은 검증을 PASS로 기록하지 않는다. 발견 결함의 보정과 통합 후보 검증 승인이 다음 단계다.

## 원 PR 시각 자료 판독

원 PR에 포함된 [before p3](../report/6865-wmf-monochrome-mask-rop/before-p3.png), [after p3](../report/6865-wmf-monochrome-mask-rop/after-p3.png), [before p7](../report/6865-wmf-monochrome-mask-rop/before-p7.png), [after p7](../report/6865-wmf-monochrome-mask-rop/after-p7.png)를 실제 열었다.

3쪽 패널의 체커보드 감소와 7쪽 배너의 어두운 파란 면 노출은 제공 이미지에서 보인다. 그러나 after p3는 영역 이미지이며, 이 자료는 원 contributor가 만든 before/after다. 통합 candidate를 다시 렌더한 자료도, maintainer가 기준 한컴 PDF와 직접 대조한 review PNG도 아니다. 7쪽 영문 제목 주변의 얼룩도 남아 있어 전체 도해 fidelity 해결로 표현하지 않는다.

기여자는 패널 샘플값 96~98 → 217, 한컴 기준 245 및 21개 문서 비교를 보고했다. 이 수치와 전수 범위는 기여자의 측정이며 이번 세션에서 독립 재측정하지 않았다. 기존 fixture `samples/issue6469/wmf_fill_shapes.hwpx`와 원본 156627451을 직접 비교할 필요가 있다. 기준 PDF는 기존 자료와 pdfinfo를 우선 확인하고, 적합한 파일이 있으면 재출력하지 않는다.

## Merge 후 contributor PR comment 계획

- 정본: [Visual Sweep GitHub merge comment](../manual/verification/visual_sweep_guide.md#github-merge-comment).
- 현재는 `visual sweep 미실행`, `원 PR 증적만 확인`, `머지 보류`다. 새로운 flagged/pixel_match/proxy나 대표 review PNG URL을 만들어 쓰지 않는다.
- 위 P1/P2 보정과 음성 대조, 통합 head의 원 문서 3/7쪽 및 최소 fixture 직접 검증이 완료되면 실제 수치와 판독·남은 색상 차이를 기록하고 대표 PNG만 mydocs/pr/assets에 보관한다. 로그·SVG·중간 raster는 output에 두고 커밋하지 않는다.
- 향후 수용 시에도 #6865의 정확한 색상/하프톤 잔여가 해결됐는지 또는 별도 이슈로 분리됐는지 판단한 뒤 close 범위를 정한다. 현재 #6865 close나 GitHub approve/merge/comment는 수행하지 않았다.
- 실제 merge 뒤 asset의 devel 포함을 확인한 다음 merge SHA 고정 raw URL을 `--body-file`로 게시한다. 현재는 그 선행 조건이 충족되지 않았다.
