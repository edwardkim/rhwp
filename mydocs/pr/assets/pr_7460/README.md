# PR #7460 한컴 열기 검증 증적

2026-09-30 UTC에 postmelee가 실제 실행한 HWP5 저장·열기 검증입니다.
개인 자료를 포함하지 않는 합성 입력이며, 실제 한컴 생성 원본이나 조판 기준 PDF는 아닙니다.
검토 문서는 [PR #7460 review](../../archives/pr_7460_review.md)를 참조해 주세요.

## 입력과 동일성

다음 8개 파일은 생성에 사용한 HWPX 1개와 두 앱에서 실제로 연 HWP 7개입니다.
기존 source head에서 동일 Git blob이 없음을 확인하고 별도 검토 입력으로 보존했습니다.
코드의 자동 회귀 fixture 목록에는 추가하지 않았습니다. HWP 출력 버전은 모두 **5.1.0.0**입니다.

| 입력 | 바이트 | SHA-256 |
| --- | ---: | --- |
| [body.hwpx](inputs/body.hwpx) | 5915 | `8e0669972fb73051ba202f214148421dbf18850592373f4834b3a027f6bb4c0e` |
| [control.hwp](inputs/control.hwp) | 6144 | `e3190331c1570abb831563ea636e9e6902f7c0a857fde0e1c7dcac83ba3f0e28` |
| [before-header.hwp](inputs/before-header.hwp) | 6144 | `b2f9eb7c34e15f8b6cf6da68811888f267830ba4866cf01bcaab3db4bfd26320` |
| [after-header.hwp](inputs/after-header.hwp) | 6144 | `ac010ea6464a4f3b88c611713600fe63ab458fc0a353894d2a9ed8fceb7ec0b9` |
| [before-footer.hwp](inputs/before-footer.hwp) | 6144 | `6bf6b0249ff35270f01b5a568b2089c39083eff97d015cde0e6656b9f575c2a5` |
| [after-footer.hwp](inputs/after-footer.hwp) | 6144 | `c57c1559dd556f36a81b7978262dc401cf657524ce9facef2d36cdd9e59c6ba7` |
| [before-both-all.hwp](inputs/before-both-all.hwp) | 6144 | `df138507258a01e99e28ddabb4ef19b81ee6fbbcc4b4a2c5029ffa4fb77f312c` |
| [after-both-all.hwp](inputs/after-both-all.hwp) | 6144 | `9acaf4b6e1044b56b495e69b8952e07f5370111c06e8f80147f5944c15ccf154` |

한컴뷰어 **12.31.8 (6446)**와 Chrome **154.0.8037.58**에서 동일한 HWP를 열었습니다.
수정 전 세 파일은 Viewer에서 “파일이 손상되었습니다.”로 실패했습니다.
웹한글기안기의 수정 전 세 콜백은 `result:false`, `errorCode:4100`,
`errorMessage:"문서를 여는 동안 HWPSDK 오류가 발생했습니다."`,
`exception:"com.hancom.webhwp.control.exception.HwpSDKException: -2147467245"`였습니다.
대조군과 수정 후 세 파일은 두 앱에서 열렸고, 아래 두 문단을 화면에서 직접 확인했습니다.
Viewer 상태바는 **1/1쪽·55글자**, 웹 콜백은 `result:true`, 원본 파일명과 `size:6144`였습니다.
각 웹 시험 전에 페이지를 새로 로드했습니다.

```text
PR7460 BODY PRESERVED 20260930
빈 머리말과 꼬리말 저장 후 열기 검증입니다.
```

독립 CFB 판독으로 수정 전 HF 문단의 `nChars=0x80000000`, 22바이트 헤더와
수정 후 `nChars=0x80000001`, 24바이트 헤더를 확인했습니다.
각 전후 쌍은 Section0의 해당 문단 헤더만 달랐고, 다른 CFB stream과 본문 레코드는 동일했습니다.
한 쪽의 열기 성공을 여러 쪽의 짝수·홀수 표시 의미나 PDF 조판 일치의 증거로 사용하지 않습니다.

## 생성 출처와 재현 절차

수정 후 바이너리 source는 `7e47b085e5f744fa6bf632c84f68b47391342aed`입니다.
원 PR code head `bf50bc7df46ff9df566e407291fddad0155a9bd6`와 변경된 5개 파일이 바이트 단위로 같습니다.
수정 전은 같은 로컬 검토 source에 `b19eb36c48dcc58ed293b0fcf1989e879b52936d`의
`src/serializer/body_text.rs`와 `src/serializer/cfb_writer.rs` 두 파일만 복원한 음성 대조입니다.
전체 source가 과거 devel과 같다는 뜻은 아닙니다. 진단 후 두 파일을 정확히 복원했습니다.

| 생성 바이너리 | SHA-256 |
| --- | --- |
| 수정 전 대조 | `791376cf84a247d2886b7de9f6fef58c0e5706daa5d8309aa20c3a7153c4a739` |
| 수정 후 새 빌드 | `ad3489c4ecfd851cc2741a72ba1861742c442fdfc8a38e102e78eecd96b0b955` |

빌드 명령은 `cargo build --locked --profile release-test --bin rhwp --target-dir target/pr-review`입니다.
`scaffold`로 아래 spec의 HWPX를 만들고 HWP5로 `convert`한 파일이 `control.hwp`입니다.
보존된 `body.hwpx`부터도 재현할 수 있습니다. 파일 내용의 해시는 위 표를 기준으로 대조합니다.

```json
{
  "version": "1",
  "font": "함초롬바탕",
  "blocks": [
    {
      "type": "paragraph",
      "text": "PR7460 BODY PRESERVED 20260930"
    },
    {
      "type": "paragraph",
      "text": "빈 머리말과 꼬리말 저장 후 열기 검증입니다."
    }
  ]
}
```

아래는 각 생성 바이너리에서 실행한 공개 편집 명령의 경로를 단순화한 재현식입니다.
`RHWP_REVIEW_BIN`에는 위 수정 전 또는 수정 후 바이너리를 지정하고,
`RHWP_REVIEW_CASE_DIR`에는 복사한 입력을 보관하는 새 임시 폴더를 지정합니다.
보존 파일을 덮어쓰지 말고 같은 문서의 새 출력으로 실행해 주세요.

```sh
"$RHWP_REVIEW_BIN" edit insert-header-footer "$RHWP_REVIEW_CASE_DIR/control.hwp" --header --apply-to 0 -o "$RHWP_REVIEW_CASE_DIR/header.hwp" --json
"$RHWP_REVIEW_BIN" edit insert-header-footer "$RHWP_REVIEW_CASE_DIR/control.hwp" --footer --apply-to 0 -o "$RHWP_REVIEW_CASE_DIR/footer.hwp" --json
```

`both-all.hwp`는 control에서 시작해 **header 0 → header 1 → header 2 → footer 0 → footer 1 → footer 2**를
순서대로 삽입한 최종 출력입니다. 앞 단계 출력을 다음 단계 입력으로 전달했습니다.
생성 중간 파일·실행 바이너리·원시 로그는 이 공개 증적에 포함하지 않았습니다.

## 웹한글기안기 실행 방법

[한컴 공식 Open 계약](https://developer.hancom.com/webhwp/devguide/hwpctrl/methods/open)에 따라
[공식 Open 예제](https://webhwpctrl-example.cloud.hancom.com/run-example?title=Open&main=example.js&init=init.js&path=https://developer.hancom.com/webhwp-examples/hwpctrl/methods/open/)의
Script Editor에 아래 스크립트를 넣고 실행 버튼과 실제 파일 선택 창으로 보존 입력을 선택했습니다.
선택한 File/Blob을 `HwpCtrl.Open`에 전달했습니다. 화면 하단 결과 표시는 콜백 원문입니다.

```javascript
var picker = document.createElement("input");
picker.type = "file";
picker.accept = ".hwp";
picker.id = "pr7460-test-file";
picker.style.cssText = "position:fixed;top:80px;left:10px;z-index:99999;background:white";
document.body.appendChild(picker);
picker.onchange = function () {
  var file = picker.files[0];
  var output = document.createElement("pre");
  output.id = "pr7460-open-result";
  output.style.cssText = "position:fixed;bottom:10px;left:10px;z-index:99999;background:white;color:black;border:2px solid #333;padding:12px;max-width:90vw;white-space:pre-wrap";
  document.body.appendChild(output);
  output.textContent = "PR7460 OPEN " + file.name + " — opening";
  HwpCtrl.Open(file, "HWP", "", function (result) {
    output.textContent = "PR7460 OPEN " + file.name + "\n" + JSON.stringify(result);
    console.log("PR7460 OPEN", file.name, result);
  }, null);
};
picker.click();
```

## 스크린샷 편집과 출처

실제로 판독한 최종 스크린샷 **14개를 7개 비교 PNG**로 묶었습니다.
Viewer 수정 전은 오류 대화상자 전체입니다. 성공 화면은 제목·본문·상태바를 발췌해
빈 페이지 여백 때문에 글자가 너무 작아지지 않게 했습니다. 화면 내용의 문구를 바꾸지 않았습니다.
웹 화면은 1200×762 전체를 좌우에 그대로 배치했습니다. 비교 설명은 화면 밖에 추가했습니다.
원 캡처는 `.png`라는 임시 이름으로 반환됐지만 실제 인코딩은 JPEG였으며,
아래 비교 산출물은 실제 PNG로 저장했습니다.

Viewer 성공 캡처의 원본 좌표 `(left, top, right, bottom)`:
제목 `(1000,0,2050,90)`, 본문 `(820,570,2250,810)`,
상태바 `(0,height-65,950,height)`입니다. 각 발췌는 비율을 유지해 축소·배치했습니다.
Viewer control은 3008×1890, 세 성공 화면은 3024×1898입니다.

| 판독한 원 캡처 | SHA-256 |
| --- | --- |
| `viewer-before-header.png` | `5ed66b30b022f6c3f9f6e7b9756f0430e5682c79e9c379c83eab9293b4605aa6` |
| `viewer-after-header.png` | `c0dbfdd9cf5c47d8938f524b5a294c62b4933d96510b18d373d065754c9aae6e` |
| `web-before-header.png` | `821d2bffc5e90ab3e38b2021cb600a5a017a70d71797dd14a76699c95a5efd13` |
| `web-after-header.png` | `23ce05dfdd07d981df063763e9a6c05070dd2eabab824051708d8009cac370d5` |
| `viewer-before-footer.png` | `cced1c3b4701c3c3f7b3fe09f3bff82c2192b3706ae5f947a3f799708ba0ecbe` |
| `viewer-after-footer.png` | `349a6e75cb1f67c18538056d42761848b289fa483961d1778e1a5d6a992611a5` |
| `web-before-footer.png` | `bb0de7c9cc348ab1a248d3ad396ffb6f814b70a28f1dee4daec53f540098c248` |
| `web-after-footer.png` | `baf904ed25eb4a34fafdc728c323fffcd72d5e136ffaeb1db132df989423127c` |
| `viewer-before-both-all.png` | `09ebcff528bac24d8c3c79c5cbd6e6bd11bfeb6d5fb73b70de7d4499b16dd9cd` |
| `viewer-after-both-all.png` | `19218d2c25d1612d777f6ef1a4e1f59a6fcd7b87e01031511e793e9a198815d2` |
| `web-before-both-all.png` | `f0e52306c5149f9c44ed3b5bf3c42cba7b28de4576257e2b7977a907a82fd15a` |
| `web-after-both-all.png` | `93f8d57181a32036321a760e9752f2dde749f753c196e2b2db33c6c56cca2983` |
| `viewer-control.png` | `9b37cab52a87c6e80b90a9d138bb2f6e73eae31c78f05eb50d70dbd9321052c8` |
| `web-control.png` | `bf6a8cf7bfc50aa3f5dbfaabcd4fef694279f84605f8dcd1d3f7fab6f44ccf8f` |

| 공개 비교 PNG | SHA-256 |
| --- | --- |
| [viewer-header-comparison.png](viewer-header-comparison.png) | `8f89501afff30c111afbb576ccfeaa50a7937d53782f32c2100b8bccd5d5d96c` |
| [web-header-comparison.png](web-header-comparison.png) | `ed67f0e50c25ea92c590b58927f38af27741988e82942439622ecb654796246a` |
| [viewer-footer-comparison.png](viewer-footer-comparison.png) | `bf30586248323900e3cc1c20943b5b1d8e69974d0f25503b4f51e7525a3a344e` |
| [web-footer-comparison.png](web-footer-comparison.png) | `89f12e6a8e34735ae4ca6954ff72112f629a5dffbbcae92b26279c93e16d4690` |
| [viewer-both-all-comparison.png](viewer-both-all-comparison.png) | `f375b8149fa97bb44e6b8c140674f1b7cbcf8a7867c5352a4b245d48490db57a` |
| [web-both-all-comparison.png](web-both-all-comparison.png) | `b1cc8d393afa7948a648a17d30311161602384ba41acc9641c9cc399f9b1fe0b` |
| [control-comparison.png](control-comparison.png) | `ff81f53f24a51de4fcab455364bf33d344131f72c45cdbb8f59aa2b2138a72fe` |

인앱 브라우저는 파일 열기를 완료하지 못했고 Firefox 공식 예제는 Edge·Chrome 지원 안내로
차단됐습니다. 이 미실행 경로를 Chrome의 실제 성공과 구분했습니다.
실행 완료된 Viewer·Chrome 시험의 화면만 공개 비교 증적에 사용했습니다.

## 빈 문단 저장 축의 추가 메모리 계약

검토한 `actual_char_count`는 헤더 문자 수뿐 아니라 `line_segs_within_text`의 범위 필터에도
전달됩니다. 기존 #5563 계약은 축 끝과 같은 LineSeg를 보존하는 `>` 경계입니다.
수정 후 source `7e47b085e5f744fa6bf632c84f68b47391342aed`의 새 빌드 library를 링크한
아래 검사로 실제 PARA_HEADER와 PARA_LINE_SEG 레코드를 판독했습니다.
파일 입력 없이 메모리에서 생성·소비한 합성 계약 검사이며 한컴 원본·PDF 기준은 아닙니다.
수정 전 FAIL 실행을 하지 않았으므로 결함 검출 증거로 세지 않습니다.

```text
PASS input=[] nChars=1 lineSegs=[]
PASS input=[0] nChars=1 lineSegs=[0]
PASS input=[0, 1] nChars=1 lineSegs=[0, 1]
PASS input=[0, 1, 2] nChars=1 lineSegs=[0, 1]
```

링크한 `target/pr-review/release-test/deps/librhwp.rlib`의 SHA-256:
`11c5e59d9b5efd12562eb50f29b2893cadf65cb2029ff4f9ea69b2af0af9749d`.
아래 코드를 임시 `check_empty_axis.rs`로 저장하고 다음 명령으로 실행했습니다.

```sh
rustc --edition=2021 check_empty_axis.rs --extern rhwp=target/pr-review/release-test/deps/librhwp.rlib -L dependency=target/pr-review/release-test/deps -o check-empty-axis
./check-empty-axis
```

```rust
use rhwp::model::document::Section;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::parser::record::Record;
use rhwp::parser::tags;

fn main() {
    for starts in [vec![], vec![0], vec![0, 1], vec![0, 1, 2]] {
        let section = Section {
            paragraphs: vec![Paragraph {
                line_segs: starts.iter().map(|&text_start| LineSeg {
                    text_start, line_height: 1000, text_height: 1000,
                    baseline_distance: 850, segment_width: 42520,
                    ..Default::default()
                }).collect(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let bytes = rhwp::serializer::body_text::serialize_section(&section);
        let records = Record::read_all(&bytes).unwrap();
        let header = records.iter().find(|r| r.tag_id == tags::HWPTAG_PARA_HEADER).unwrap();
        let chars = u32::from_le_bytes(header.data[0..4].try_into().unwrap()) & 0x7fff_ffff;
        let count = u16::from_le_bytes(header.data[16..18].try_into().unwrap()) as usize;
        let emitted: Vec<u32> = records.iter().filter(|r| r.tag_id == tags::HWPTAG_PARA_LINE_SEG)
            .flat_map(|r| r.data.chunks_exact(36).map(|b| u32::from_le_bytes(b[0..4].try_into().unwrap())))
            .collect();
        let expected: Vec<u32> = starts.iter().copied().take_while(|&p| p <= 1).collect();
        assert_eq!(chars, 1);
        assert_eq!(count, expected.len());
        assert_eq!(emitted, expected);
        assert!(!records.iter().any(|r| r.tag_id == tags::HWPTAG_PARA_TEXT));
        println!("PASS input={starts:?} nChars={chars} lineSegs={emitted:?}");
    }
}
```
