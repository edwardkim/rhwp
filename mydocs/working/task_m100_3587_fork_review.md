# #3587 포크 구현 사전 검토 — f40c12f0

- Issue: [#3587](https://github.com/edwardkim/rhwp/issues/3587)
- 수행계획: [task_m100_3587.md](../plans/task_m100_3587.md)
- 조사일: 2026-09-12
- 포크: yuyu04/rhwp, `feat/hop-agent-native-ops`
- 확정 SHA: `f40c12f0f542cbf309898093183706a0379768c2`
- parent: `b3e16ef212af81ef37d973ddb86d6816d3804642`, Cargo version `0.7.13`
- 현행 대조: `59a11f180ad1bd5cadcbbf0a6dc9d0162f4a0a21`
- 범위: 메인테이너가 요청한 구현 전 소스·테스트 정적 검토. Stage 1 전체 완료가 아니다.
- 실행: GitHub API로 SHA 확인, 해당 commit을 Git object로 fetch하여 `git show`/`git diff`로 읽었다.
  작업 브랜치는 `task_m100_3587`로 유지했다. 포크 checkout·cherry-pick·빌드·테스트·한컴 실행은 하지 않았다.

## 1. 결론

포크는 기존 복사·붙여넣기를 이용하는 양식 자동화에서 부족했던 native 편집 연산과,
특정 연구노트 양식에 맞춘 사후 정리·서식 정책을 함께 추가한다.
템플릿 데이터 바인딩·반복 생성의 전체 실행기를 새로 구현한 commit은 아니다.
**실제 사용에서 발견한 요구와 실패 단서는 재사용하되, 이 commit 전체를 범용 엔진 기능으로
그대로 이식할 수 있다고 판단하지 않는다.**

후속 설계의 우선순위는 다음과 같다.

1. 기존 붙여넣기의 문단 생성·분할 계약과 자동화 반복 블록의 경계 계약 대조.
2. 복제 개체 ID·스타일·리소스 참조의 범위와 갱신 책임 확인.
3. 위 계약을 사용하는 생성·복제·채우기 공통 흐름 설계.
4. 필요한 후처리만 명시적 대상 범위로 제한. 2pt·9pt·60%·6×3 등의 업무 정책은 공통 동작과 분리.

## 2. 실제 변경 범위

[확정 commit](https://github.com/yuyu04/rhwp/commit/f40c12f0f542cbf309898093183706a0379768c2)은
7개 파일, 1,681줄 추가·14줄 삭제다. MIT 라이선스이며 코드 재사용 시 기존 고지와 기여 출처를 유지한다.

| 파일 | 변경 |
| --- | --- |
| `src/document_core/commands/text_editing.rs` | 문단 정리 3종, 구역 병합, 셀/글상자/캡션 vpos 누적 보정, 테스트 3개 |
| `src/document_core/commands/table_ops.rs` | 목차 분할·쪽번호 갱신, 원본 양식 표 제거, set-table 공개 및 treatAsChar 저장 보정 |
| `src/document_core/commands/object_ops.rs` | 이미지 크기 해석·그림 모서리 보정, 셀 내부 그림/표 삽입 |
| `src/model/paragraph.rs` | 텍스트 삭제 후 같은 시작 위치의 char-shape 참조 중복 제거 |
| `src/wasm_api.rs` | 신규 WASM 바인딩 6종 |
| `rhwp-studio/src/core/wasm-bridge.ts` | 위 6종의 호출 래퍼 |
| `src/wasm_api/tests.rs` | 그림 2개, 원본 표 제거/끝 정리 1개, 뒤 내용 보존 1개: 테스트 4개 |

새 native 연산은 정리 3 + 구역 병합 1 + 원본 표 제거 1 + 목차 2 + 그림 보정 1 + 셀 삽입 2로
**10종**이다. set-table 공개와 기존 경로 수정은 별도다.
Discussion의 “9종”, commit 설명의 “테스트 6개”와 실제 diff의 개수는 다르다.
이 검토에서 센 신규 `#[test]`는 **7개**이며 실행 통과 수가 아니다.

WASM/Studio에 새로 노출한 것은 `removeOrphanParasBeforePageBreaks`,
`trimTrailingParasAfterLastTable`, `compactLeadingParasBeforeTables`, `removeSourceFormTable`,
`createTableInCell`, `insertPictureInCell`이다. 이 commit의 WASM 파일과 Studio에서
구역 병합·목차 2종·그림 보정의 대응 래퍼는 찾지 못했다. 모든 native 연산이 WASM에 노출되었다는
commit 요약을 그대로 지원 범위로 사용하면 안 된다.

## 3. 복사·붙여넣기와 빈 문단의 관계

이 commit은 `clipboard.rs`를 수정하지 않는다. 기반 구현의
[`paste_control_native`](https://github.com/yuyu04/rhwp/blob/f40c12f0f542cbf309898093183706a0379768c2/src/document_core/commands/clipboard.rs#L677)는:

- clipboard의 첫 컨트롤 문단을 clone한다.
- 대상 문단이 비었는지, 문자 위치가 0인지에 따라 교체·앞 삽입·문단 분할 후 삽입을 선택한다.
- 삽입 컨트롤 바로 뒤에 **빈 문단을 항상 추가**한다.
- 호출 결과로 문단 인덱스·컨트롤 인덱스를 반환한다.

따라서 “복제 뒤 생긴 빈 문단을 지운다”는 제안에는 기반 API의 동작과 연결되는 소스 근거가 있다.
현행 `src/document_core/commands/clipboard.rs::paste_control_native`에도 빈 문단 추가가 남아 있다.
반면 현행에는 #2299의 신규 문단 vpos 처리, 복제 표의 reflow 상태 상속, 그림/도형의
반복 붙여넣기 위치 이동 등이 있어 과거 함수 전체로 되돌려서는 안 된다.

**빈 문단 추가 자체를 현재 버그로 확정하지 않는다.** 대화형 편집에서 필요한 후속 커서 문단과
자동화가 원형 블록을 반복하는 경우의 계약을 대조하고, 실제 템플릿으로 필요/불필요를 재현해야 한다.
사용 앱의 전체 호출 순서는 이 diff만으로 알 수 없다. 원본 표 삭제/목차/정리 연산의 실행 순서를
소스 주석으로 추정해 확정하지 않는다.

## 4. 컨트롤 ID와 참조 — 범용화 전에 검증할 지점

### 4.1 중첩 표 생성의 ID 계산

[`create_table_in_cell_native`](https://github.com/yuyu04/rhwp/blob/f40c12f0f542cbf309898093183706a0379768c2/src/document_core/commands/object_ops.rs#L2096)는
raw CommonObjAttr의 INSTANCE_ID를 다음 입력만으로 계산한다.

```text
0x7c160000 + rows × 0x1000 + cols × 0x100 + total_width (wrapping u32)
```

동일 행·열·너비이면 **이 함수가 raw에 쓰는 ID 값은 동일하다**. 문서 내 기존 개체와의
충돌 검사나 별도 발급기는 이 계산에 없다. `common`은 일부 필드와 Default로 따로 구성한다.
이 사실은 소스에서 확인한 것이며, 저장기의 최종 재번호 부여 여부나 한컴에서 나타날 증상까지
실행으로 확인한 것은 아니다. IR/raw/저장 단계의 최종 ID를 추적할 우선 후보로 삼는다.

### 4.2 clone이 어디까지 복사하는가

[`rechunk_toc_table_native`](https://github.com/yuyu04/rhwp/blob/f40c12f0f542cbf309898093183706a0379768c2/src/document_core/commands/table_ops.rs#L1176)는
표와 **그 표가 들어 있는 문단 전체**를 clone한 뒤 선택한 표 슬롯만 교체한다.
따라서 원형 문단에 다른 텍스트·컨트롤이 있으면 그것도 청크마다 복제하는 구조다.
이 루프에는 개체 ID 재발급 호출이 없다. 반복 제목행의 내부 컨트롤·필드까지 포함한 참조 정합성은
확인되지 않았다. 동일 문서 내 스타일 ID 공유 자체를 오류로 규정하지 않는다.

기반 `copy_control_native`/`paste_control_native` 역시 clone 경로이며 포크에서 개선하지 않았다.
현행 문서 간 붙여넣기 `foreign_paste.rs`에는 스타일·필드·리소스 재매핑과
`rebuild_resolved_styles()`가 이미 있다. 그 경로를 내부 컨트롤 paste가 자동으로 사용한다고
가정하지 않고 공통화 가능성과 검증 범위를 따로 조사한다.

### 4.3 스타일·리소스 생성 정책

- 압축은 첫 글자모양을 복제해 2pt 스타일을 매번 추가한다. 같은 대상의 두 번째 호출도
  대상에서 제외하지 않으므로, 제안된 “두 번째는 0건” 계약과 정적 코드가 맞지 않는다.
- 중첩 표는 9pt 비볼드 글자모양을 추가하고, line_height=900·line_spacing=90을 직접 지정한다.
  테두리는 네 면이 실선이고 두께가 1 이상인 첫 BorderFill을 재사용한다. 색상·배경 등 전체
  스타일 동등성을 확인하는 조건은 아니므로 임의 템플릿의 서식 정책으로 일반화할 수 없다.
- 그림 BinData ID는 `bin_data_content.len() as u16 + 1`이다. 실제 ID 집합의 빈 번호·기존 최대값·
  포화 조건을 확인하는 발급 계약이 아니다. 허용 문서에서 충돌하는지는 별도 입력 검증이 필요하다.
- 새 스타일 추가 후 캐시가 언제 재해석되는지, raw 참조가 최종 저장에서 어떤 값을 쓰는지는
  소스 경로와 실행을 함께 확인해야 한다. 이번 검토에서는 저장 손상을 재현하지 않았다.

## 5. 후처리와 앱 정책의 구분

| 포크 동작 | 코드에서 확인한 점 | 설계에 반영할 사항 |
| --- | --- | --- |
| 고아/꼬리 문단 정리 | Table/Picture/Shape만 배제한 “빈 문단” 판정. 구역 전체를 순회 | 수식·필드·제어 구조와 자동화 대상 밖의 기존 빈 문단 보호 |
| 표 앞 압축 | 2pt, 줄높이 200·baseline 170 강제 | 문단 경계 문제를 스타일 축소로 숨기지 않고 명시적 업무 서식과 구분 |
| 원본 양식 표 제거 | 다음 문단이 Page이면 내용 유무 검사 없이 문단 전체 삭제. 첫 위치에서는 다음 표를 앵커로 이동 | 쪽나눔 속성과 삭제 가능한 임시 문단을 구별. 내용 있는 다음 문단 반례 필요 |
| 구역 병합 | 용지 크기·상하좌우 여백만 비교. 뒤 구역 첫 문단의 구역/단/머리말/꼬리말 컨트롤 제거 | 호환성 검사와 제거 정책을 명시. 동일 인덱스 재호출은 멱등 연산이 아님 |
| 목차 번호 갱신 | 6행×3열 표를 항목으로 간주. 목차 행 수와 항목 수의 min만 처리 | 형태가 아닌 명시적 대상·대응으로 일반화. 불일치·부분 성공 보고 |
| 목차 분할 | 첫 행=제목행, 지정 행 수로 분할, 이후 문단 Page, 표 자체 분할 None | 병합 셀·다중 제목행·실제 높이를 보장하지 않음. 데이터 블록과 페이지 정책 분리 |
| 그림 삽입 | 셀 높이 60%와 최소 7500 HU로 상한 결정, 실제 비율로 높이 재계산 | 비율 유지와 임의 크기 제한을 분리하고 템플릿/호출자 정책으로 결정 |

특히 원본 표 제거 함수의 설명은 “쪽나누기 제거”지만 실제로는
[`delete_paragraph_native`](https://github.com/yuyu04/rhwp/blob/f40c12f0f542cbf309898093183706a0379768c2/src/document_core/commands/table_ops.rs#L1832)를 호출한다.
원래 앱이 빈 쪽나눔 문단만 전달했다면 목적에 맞을 수 있지만, 임의 템플릿에 같은 API를 노출할 때의
안전 계약은 별도다. 이것을 현행 devel의 재현된 손실 버그라고 보고하지 않는다.

## 6. 테스트 증거의 범위

추가된 7개 테스트는 압축 1, 목차 청크 행 수 1, 구역 병합 1, 그림 크기 2,
원본 표 제거와 꼬리 정리 1, 표 뒤 “맺음말” 보존 1이다.

- 그림 2건은 PNG 헤더만 만든 `fake_png_bytes`를 사용한다. 이미지 디코딩·한컴 출력의 유효성을
  입증하는 테스트가 아니라 내부 크기 계산 검사다.
- 원본 표 제거 검사는 표를 직접 clone해 문단 목록을 구성한다. 실제 copy/paste 공개 API의
  반복 호출 전체 흐름을 검사하지 않는다.
- 이 commit의 신규 테스트에는 멱등 재호출, 복제 ID 충돌, 스타일 참조 충돌, HWP/HWPX 저장 왕복,
  앞뒤 복합 문단의 완전 보존을 검증하는 항목이 없다.
- 기존 저장소 전체 테스트가 없다는 뜻은 아니다. 이번 변경의 전용 증거가 어디까지인지 구분한다.
- 실행하지 않았으므로 7 PASS나 한컴 호환 통과로 기록하지 않는다. 실측 92쪽/빈 페이지/손상 보고는
  제안자 주장으로 유지하며 대응 원본과 출력부터 확보한다.

## 7. 현행 devel과의 관계 및 다음 조사

`set_table_properties_native`는 현행에서 이미 public이며, 외부 스타일 재매핑·편집 후
raw 무효화·reflow 상태 관리도 과거와 달라졌다. #3576의 char-shape 중복 문제와 #3552의 저장
문제는 선행 upstream 처리 이력이 있다. 각 수정의 현재 계약을 확인하고 포크 보정과 중복 이식하지 않는다.

수행계획의 Stage 1은 다음 증거가 더 필요하다.

1. 사용 앱의 실제 호출 순서와 유효한 템플릿/생성 출력. 이 commit만으로 채우기 파이프라인을
   재구성했다고 주장하지 않는다.
2. 내부 paste·외부 paste·표/글상자 생성의 ID/raw/참조 소유권 표와 기존 테스트 재사용 목록.
3. 동일·다른 문서에서 반복 복제 후 개별 편집 및 HWP/HWPX 재열기 실험.
4. 대화형 paste의 빈 커서 문단을 유지하면서 자동화의 앞뒤 경계를 보장하는 공통 계약 결정.

작업 브랜치·사용자 소스·WASM 산출물은 변경하지 않았으며 이 문서와 수행계획만 갱신한다.
기여자에게 아직 comment를 게시하지 않았고 구현 승인도 받지 않았다.

## 8. 재현 명령

```bash
gh api repos/yuyu04/rhwp/commits/f40c12f0
gh api repos/yuyu04/rhwp/branches/feat/hop-agent-native-ops
git fetch --no-tags https://github.com/yuyu04/rhwp.git f40c12f0f542cbf309898093183706a0379768c2
git diff f40c12f0^ f40c12f0 --stat
git diff f40c12f0^ f40c12f0 -- src/document_core/commands/text_editing.rs
git diff f40c12f0^ f40c12f0 -- src/document_core/commands/table_ops.rs
git diff f40c12f0^ f40c12f0 -- src/document_core/commands/object_ops.rs
git diff f40c12f0^ f40c12f0 -- src/model/paragraph.rs src/wasm_api.rs src/wasm_api/tests.rs rhwp-studio/src/core/wasm-bridge.ts
git show f40c12f0:src/document_core/commands/clipboard.rs
```
