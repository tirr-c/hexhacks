// Copyright (C) 2021  Wonwoo Choi <chwo9843@gmail.com>
//
// This file is part of hexhacks.
//
// hexhacks is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// hexhacks is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with hexhacks.  If not, see <https://www.gnu.org/licenses/>.

use std::io;
use std::sync::Arc;

use crossterm::{
    execute,
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{self, Stylize},
    terminal,
    tty::IsTty,
};

#[derive(Debug, Clone)]
struct ScreenManager(Option<Arc<parking_lot::Once>>);

impl ScreenManager {
    fn init() -> crossterm::Result<Self> {
        Ok(Self(if io::stdout().is_tty() {
            execute!(
                io::stdout(),
                terminal::EnterAlternateScreen,
                terminal::Clear(terminal::ClearType::All),
                cursor::MoveTo(0, 0),
            )?;
            terminal::enable_raw_mode()?;
            Some(Default::default())
        } else {
            None
        }))
    }

    fn is_tty(&self) -> bool {
        self.0.is_some()
    }

    fn cleanup(&self) {
        if let Some(once) = &self.0 {
            once.call_once(|| {
                terminal::disable_raw_mode().ok();
                execute!(
                    io::stdout(),
                    cursor::Show,
                    terminal::LeaveAlternateScreen,
                ).ok();
            });
        }
    }
}

impl Drop for ScreenManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}

fn main() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let screen = ScreenManager::init()?;

    {
        // Adopted from human_panic::setup_panic!()
        let metadata = human_panic::Metadata {
            version: env!("CARGO_PKG_VERSION").into(),
            name: env!("CARGO_PKG_NAME").into(),
            authors: env!("CARGO_PKG_AUTHORS").replace(':', ", ").into(),
            homepage: env!("CARGO_PKG_HOMEPAGE").into(),
        };
        let screen = screen.clone();

        std::panic::set_hook(Box::new(move |info| {
            screen.cleanup();
            let file_path = human_panic::handle_dump(&metadata, info);
            human_panic::print_msg(file_path, &metadata)
                .expect("printing panic message failed")
        }));
    }

    if !screen.is_tty() {
        panic!("not a tty");
    }

    execute!(
        stdout,
        cursor::Hide,
        style::Print("Hello world!".underlined()),
    )?;

    loop {
        match event::read()? {
            Event::Key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE }) => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
