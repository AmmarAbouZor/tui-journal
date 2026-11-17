use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend, style::Modifier};

use super::create_default_app;
use crate::app::{UIComponents, keymap::Input, ui::ControlType};

#[tokio::test]
async fn focused_preview_scrolls_wrapped_text_without_editing_source() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();
    app.state.full_screen = true;
    app.current_entry_id = Some(0);
    let words = ('a'..='z')
        .map(|c| c.to_string().repeat(35))
        .collect::<Vec<_>>();
    let source = format!("**{}** {}", words[0], words[1..].join(" "));
    app.update_current_entry_content(source.clone())
        .await
        .unwrap();

    let mut ui = UIComponents::new(Default::default());
    ui.set_current_entry(Some(0), &mut app);
    let mut terminal = Terminal::new(TestBackend::new(42, 14)).unwrap();
    let key = |code| Input::new(code, KeyModifiers::NONE);
    for code in [KeyCode::Char('p'), KeyCode::Tab] {
        ui.handle_input(&key(code), &mut app).await.unwrap();
    }
    let frame = terminal.draw(|f| ui.render_ui(f, &app)).unwrap();
    let first = &frame.buffer[(1, 1)];
    assert_eq!(first.symbol(), "a");
    assert!(first.modifier.contains(Modifier::BOLD));
    let second = &frame.buffer[(1, 2)];
    assert_eq!(second.symbol(), "b");
    assert!(!second.modifier.contains(Modifier::BOLD));

    ui.handle_input(&key(KeyCode::Down), &mut app)
        .await
        .unwrap();
    let frame = terminal.draw(|f| ui.render_ui(f, &app)).unwrap();
    assert_eq!(frame.buffer[(1, 1)].symbol(), "b");

    // Editor commands must not mutate the underlying buffer while preview has focus.
    for code in [KeyCode::Char('x'), KeyCode::Enter, KeyCode::Char('!')] {
        ui.handle_input(&key(code), &mut app).await.unwrap();
    }
    let save = Input::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    ui.handle_input(&save, &mut app).await.unwrap();
    assert_eq!(
        app.get_current_entry().unwrap().content,
        format!("{source}!")
    );
}

#[tokio::test]
async fn preview_exit_shortcuts_restore_list_focus() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();
    let mut ui = UIComponents::new(Default::default());
    ui.set_current_entry(Some(0), &mut app);
    ui.handle_input(
        &Input::new(KeyCode::Char('p'), KeyModifiers::NONE),
        &mut app,
    )
    .await
    .unwrap();

    for exit in [
        Input::new(KeyCode::Esc, KeyModifiers::NONE),
        Input::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        Input::new(KeyCode::Char('['), KeyModifiers::CONTROL),
    ] {
        ui.handle_input(&Input::new(KeyCode::Tab, KeyModifiers::NONE), &mut app)
            .await
            .unwrap();
        ui.handle_input(&exit, &mut app).await.unwrap();
        assert_eq!(ui.active_control, ControlType::EntriesList);
    }
}
