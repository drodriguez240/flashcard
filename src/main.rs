use std::str::CharIndices;

mod app;
mod database;
mod markup;
mod pages;
mod utils;

fn main() -> std::io::Result<()> {
    // let terminal = ratatui::init();
    // let app = app::App::new().run(terminal);
    // ratatui::restore();
    // app

    let s = r#"
yoyo

this is a line                                         that         is super long and should definitively wrap

kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk

bye"#;

    // dbg!(s);

    // let s = textwrap::fill(
    //     s,
    //     textwrap::Options::new(24)
    //         .wrap_algorithm(textwrap::WrapAlgorithm::FirstFit)
    //         .subsequent_indent(" "),
    // );
    // dbg!(s);

    dbg!(LineParser::new(s, 24).collect::<Vec<_>>());

    Ok(())
}

struct LineParser<'a> {
    text: &'a str,
    line_width: usize,
    start: usize,
    chars: CharIndices<'a>,
}

impl<'a> LineParser<'a> {
    fn new(text: &'a str, line_width: usize) -> Self {
        Self {
            text,
            line_width,
            start: 0,
            chars: text.char_indices(),
        }
    }
}

impl<'a> Iterator for LineParser<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.text.len() {
            return None;
        }

        let mut width = 0;
        loop {
            let Some((i, c)) = self.chars.next() else {
                let line = &self.text[self.start..];
                self.start = self.text.len();
                return Some(line);
            };

            match c {
                '\n' => {
                    let line = &self.text[self.start..i];
                    self.start = i + 1;
                    return Some(line);
                }
                _ => {
                    width += 1;
                    if width >= self.line_width {
                        let end = i + c.len_utf8();
                        let line = &self.text[self.start..end];
                        self.start = end;
                        return Some(line);
                    }
                }
            }
        }
    }
}
