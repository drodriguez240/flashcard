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

this is a line        that         is super long and should definitively wrap

kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk

bye"#;

    dbg!(s);

    let wrapped = textwrap::fill(
        s,
        textwrap::Options::new(24).wrap_algorithm(textwrap::WrapAlgorithm::FirstFit),
    );
    dbg!(wrapped);

    Ok(())
}
