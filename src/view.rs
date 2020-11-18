use std::io::{
    stdout,
    Stdout,
    Write,
};

use crossterm::{
    cursor,
    event,
    ExecutableCommand,
    QueueableCommand,
    style,
    terminal,
    terminal::ClearType,
};

use crate::{
    components::*,
    map::{
        Map,
        TileType,
    },
    util::Queue,
};

pub struct View {
    pub stdout: Stdout,
    pub width: u16,
    pub height: u16,
}

impl View {
    pub fn init() -> crossterm::Result<View> {
        let (width, height) = terminal::size()?;

        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        stdout
            .execute(cursor::Hide)?
            .execute(event::EnableMouseCapture)?;

        Ok(View { 
            stdout,
            width,
            height,
        })
    }

    pub fn draw_entity(&mut self, position: &Position, renderable: &Renderable) -> crossterm::Result<()> {
        self.stdout
            .queue(cursor::MoveTo(position.x as u16, position.y as u16))?
            .queue(style::PrintStyledContent(
                    style::style(renderable.glyph)
                    .with(renderable.foreground)
                    .on(renderable.background)
                )
            )?;

        Ok(())
    }

    pub fn draw_map(&mut self, map: &Map) -> crossterm::Result<()> {
        for (idx, tile) in map.tiles.iter().enumerate() {
            let x = idx / map.height;
            let y = idx % map.height;
             
            if map[(x, y)].visible {
                self.stdout.queue(cursor::MoveTo(x as u16, y as u16))?;
                
                if tile.tile_type == TileType::Wall {
                    self.stdout.queue(
                        style::PrintStyledContent(
                            style::style('\u{2592}')
                            .with(style::Color::White)
                            .on(style::Color::Black)
                        )
                    )?;

                } else {
                    self.stdout.queue(
                        style::PrintStyledContent(
                            style::style('.')
                            .with(style::Color::White)
                            .on(style::Color::Black)
                        )
                    )?;
                }
            } else if map[(x, y)].revealed {
                self.stdout.queue(cursor::MoveTo(x as u16, y as u16))?;
                
                if tile.tile_type == TileType::Wall {
                    self.stdout.queue(
                        style::PrintStyledContent(
                            style::style('\u{2592}')
                            .with(style::Color::Rgb{r: 128, g: 128, b: 128})
                            .on(style::Color::Black)
                        )
                    )?;
                } else {
                    self.stdout.queue(
                        style::PrintStyledContent(
                            style::style('.')
                            .with(style::Color::Rgb{r: 128, g: 128, b: 128})
                            .on(style::Color::Black)
                        )
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn draw_message_log(&mut self, message_log: &Queue<String>) -> crossterm::Result<()> {
        self.stdout.queue(cursor::MoveTo(0, self.height - message_log.max_size as u16))?;

        for msg in message_log.iter() {
            self.stdout
                .queue(style::Print(msg))?
                .queue(cursor::MoveToNextLine(1))?;
        }

        Ok(())
    }

    pub fn begin_frame(&mut self) -> crossterm::Result<()> {
        self.stdout
            .execute(terminal::Clear(ClearType::All))?;

        Ok(())
    }

    pub fn end_frame(&mut self) -> crossterm::Result<()> {
        self.stdout.flush()?;

        Ok(())
    }
}

impl Drop for View {
    fn drop(&mut self) {
        self.stdout
            .execute(terminal::Clear(ClearType::All)).unwrap()
            .execute(cursor::Show).unwrap()
            .execute(cursor::MoveTo(0,0)).unwrap()
            .execute(event::DisableMouseCapture).unwrap();

        terminal::disable_raw_mode().unwrap();
    }
}
