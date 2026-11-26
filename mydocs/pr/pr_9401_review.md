# PR #9401 리뷰 — 중첩된 표와 글상자의 문단 나누기·합치기

## 최종 판정

**메인테이너 보완본 수용 가능.** 실제 Studio에서 수식 뒤의 `B|OX`를 나누고 합친 다음 실행 취소하면 `B`와 `OX`가 원래 문단에 돌아왔다. 다시 실행하고 같은 자리에 입력한 K도 HWP·HWPX 각각 두 번 저장하고 다시 열 때까지 보존됐다. 원 기여와 보완을 함께 담은 통합 PR에서 CI를 확인한다. [최종 검증](assets/pblsketch-20261126/README.md)

## 접수와 변경 범위

[원 PR #9401](https://github.com/edwardkim/rhwp/pull/9401)의 작성자는 pblsketch이고 대상은 `devel`이다. 연결 이슈는 [#9398](https://github.com/edwardkim/rhwp/issues/9398)다. 접수할 때 리뷰 요청이 비어 있어 2026-11-26에 `edwardkim`을 지정하고 다시 조회했다. 원 커밋은 `eb826cfc233c26dca86e083dd6c43b9e9d92196c`이며 [제품 CI](https://github.com/edwardkim/rhwp/actions/runs/45538521657)와 [정책 검사](https://github.com/edwardkim/rhwp/actions/runs/45542230976)가 성공했다.

검토 기준은 2026-11-26의 `upstream/devel` `48a678399072d0df11380e1ad6a3b6aa77dd677a`이다. 두 원 커밋을 번호순으로 병합한 로컬 소스는 `2647cad077ca350659b15863df30bad6e01eeca4`다. 원 PR 두 건은 서로의 커밋에 의존하지 않는다. 각 원 커밋과 현재 기준의 병합 시뮬레이션, 실제 누적 병합, 공백 검사와 Rust 테스트 자동 수집 검사를 통과했다.

## 검토 대상

`src/document_core/commands/text_editing.rs`에서 표 칸과 묶음 글상자의 문단 목록을 같은 경로로 찾도록 변경했다. 최신 기준은 표 칸 안 첫째·둘째 글상자와 글상자 안 표 칸에서 문단을 나눌 때 모두 경로 오류를 반환했다. 검토는 실제 Enter·Backspace, 실행 취소·다시 실행, HWP와 HWPX 저장·재열기, 이웃 문단과 인라인 개체의 보존을 포함한다.

- `tests/fixtures/group_textbox_cell/cell-group-textbox.hwp`
- `samples/issue8513/textbox_floating_table_offset.hwpx`
- `mydocs/pr/assets/pr9424/core-export-inline-equation.hwpx`의 인라인 수식을 사용한 대조 입력

## 메인테이너 보완

원 기여로 묶음의 두 글상자와 글상자 안의 표 칸에서 Enter·Backspace를 사용할 수 있게 됐다. 수식 다음의 `B|OX`를 나눴다가 합치면 원 기여는 앞 문단의 문자 수 1을 병합 위치로 돌려줬다. 실제 커서는 수식 한 자리와 `B` 한 자리를 지나 위치 2에 있었다. 이 차이로 실행 취소 후 `B`가 뒤 문단으로 이동했다. [보완 전 실제 측정](assets/pblsketch-20261126/native-undo.tsv)

`c59c1893f0`은 Rust 병합에서 편집기가 사용하는 논리 길이를 반환하도록 했다. Studio는 해당 위치와 복원할 문단 정보를 함께 보관한다. 일반 입력·삭제·실행 취소 때도 중첩 칸의 논리 위치를 문단의 문자 범위로 환산한다. 기존 문자 인덱스 API는 같은 입력을 계속 받아들이며, 새 WASM API와 공개 바인딩은 커서 좌표를 받는다. [재현 절차](pblsketch_20261126_review_impl.md)
