---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase47-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase47-goal-checkpoint.md` · 원문 SHA-256: `1d4689ded2adc4088a3d73eead83be2cb67101b1a43a75fb4c0a58453ca424d6`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# 47차 — 기존 px 보류 6개 재판정

목표 ACTIVE. 승인/순서/환경은 phase46-goal-checkpoint.md와 동일하다. 전수조사→이슈갱신→부적합실제보정/필요공통구현→보정파일/함수 issue_8232 명명·관련helper/module/manifest/생성/script/E2E참조갱신→필수재검증. 정상검사일괄개명/옛wrapper/중복등록/무승인PR·subagent/task메시지/예약/메모리쓰기 금지. 현재제품/검사실제변경0·개명0·전수완료아님. 이번턴은독립근거·실제Native4·dual전26쪽·직접판독/보류재판정·원장/이슈갱신을완료한progress.

R=/Users/tsjang/rhwp, branch audit/8232-px-regressions-20261025, HEAD238de6de41a208648f4ade6b494fe433a81c1fce, tracked clean. D=R/output/issue8232-audit. 공유target/pr-review·원본/PDF·다른작업장 보존. Cargo순차/jobs8/testthreads8/capture총8. 47차nextest3768/capture39662/ledger21801/catalog31661 모두exit0회수. 이번 기록 뒤 최종 프로세스 상태를 확인한다. 완료실행을중복시작하지말것.

대상 tests/cases/issue_7902_completed_page_geometry.rs 기존20pending중6개만재판정: vpos_title316,caller332/337,stack_picture383,caller397/402. 4tests+2helpers. 신규direct0/기존pending6해소/새원장0/새cause0. integration59/helper52. 직접7796/806files 그대로. 전체원장11765/2138files. 미분류3644/adjud7302/의미메모113/독립pending494/파일맥락212. classification unresolved3969그대로;pending500→494. 파일의다른14pending·sharednear68/edges79/baseline127를이6개의자격으로자동수용하지않음. sourceprefix override없음. validate_phase47.py PASS. product/test보정이나rename가아니라정상유지자격재검증이다.

Native선택4등록/4PASS/0FAIL summary0.040s,282필터제외,elapsed1.1139s. source/binary전후같음. phase47-existing-tests-producer.json/log. module전체14tests를실행한것이아니며그중vposHWP/HWPX·stackHWP/HWPX네caller만선택. 현재manifest에서suite004계산. 이전30함수전체단위해석은integration51/helper44,이번에는1–150/310–408과원검증문서task_m100_7902_stage3.md1–177관련부분을재확인했다. 같은파일의모든30함수를이번신규진척으로세지않았다.

run_phase47_completed_pages.py/O=phase47-completed-pages-evidence/state.json. frozenNative5c9a.../freshWASM0a81.../JSba97.../정본script현재hash동일·compat2022/96dpi/NativeSkia+PrintCanvasPNG/배타capturelock8. 기존Maclockedwrapperno-opt산출물현재samehead재사용이며새WASMbuild/Docker통과주장금지.4조건전26쪽/backend: table-vpos-01.hwp/.hwpx각5 + issue2004_cell_image_stack.hwp/.hwpx각8.248.0594s전체종료. raw전체min vposN99.93951/W99.94159,stackN96.94369/W96.94378.미달/쪽수차이/누락0.선정vposp1/stackp5는4조건모두N/W100. stackp1은98미만이며여기에새px핀추가하지않음.다른서식의점수를현재입력대신쓰지않았다.

독립: inspect_phase47_independent.py/phase47-independent-contracts.json. Pdfium5.13원시textUnicode/CharOrigin에서공백정규화제목'정부혁신으로달라지는모습'유일출현을고르고12문자×2형식의baseline426.719970703125CSS를대조(기존426.719971와<1e-6).기대좌표근접검색으로문자선택하지않음. p5유일image/level0/objectget_bounds는80.357002,144.561996,514.523987,774.473022pt;841ptpage의top96CSS환산bbox107.142670,88.702637,686.031982,928.583984와max0.0000207201CSS차이.처음진단의존재하지않는get_pos호출AttributeError는출력직전실패했고실제localAPI.get_bounds로정정했다.제품/원본변경없음. CFB원시TABLE와HWPX직접XML에서vpos문단3의3x3·stack문단42의1x1소유와inputhash확인. bbox만으로소유주장하지않으며기존nodeidentity/셀포함·제목/후속본문순서도실제test에서보호한다.

읽기: make_phase47_reading.py/supplement_phase47_reading.py,phase47-reading.json. vpos각p1/p2·stack각p1/p5=대표8쪽/backend preview모두도구로직접읽음. Nativevposp1전체review원본과stackp5rawPNG도읽었다. 제목/후속본문2영역+그림모서리8영역/backend초기2x,추가후속본문좌우4/그림첫마지막가시내용좌우8=12영역/backend추가,총22영역/backend2x·원본review와standaloneoverlay패널같음assert.8preview와detail4+supplement4시트모두읽음. Native/WASM의미세글리프/괘선/그레이차이는남으며raw100을strictRGB동일로쓰지않음.

그림bbox경계에는원본bitmap흰여백이있다.초기40x40모서리crop은실제이미지잉크가아닌가까운바깥셀가로선만보인다.이를그림네모서리잉크의직접판독으로주장하지않는다. 독립PDF유일imagebbox·원본셀소유·typedNode와전체그림/첫마지막가시내용확대를연결해수용근거구분. sourceImagebboxx107과외부Tableframex79는같은개체가아니다. 이혼동을만나rawPNG/review/metadata를직접확인했다. supplementary초기inlinecomprehension의지역label'left-right'는개별왼쪽/오른쪽이름이잘못붙는자기오류였고명시적script로재생성한뒤판독.캡처PNG/원본/PDF는안바꿈.전26쪽모든글리프확대판독을주장하지않음.

실제MERGED7941/84764fa21c96...가현재조상·테스트sourcebyte같음.원 도입의목적/선정사유task_m100_7902_stage3.md144–177에서각형식5쪽stack과1쪽vpos선정·source관계·기존실물px유지근거읽음.옛98/99점수를현재로재인용안함. phase47-origin-chain.json/prmetadata.

카탈로그builder에phase47whole4조건을연결하고기존table-vpos-page5 HWP같은조건5쪽을replace했다.원래key table-vpos-page5보존/capture_case_key=vpos-hwp로현재실제산출key기록/supersedes_evidence_source보존.단순재실행5쪽을추가조건으로부풀리지않아173→176조건,2686→2707쪽/backend(현재26새capture지만순증21).기존실패/재사용포함이며전체목록PASS아님. build_paired_evidence_catalog-before-phase47.py.txt보존.ignoredbuilder변경이며제품sweep자동PR래칫변경아님.

#8232본문64,845문자로한도임박하여자기가쓴44–46차블록5251문자만원문phase47-prior-44-46-unabridged.md와각차수fullbody/receipt보존후원인/근거/수치/범위를남긴요약으로압축. 외부/사용자편집은변경하지않았고요약블록을원문으로되돌리면beforebodybyte같음을검증(phase47-history-compaction.json). 현재topcatalog만176/2707로갱신하고47차추가. fresh46body동일확인→게시직전fresh확인→게시후EOF포함byte동일확인. updated2026-10-26T05:02:14Z,최종62768Python문자(JSsurrogate수는별도). bodySHA는phase47-remote-verification.json. 과거세부판정/실행범위를압축하면서완료로바꾸지않았다. 47차직전원문/요약전문/압축기록모두보존했다.

다음: 남은미분류3644/의미메모+맥락325/독립pending494와sourcecfg/공용helper/동적fixture/script/E2E의원본·독립정답·실행연결계속판정. 현재7902completedfile나머지14pending(near/edges/baseline와10caller범위등),별도original_frame_and_content의3pending도남아있다. sourcepending38/HeightCursor12,다른실물7379/rowbreak·WMF·원본PDF근거pending등도이번6개로해소하지않는다. 모든실제부적합보정/새issue8232명명·전체호출참조갱신·필수집중/전체/시각재검증은전수확정뒤목표범위대로진행.전체목표ACTIVE/PR미생성유지.이번4/26쪽/독립24glyph+8edges/대표8+22영역/6재판정완료는반복실행하지않음.
