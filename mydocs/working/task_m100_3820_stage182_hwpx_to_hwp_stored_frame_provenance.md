# #3820 Stage 182 - HWPX-to-HWP stored-frame provenance

## 목적

Stage 181에서 확인한 HWPX 원본 383쪽 → 저장 HWP 재로드 381쪽 불일치를 해결한다.

Stage 181의 최종 게이트에서 원본 HWP/HWPX 383쪽은 통과했지만 #3930 저장 HWP가
381쪽으로 실패했다. 이 실패 증거는 별도 문서가 아니라 본 단계의 provenance
수정 근거로 포함한다.

## provenance 계약

`Document::layout_profile()`의 정의를 확인했다.

| 계보 | `native_hwp5_layout` | `hwpx_stored_layout` | `hwpx_container` |
| --- | --- | --- | --- |
| 원본 native HWP5 | true | false | false |
| 원본 HWPX | false | true | true |
| rhwp HWPX-to-HWP 변환본 | false | true | false |
| HWP5-origin marker HWPX | false | false | true |

직접 HWPX `RowBreak` stored-frame 및 single-visible-cell source-frame은 XML 컨테이너
자체가 아니라 **HWPX stored pagination 계보**의 계약이다. `hwpx_container()`로
한정하면 저장 HWP 재로드가 이 계약을 잃는다. 반대로 `hwpx_stored_layout()`은
native HWP5와 HWP5-origin marker HWPX를 제외하므로 두 renderer 계약을 섞지 않는다.

## 수정

다음 두 파일만 수정했다.

- `src/renderer/layout/table_layout.rs`
  - 직접 HWPX RowBreak cell의 stored frame rewind 인식 기준을
    `hwpx_container()`에서 `hwpx_stored_layout()`으로 전환했다.
- `src/renderer/typeset.rs`
  - HWPX stored source frame cut, terminal response frame, single-visible source-frame
    fast-path의 동일한 provenance 기준을 `hwpx_stored_layout()`으로 전환했다.

이 수정은 table 크기·행 번호·문단 수·고정 pixel allowance를 추가하지 않는다.

## 검증

```text
cargo test --profile release-test --target-dir target/task-3820-stage168 \
  --test issue_3820_rowbreak_rowspan_band
PASS: 4 passed

cargo test --profile release-test --target-dir target/task-3820-stage168 \
  --test issue_3930_hwpx_hwp_save_layout
PASS: 3 passed
```

#3930의 핵심 `issue_3930_preserves_page_count_and_inherited_even_master_page`가
통과했다. 원본 HWPX와 `export_hwp_with_adapter()` 뒤 재로드 HWP의 page count 및
지정 page render-tree owner가 동일함을 확인한다.

## 최종 배포 산출물 게이트

```text
wasm-pack build --target web --out-dir pkg
PASS

cd rhwp-studio && npm run e2e:issue-3820
PASS: hwp=383, hwp p285 owner
PASS: hwpx=383, hwpx p144 owner
```

새 WASM `pkg`로 실행한 headless Studio에서 두 실제 편람의 383쪽 계약과 지정
render-tree owner를 모두 확인했다.

## 상태

provenance 결함 수정과 최종 배포 산출물 검증을 완료했다.
