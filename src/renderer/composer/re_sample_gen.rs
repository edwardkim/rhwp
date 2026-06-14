//! 재현검증용 HWP 샘플 자동 생성 테스트
//!
//! 기존 HWP 파일을 템플릿으로 로드하고 텍스트를 교체하여
//! 통제된 재현검증 샘플을 생성한다.
//! 생성된 파일은 작업지시자가 한컴에서 열어 LINE_SEG 재현 결과를 검증한다.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    /// 템플릿 HWP 로드 → 텍스트 교체 → 저장
    fn generate_sample(
        template_path: &str,
        output_path: &str,
        texts: &[&str],
    ) -> Result<(), String> {
        generate_sample_with_options(template_path, output_path, texts, None, None)
    }

    /// 폰트를 코드로 변경 (한컴의 방식을 재현)
    /// ko_font: 한글 폰트, en_font: 영문 폰트 (None이면 한글과 동일)
    fn apply_font(
        core: &mut crate::document_core::DocumentCore,
        ko_font: &str,
        en_font: Option<&str>,
    ) {
        let en = en_font.unwrap_or(ko_font);
        let doc = &mut core.document;

        // 1. font_faces에 새 폰트 추가 (이미 있으면 인덱스 반환)
        let add_font = |fonts: &mut Vec<crate::model::style::Font>, name: &str| -> u16 {
            if let Some(idx) = fonts.iter().position(|f| f.name == name) {
                return idx as u16;
            }
            fonts.push(crate::model::style::Font {
                raw_data: None,
                name: name.to_string(),
                alt_type: 0,
                alt_name: None,
                type_info: None,
                default_name: None,
                subst_font: None,
            });
            (fonts.len() - 1) as u16
        };

        // 모든 언어 카테고리에 폰트 추가
        let mut ko_ids = [0u16; 7];
        let mut en_ids = [0u16; 7];
        for (lang_idx, lang_fonts) in doc.doc_info.font_faces.iter_mut().enumerate() {
            ko_ids[lang_idx] = add_font(lang_fonts, ko_font);
            en_ids[lang_idx] = add_font(lang_fonts, en);
        }

        // 2. CharShape의 font_ids 설정
        // 문단이 참조하는 CharShape를 찾아 font_ids 변경
        let para_cs_id = doc.sections[0].paragraphs[0]
            .char_shapes
            .first()
            .map(|cs| cs.char_shape_id as usize)
            .unwrap_or(0);

        if para_cs_id < doc.doc_info.char_shapes.len() {
            let cs = &mut doc.doc_info.char_shapes[para_cs_id];
            for lang in 0..7 {
                cs.font_ids[lang] = if lang == 1 {
                    en_ids[lang]
                } else {
                    ko_ids[lang]
                };
            }
            cs.raw_data = None;
        }

        // 3. raw_stream 무효화
        doc.doc_info.raw_stream = None;
        doc.doc_info.raw_stream_dirty = true;
    }

    /// 폰트 지정 샘플 생성
    fn generate_sample_with_font(
        _template_path: &str,
        output_path: &str,
        texts: &[&str],
        font_name: Option<&str>,
    ) -> Result<(), String> {
        generate_sample_with_font_pair(output_path, texts, font_name, None, None)
    }

    /// 한글/영문 폰트 별도 지정 샘플 생성
    /// 한컴 템플릿이 있으면 사용, 없으면 기본 템플릿 + apply_font
    fn generate_sample_with_font_pair(
        output_path: &str,
        texts: &[&str],
        ko_font: Option<&str>,
        en_font: Option<&str>,
        alignment: Option<crate::model::style::Alignment>,
    ) -> Result<(), String> {
        // 폰트 조합에 맞는 한컴 템플릿 선택
        let template = match (ko_font, en_font) {
            (Some("바탕"), Some("Arial")) => "template/blank-batang-arial.hwp",
            (Some("맑은 고딕"), Some("Times New Roman")) => "template/blank-malgun-times.hwp",
            (Some(ko), None) => match ko {
                "바탕" => "template/blank-batang.hwp",
                "바탕체" => "template/blank-batangche.hwp",
                "돋움" => "template/blank-dotum.hwp",
                "맑은 고딕" => "template/blank-malgun.hwp",
                _ => "template/empty.hwp",
            },
            _ => "template/empty.hwp",
        };

        generate_sample_with_options(template, output_path, texts, None, alignment)
    }

    /// 템플릿에 DocumentCore API로 텍스트 삽입하여 샘플 생성
    fn generate_sample_with_options(
        template_path: &str,
        output_path: &str,
        texts: &[&str],
        _font_name: Option<&str>, // 미사용 (폰트는 템플릿으로 결정)
        alignment: Option<crate::model::style::Alignment>,
    ) -> Result<(), String> {
        // output/ 디렉토리 자동 생성
        if let Some(parent) = Path::new(output_path).parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let tmpl = Path::new(template_path);
        if !tmpl.exists() {
            return Err(format!("템플릿 없음: {}", template_path));
        }
        let data = fs::read(tmpl).map_err(|e| e.to_string())?;
        let mut core = crate::document_core::DocumentCore::from_bytes(&data)
            .map_err(|e| format!("{:?}", e))?;

        // 정렬 변경
        if let Some(align) = alignment {
            let para = &core.document.sections[0].paragraphs[0];
            let ps_id = para.para_shape_id as usize;
            if ps_id < core.document.doc_info.para_shapes.len() {
                core.document.doc_info.para_shapes[ps_id].alignment = align;
                core.document.doc_info.para_shapes[ps_id].raw_data = None;
                core.document.doc_info.raw_stream = None;
                core.document.doc_info.raw_stream_dirty = true;
            }
        }

        // 첫 문단에 텍스트 삽입 (DocumentCore API 사용)
        if let Some(first_text) = texts.first() {
            let _ = core.insert_text_native(0, 0, 0, first_text);
        }

        // 추가 문단: Enter로 문단 분할 후 텍스트 삽입
        for (i, text) in texts.iter().enumerate().skip(1) {
            // 이전 문단 끝에서 Enter
            let prev_para = &core.document.sections[0].paragraphs[i - 1];
            let end_pos = prev_para.text.chars().count();
            let _ = core.split_paragraph_native(0, i - 1, end_pos);
            // 새 문단에 텍스트 삽입
            let _ = core.insert_text_native(0, i, 0, text);
        }

        // raw_stream 무효화
        core.document.sections[0].raw_stream = None;

        // 1. LINE_SEG가 채워진 버전 저장 (rhwp 렌더링 확인용)
        let bytes = crate::serializer::serialize_document(&core.document)
            .map_err(|e| format!("{:?}", e))?;
        fs::write(output_path, &bytes).map_err(|e| e.to_string())?;
        eprintln!("생성: {} ({}문단)", output_path, texts.len());

        // 2. LINE_SEG를 비운 버전 저장 (한컴 역공학용: *-empty.hwp)
        let empty_path = output_path.replace(".hwp", "-empty.hwp");
        for para in &mut core.document.sections[0].paragraphs {
            para.line_segs = vec![crate::model::paragraph::LineSeg::default()];
        }
        core.document.sections[0].raw_stream = None;
        let empty_bytes = crate::serializer::serialize_document(&core.document)
            .map_err(|e| format!("{:?}", e))?;
        fs::write(&empty_path, &empty_bytes).map_err(|e| e.to_string())?;
        eprintln!("생성: {} (LINE_SEG 비움)", empty_path);

        Ok(())
    }

    /// 한글만 반복하여 지정 길이의 텍스트 생성
    fn hangul_repeat(pattern: &str, target_chars: usize) -> String {
        let chars: Vec<char> = pattern.chars().collect();
        let mut result = String::new();
        for i in 0..target_chars {
            result.push(chars[i % chars.len()]);
        }
        result
    }

    /// 한글+공백 패턴 생성 ("가 나 다 라 ...")
    fn hangul_with_spaces(pattern: &str, target_chars: usize) -> String {
        let chars: Vec<char> = pattern.chars().collect();
        let mut result = String::new();
        let mut count = 0;
        let mut ci = 0;
        while count < target_chars {
            result.push(chars[ci % chars.len()]);
            count += 1;
            if count < target_chars {
                result.push(' ');
                count += 1;
            }
            ci += 1;
        }
        result
    }

    // ─── 1차 샘플: 기본 폭 측정 ───

    #[test]
    fn test_gen_re01_hangul_only() {
        // 한글만 반복 (공백 없음), 2~3줄 분량
        // A4 바탕체 10pt: 한 줄 약 43자 → 100자면 ~2.3줄
        let text = hangul_repeat("가나다라마바사아자차카타파하", 100);
        let result = generate_sample(
            "samples/lseg-01-basic.hwp",
            "output/re-01-hangul-only.hwp",
            &[&text],
        );
        if let Err(e) = result {
            eprintln!("re-01 생성 실패: {}", e);
        }
    }

    #[test]
    fn test_gen_re02_space_count() {
        // 한글+공백 ("가 나 다 라 ..."), 2~3줄
        let text = hangul_with_spaces("가나다라마바사아자차카타파하", 100);
        let result = generate_sample(
            "samples/lseg-01-basic.hwp",
            "output/re-02-space-count.hwp",
            &[&text],
        );
        if let Err(e) = result {
            eprintln!("re-02 생성 실패: {}", e);
        }
    }

    #[test]
    fn test_gen_re03_latin_only() {
        // 영문만 반복, 2~3줄
        let text = "abcdefghijklmnopqrstuvwxyz".repeat(8); // 208자
        let result = generate_sample(
            "samples/lseg-01-basic.hwp",
            "output/re-03-latin-only.hwp",
            &[&text],
        );
        if let Err(e) = result {
            eprintln!("re-03 생성 실패: {}", e);
        }
    }

    #[test]
    fn test_gen_re04_digit_only() {
        // 숫자만 반복, 2~3줄
        let text = "1234567890".repeat(20); // 200자
        let result = generate_sample(
            "samples/lseg-01-basic.hwp",
            "output/re-04-digit-only.hwp",
            &[&text],
        );
        if let Err(e) = result {
            eprintln!("re-04 생성 실패: {}", e);
        }
    }

    #[test]
    fn test_gen_re05_mixed_koen() {
        // 한영 혼합 반복, 2~3줄
        let base = "한글English한글English";
        let text = base.repeat(8);
        let result = generate_sample(
            "samples/lseg-01-basic.hwp",
            "output/re-05-mixed-koen.hwp",
            &[&text],
        );
        if let Err(e) = result {
            eprintln!("re-05 생성 실패: {}", e);
        }
    }

    #[test]
    fn test_gen_re06_punctuation() {
        // 구두점 포함 한글, 2~3줄
        let base = "가,나.다!라?마(바)사[아]자{차}";
        let text = base.repeat(5);
        let result = generate_sample(
            "samples/lseg-01-basic.hwp",
            "output/re-06-punctuation.hwp",
            &[&text],
        );
        if let Err(e) = result {
            eprintln!("re-06 생성 실패: {}", e);
        }
    }

    // ─── 폰트별 샘플 ───

    #[test]
    fn test_gen_re_font_variations() {
        let fonts = [
            ("batang", "바탕"),
            ("batangche", "바탕체"),
            ("gulim", "굴림"),
            ("gulimche", "굴림체"),
            ("dotum", "돋움"),
            ("dotumche", "돋움체"),
            ("malgun", "맑은 고딕"),
        ];

        // 동일한 테스트 텍스트 (한글+영문+숫자+구두점 혼합)
        let text = "가나다라 English 12345 가,나.다! 마바사아 test 67890 자차카타파하";
        let long_text = format!("{} {}", text, text); // 2줄 이상

        for (suffix, font_name) in &fonts {
            let output = format!("output/re-font-{}.hwp", suffix);
            let result = generate_sample_with_font(
                "samples/lseg-01-basic.hwp",
                &output,
                &[&long_text],
                Some(font_name),
            );
            match result {
                Ok(()) => eprintln!("생성: {} (폰트: {})", output, font_name),
                Err(e) => eprintln!("실패: {} — {}", output, e),
            }
        }
    }

    // ─── 정렬별 샘플 ───

    #[test]
    fn test_gen_re_alignment_variations() {
        use crate::model::style::Alignment;

        let aligns = [
            ("justify", Alignment::Justify),
            ("left", Alignment::Left),
            ("center", Alignment::Center),
            ("right", Alignment::Right),
        ];

        let text = hangul_repeat("가나다라마바사아자차카타파하", 100);

        for (suffix, alignment) in &aligns {
            let output = format!("output/re-align-{}.hwp", suffix);
            let result = generate_sample_with_options(
                "samples/lseg-01-basic.hwp",
                &output,
                &[&text],
                None,
                Some(*alignment),
            );
            match result {
                Ok(()) => eprintln!("생성: {} (정렬: {:?})", output, alignment),
                Err(e) => eprintln!("실패: {} — {}", output, e),
            }
        }
    }

    // ─── 폰트 설정 분석 (Task 404) ───

    #[test]
    fn test_analyze_font_config() {
        let templates = [
            "template/empty.hwp",
            "template/blank-batang.hwp",
            "template/blank-dotum.hwp",
            "template/blank-batangche.hwp",
            "template/blank-malgun.hwp",
            "samples/re-eng-mixed-batang-arial-empty.hwp",
            "samples/re-mixed-malgun-timesnew-hancom.hwp",
            "samples/re-eng-nospace-malgun-times.hwp", // + char_shapes_ref 출력
        ];

        for path in &templates {
            let p = Path::new(path);
            if !p.exists() {
                continue;
            }
            let data = fs::read(p).unwrap();
            let doc = crate::parser::parse_document(&data).unwrap();

            eprintln!("\n=== {} ===", path);

            // font_faces: 7개 언어 카테고리별 폰트 목록
            let lang_names = ["한글", "영어", "한자", "일어", "기타", "기호", "사용자"];
            for (li, fonts) in doc.doc_info.font_faces.iter().enumerate() {
                let lang = if li < lang_names.len() {
                    lang_names[li]
                } else {
                    "?"
                };
                let names: Vec<String> = fonts
                    .iter()
                    .map(|f| format!("{}(t{})", f.name, f.alt_type))
                    .collect();
                eprintln!("  font_faces[{}]({}): {:?}", li, lang, names);
            }

            // char_shapes: font_ids 확인
            for (ci, cs) in doc.doc_info.char_shapes.iter().enumerate() {
                eprintln!(
                    "  char_shapes[{}]: font_ids={:?} base_size={} spacings={:?} ratios={:?}",
                    ci, cs.font_ids, cs.base_size, cs.spacings, cs.ratios
                );
            }
            // 문단의 char_shape_ref 확인
            for (pi, para) in doc.sections[0].paragraphs.iter().enumerate() {
                let refs: Vec<String> = para
                    .char_shapes
                    .iter()
                    .map(|r| format!("pos={}→cs_id={}", r.start_pos, r.char_shape_id))
                    .collect();
                if !refs.is_empty() {
                    eprintln!("  para[{}] char_shapes: {:?}", pi, refs);
                }
            }
        }
    }

    // ─── 영문 폰트별 샘플 (Task 404) ───

    #[test]
    fn test_gen_re_english_font_variations() {
        // 가변폭 + 고정폭 폰트로 순수 영문 테스트
        // 한글/영문 폰트 조합 테스트
        let fonts: Vec<(&str, &str, Option<&str>)> = vec![
            // (접미사, 한글폰트, 영문폰트)
            ("batang", "바탕", None),                // 한영 동일: 바탕
            ("batangche", "바탕체", None),           // 한영 동일: 바탕체 (고정폭)
            ("dotum", "돋움", None),                 // 한영 동일: 돋움
            ("malgun", "맑은 고딕", None),           // 한영 동일: 맑은 고딕
            ("batang-arial", "바탕", Some("Arial")), // 한글=바탕, 영문=Arial
            ("malgun-times", "맑은 고딕", Some("Times New Roman")), // 한글=맑은고딕, 영문=Times (한컴 템플릿 사용)
            ("malgun-courier", "맑은 고딕", Some("Courier New")), // 한글=맑은고딕, 영문=Courier(고정폭)
        ];

        // 순수 영문 (공백 없이 연속 — char_level_break 경로)
        let latin_nospace = "abcdefghijklmnopqrstuvwxyz".repeat(8);
        // 영문 단어 (공백 포함 — 정상 줄바꿈 경로)
        let latin_words = "The quick brown fox jumps over the lazy dog and then runs back again to test line breaking behavior in this sample document ";
        let latin_words_long = latin_words.repeat(3);
        // 한영 혼합
        let mixed = "한글과English가Mixed된Text입니다Test문장Sentence한글English한글English";
        let mixed_long = mixed.repeat(3);

        for (suffix, ko_font, en_font) in &fonts {
            // 영문 연속 (공백 없음)
            let output = format!("output/re-eng-nospace-{}.hwp", suffix);
            let _ = generate_sample_with_font_pair(
                &output,
                &[&latin_nospace],
                Some(ko_font),
                *en_font,
                None,
            );

            // 영문 단어 (공백 있음)
            let output = format!("output/re-eng-words-{}.hwp", suffix);
            let _ = generate_sample_with_font_pair(
                &output,
                &[&latin_words_long],
                Some(ko_font),
                *en_font,
                None,
            );

            // 한영 혼합
            let output = format!("output/re-eng-mixed-{}.hwp", suffix);
            let _ = generate_sample_with_font_pair(
                &output,
                &[&mixed_long],
                Some(ko_font),
                *en_font,
                None,
            );
        }
    }

    // ─── 한영 전환 정밀 분석 샘플 ───

    #[test]
    fn test_gen_re_mixed_precision() {
        // 전환점 수를 제어한 샘플: 한영 전환 간격 역공학
        let samples = [
            // (접미사, 텍스트, 설명)
            ("1tr", "가나다라마바사아자차카타파하가나다라마바사아자차카타파하가나다라마바사아자차카타파ABCDEFGHIJKLMN", "전환1회: 한글43+영문14"),
            ("2tr", "가나다라마바사아자차카타파ABCDEFG가나다라마바사아자차카타파ABCDEFGHIJKLMNOPQ", "전환2회"),
            ("0tr", "가나다라마바사아자차카타파하가나다라마바사아자차카타파하가나다라마바사아자차카타파하가나다라마바사", "전환0: 한글만"),
        ];

        for (suffix, text, desc) in &samples {
            let output = format!("output/re-mixed-{}.hwp", suffix);
            let _ = generate_sample_with_font_pair(&output, &[text], Some("돋움"), None, None);
            eprintln!("  {}: {}", suffix, desc);
        }
    }

    // ─── 일괄 생성 ───

    #[test]
    fn test_gen_all_re_samples() {
        test_gen_re01_hangul_only();
        test_gen_re02_space_count();
        test_gen_re03_latin_only();
        test_gen_re04_digit_only();
        test_gen_re05_mixed_koen();
        test_gen_re06_punctuation();
        eprintln!("\n=== 1차 샘플 생성 완료 ===");
        eprintln!("검증 필요: samples/re-01 ~ re-06.hwp를 한컴에서 열어 확인");
    }

    // ─── 멀티사이즈 샘플 (Issue #1) ───

    #[test]
    fn test_gen_re_multisize() {
        // 한 문단 내에 두 가지 크기를 혼합하여 줄별 line_height 검증
        let sizes: Vec<(&str, i32, i32)> = vec![
            ("10-16", 1000, 1600), // 10pt + 16pt
            ("10-20", 1000, 2000), // 10pt + 20pt
            ("8-14", 800, 1400),   // 8pt + 14pt
            ("10-10", 1000, 1000), // 동일 크기 (기준선)
        ];

        fs::create_dir_all("output").unwrap();

        for (suffix, base_size, big_size) in &sizes {
            let output = format!("output/re-multisize-{}.hwp", suffix);

            let data = fs::read("template/empty.hwp").unwrap();
            let mut core = crate::document_core::DocumentCore::from_bytes(&data).unwrap();

            // CharShape 추가: 큰 글자용
            let mut big_cs = core.document.doc_info.char_shapes[0].clone();
            big_cs.base_size = *big_size;
            big_cs.raw_data = None;
            let big_cs_id = core.document.doc_info.char_shapes.len() as u32;
            core.document.doc_info.char_shapes.push(big_cs);
            core.document.doc_info.raw_stream = None;
            core.document.doc_info.raw_stream_dirty = true;

            // 기본 CharShape의 base_size 설정
            core.document.doc_info.char_shapes[0].base_size = *base_size;
            core.document.doc_info.char_shapes[0].raw_data = None;

            // 텍스트 삽입
            let text = "가나다라마바사아자차카타파하가나다라마바사아자차카타파하가나다라마바사아자차카타파하가나다라마바사아자차카타파하";
            let _ = core.insert_text_native(0, 0, 0, text);

            // CharShapeRef: 1/3 기본, 1/3 큰 글자, 1/3 기본
            let para = &mut core.document.sections[0].paragraphs[0];
            let text_len = para.text.chars().count();
            let big_start = text_len / 3;
            let big_end = text_len * 2 / 3;
            let big_start_utf16 = para.char_offsets.get(big_start).copied().unwrap_or(0);
            let big_end_utf16 = para.char_offsets.get(big_end).copied().unwrap_or(0);

            para.char_shapes = vec![
                crate::model::paragraph::CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 0,
                },
                crate::model::paragraph::CharShapeRef {
                    start_pos: big_start_utf16,
                    char_shape_id: big_cs_id,
                },
                crate::model::paragraph::CharShapeRef {
                    start_pos: big_end_utf16,
                    char_shape_id: 0,
                },
            ];

            core.document.sections[0].raw_stream = None;

            // LINE_SEG 채워진 버전
            let bytes = crate::serializer::serialize_document(&core.document).unwrap();
            fs::write(&output, &bytes).unwrap();

            // LINE_SEG 비운 버전
            let empty_path = output.replace(".hwp", "-empty.hwp");
            for p in &mut core.document.sections[0].paragraphs {
                p.line_segs = vec![crate::model::paragraph::LineSeg::default()];
            }
            core.document.sections[0].raw_stream = None;
            let empty_bytes = crate::serializer::serialize_document(&core.document).unwrap();
            fs::write(&empty_path, &empty_bytes).unwrap();

            eprintln!(
                "생성: {} (기본={}pt, 큰={}pt)",
                output,
                base_size / 100,
                big_size / 100
            );
        }
    }
}
