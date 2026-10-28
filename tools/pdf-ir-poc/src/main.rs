use std::{env, fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3
        && !(args.len() == 4 && matches!(args[3].as_str(), "--painted-rules" | "--tables"))
    {
        return Err(
            "usage: rhwp-pdf-oxide-poc INPUT.pdf OUTPUT_DIRECTORY [--painted-rules|--tables]"
                .into(),
        );
    }
    let bytes = fs::read(&args[1])?;
    let compiled = if args.get(3).is_some_and(|mode| mode == "--tables") {
        rhwp_pdf_oxide_poc::compile_page_tables(&bytes, 0)?
    } else if args.len() == 4 {
        rhwp_pdf_oxide_poc::compile_page_painted_rules(&bytes, 0)?
    } else {
        rhwp_pdf_oxide_poc::compile_page(&bytes, 0)?
    };
    let output = Path::new(&args[2]);
    fs::create_dir_all(output)?;
    fs::write(output.join("compiled.hwpx"), &compiled.hwpx)?;
    fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&compiled.report)?,
    )?;
    let core = rhwp::DocumentCore::from_bytes(&compiled.hwpx).map_err(|e| format!("{e:?}"))?;
    fs::write(
        output.join("native.svg"),
        core.render_page_svg_layer_native(0)
            .map_err(|e| format!("{e:?}"))?,
    )?;
    fs::write(
        output.join("native-render.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "pageCount":core.page_count(), "backend":"layer", "source":"fresh POC path dependency"
        }))?,
    )?;
    let reopened = rhwp::parse_document(&compiled.hwpx)?;
    let original = &compiled.document.sections[0].paragraphs[0];
    let loaded = &reopened.sections[0].paragraphs[0];
    // HWPX serialization emits section/column metadata ahead of the page's objects.
    let loaded_objects: Vec<_> = loaded
        .controls
        .iter()
        .filter(|c| {
            !matches!(
                c,
                rhwp::model::control::Control::SectionDef(_)
                    | rhwp::model::control::Control::ColumnDef(_)
            )
        })
        .collect();
    if original.controls.len() != loaded_objects.len()
        || reopened.bin_data_content.len() != compiled.document.bin_data_content.len()
    {
        return Err("HWPX round-trip changed object/image count".into());
    }
    for (a, b) in original.controls.iter().zip(loaded_objects) {
        if let (rhwp::model::control::Control::Shape(a), rhwp::model::control::Control::Shape(b)) =
            (a, b)
        {
            if let (Some(a), Some(b)) = (a.drawing(), b.drawing()) {
                if let (Some(a), Some(b)) = (&a.text_box, &b.text_box) {
                    if a.paragraphs[0].text != b.paragraphs[0].text {
                        return Err("text-box round-trip changed text".into());
                    }
                }
            }
        }
    }
    println!("{}", compiled.report["lowering"]);
    println!("HWPX round-trip: object/image counts and text preserved");
    Ok(())
}
