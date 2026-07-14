use rhwp::model::control::Control;

fn control_name(control: &Control) -> &'static str {
    match control {
        Control::SectionDef(_) => "SectionDef",
        Control::ColumnDef(_) => "ColumnDef",
        Control::Table(_) => "Table",
        Control::Shape(_) => "Shape",
        Control::Picture(_) => "Picture",
        Control::Header(_) => "Header",
        Control::Footer(_) => "Footer",
        Control::Footnote(_) => "Footnote",
        Control::Endnote(_) => "Endnote",
        Control::AutoNumber(_) => "AutoNumber",
        Control::NewNumber(_) => "NewNumber",
        Control::PageNumberPos(_) => "PageNumberPos",
        Control::Bookmark(_) => "Bookmark",
        Control::Hyperlink(_) => "Hyperlink",
        Control::Ruby(_) => "Ruby",
        Control::CharOverlap(_) => "CharOverlap",
        Control::PageHide(_) => "PageHide",
        Control::HiddenComment(_) => "HiddenComment",
        Control::Equation(_) => "Equation",
        Control::Field(_) => "Field",
        Control::Form(_) => "Form",
        Control::Unknown(_) => "Unknown",
    }
}

fn dump_paragraphs(prefix: &str, paragraphs: &[rhwp::model::paragraph::Paragraph]) {
    for (paragraph_index, paragraph) in paragraphs.iter().enumerate() {
        for (control_index, control) in paragraph.controls.iter().enumerate() {
            let location = format!("{prefix}/p{paragraph_index}/c{control_index}");
            print!("CONTROL\tpath={}\ttype={}", location, control_name(control));
            match control {
                Control::Equation(value) => print!(
                    "\ttac={}\twidth={}\theight={}\tscript={:?}",
                    value.common.treat_as_char,
                    value.common.width,
                    value.common.height,
                    value.script
                ),
                Control::Form(value) => print!(
                    "\twidth={}\theight={}\tname={:?}\tcaption={:?}",
                    value.width, value.height, value.name, value.caption
                ),
                Control::Hyperlink(value) => {
                    print!("\turl={:?}\ttext={:?}", value.url, value.text)
                }
                Control::Ruby(value) => print!(
                    "\tmain={:?}\truby={:?}\tpos={}\tratio={}",
                    value.main_text, value.ruby_text, value.pos_type, value.sz_ratio
                ),
                Control::Field(value) => print!(
                    "\tfield_type={:?}\tcommand={:?}\tfield_id={}",
                    value.field_type, value.command, value.field_id
                ),
                Control::Unknown(value) => print!("\tctrl_id=0x{:08x}", value.ctrl_id),
                _ => {}
            }
            println!();

            match control {
                Control::Table(table) => {
                    for (cell_index, cell) in table.cells.iter().enumerate() {
                        dump_paragraphs(&format!("{location}/cell{cell_index}"), &cell.paragraphs);
                    }
                }
                Control::Header(value) => {
                    dump_paragraphs(&format!("{location}/header"), &value.paragraphs)
                }
                Control::Footer(value) => {
                    dump_paragraphs(&format!("{location}/footer"), &value.paragraphs)
                }
                Control::Footnote(value) => {
                    dump_paragraphs(&format!("{location}/footnote"), &value.paragraphs)
                }
                Control::Endnote(value) => {
                    dump_paragraphs(&format!("{location}/endnote"), &value.paragraphs)
                }
                Control::HiddenComment(value) => {
                    dump_paragraphs(&format!("{location}/comment"), &value.paragraphs)
                }
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths: Vec<_> = std::env::args_os().skip(1).collect();
    if paths.is_empty() {
        return Err("usage: research_object_fixture_inventory <file>...".into());
    }

    for path in paths {
        let bytes = match std::fs::read(&path) {
            Ok(value) => value,
            Err(error) => {
                eprintln!(
                    "ERROR\t{}\tread\t{}",
                    std::path::Path::new(&path).display(),
                    error
                );
                continue;
            }
        };
        let document = match rhwp::parser::parse_document(&bytes) {
            Ok(value) => value,
            Err(error) => {
                eprintln!(
                    "ERROR\t{}\tparse\t{}",
                    std::path::Path::new(&path).display(),
                    error
                );
                continue;
            }
        };
        println!("FILE\t{}", std::path::Path::new(&path).display());
        for (section_index, section) in document.sections.iter().enumerate() {
            dump_paragraphs(&format!("s{section_index}"), &section.paragraphs);
        }
    }
    Ok(())
}
