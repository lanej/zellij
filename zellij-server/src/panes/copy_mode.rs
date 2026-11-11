//! Keyboard navigation over a frozen pane, using native rows and selection rendering.
use super::grapheme_join::unit_column_spans;
use super::grid::{Grid, Row};
use super::selection::Selection;
use zellij_utils::{data::ClientId, position::Position};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Point {
    row: usize,
    column: usize,
}

#[derive(Clone)]
pub struct CopyMode {
    pub owner: ClientId,
    invalidated: bool,
    rows: Vec<Row>,
    cursor: Point,
    anchor: Option<Point>,
    linewise: bool,
    top: usize,
    goal_column: usize,
    count: Option<usize>,
    pending_g: bool,
}

pub enum CopyModeResult {
    Continue,
    Yank(String),
    Exit,
}

impl CopyMode {
    pub fn invalidate(&mut self) {
        self.invalidated = true;
    }

    pub fn label(&self) -> String {
        if self.invalidated {
            return "COPY CANCELED — press a key to exit".to_owned();
        }
        let kind = if self.anchor.is_none() {
            "COPY"
        } else if self.linewise {
            "VISUAL LINE"
        } else {
            "VISUAL"
        };
        format!(
            "{} {}/{}{}",
            kind,
            self.cursor.row + 1,
            self.rows.len(),
            self.count.map(|n| format!(" ×{n}")).unwrap_or_default()
        )
    }

    fn units(&self, row: usize) -> Vec<(usize, usize)> {
        let row = &self.rows[row];
        let mut spans = unit_column_spans(&row.columns, row.may_have_groups);
        spans.retain(|(start, end)| end > start);
        if spans.is_empty() {
            spans.push((0, 1));
        }
        spans
    }

    fn snap_column(&self, row: usize, column: usize) -> usize {
        self.units(row)
            .iter()
            .rev()
            .find(|(start, _)| *start <= column)
            .map(|(start, _)| *start)
            .unwrap_or(0)
    }

    fn end_column(&self, point: Point) -> usize {
        self.units(point.row)
            .into_iter()
            .find(|(start, _)| *start == point.column)
            .map(|(_, end)| end)
            .unwrap_or(point.column + 1)
    }

    fn step(&self, point: Point, forward: bool) -> Point {
        let units = self.units(point.row);
        if forward {
            if let Some((column, _)) = units.iter().find(|(start, _)| *start > point.column) {
                return Point {
                    column: *column,
                    ..point
                };
            }
            if point.row + 1 < self.rows.len() {
                return Point {
                    row: point.row + 1,
                    column: 0,
                };
            }
        } else {
            if let Some((column, _)) = units.iter().rev().find(|(start, _)| *start < point.column) {
                return Point {
                    column: *column,
                    ..point
                };
            }
            if point.row > 0 {
                let row = point.row - 1;
                return Point {
                    row,
                    column: self.snap_column(row, usize::MAX),
                };
            }
        }
        point
    }

    fn kind(&self, point: Point, big_word: bool) -> u8 {
        let mut column = 0;
        for cell in &self.rows[point.row].columns {
            let end = column + cell.width();
            if column <= point.column && point.column < end {
                let c = cell.character;
                return if c.is_whitespace() {
                    0
                } else if big_word || c.is_alphanumeric() || c == '_' {
                    1
                } else {
                    2
                };
            }
            column = end;
        }
        0
    }

    fn word(&mut self, motion: &str) {
        let big_word = matches!(motion, "W" | "B" | "E");
        let forward = !matches!(motion, "b" | "B");
        let end = matches!(motion, "e" | "E");
        let mut point = self.cursor;
        if end || !forward {
            point = self.step(point, forward);
        }
        if forward && !end {
            let kind = self.kind(point, big_word);
            loop {
                let next = self.step(point, true);
                if next == point {
                    break;
                }
                let crossed_line = next.row != point.row && self.rows[next.row].is_canonical;
                point = next;
                if crossed_line || self.kind(point, big_word) != kind {
                    break;
                }
            }
        }
        while self.kind(point, big_word) == 0 {
            let next = self.step(point, forward);
            if next == point {
                break;
            }
            point = next;
        }
        if end || !forward {
            let kind = self.kind(point, big_word);
            loop {
                let next = self.step(point, forward);
                let crossed_line = next.row != point.row
                    && self.rows[if forward { next.row } else { point.row }].is_canonical;
                if next == point || crossed_line || self.kind(next, big_word) != kind {
                    break;
                }
                point = next;
            }
        }
        self.cursor = point;
    }

    fn range(&self, width: usize) -> Option<(Point, Point)> {
        let anchor = self.anchor?;
        let (mut start, mut end) = if anchor <= self.cursor {
            (anchor, self.cursor)
        } else {
            (self.cursor, anchor)
        };
        if self.linewise {
            while start.row > 0 && !self.rows[start.row].is_canonical {
                start.row -= 1;
            }
            while end.row + 1 < self.rows.len() && !self.rows[end.row + 1].is_canonical {
                end.row += 1;
            }
            start.column = 0;
            end.column = width;
        } else {
            end.column = self.end_column(end);
        }
        Some((start, end))
    }

    pub fn selected_text(&self, width: usize) -> Option<String> {
        if self.invalidated {
            return None;
        }
        let (start, end) = self.range(width)?;
        let mut lines: Vec<String> = vec![];
        for index in start.row..=end.row {
            let row = &self.rows[index];
            let from = if index == start.row { start.column } else { 0 };
            let to = if index == end.row { end.column } else { width };
            let mut text = String::new();
            let mut column = 0;
            for cell in &row.columns {
                let next_column = column + cell.width();
                if column < to && next_column > from {
                    cell.push_text_to(&mut text);
                }
                column = next_column;
            }
            if row.is_canonical || lines.is_empty() {
                lines.push(text);
            } else if let Some(line) = lines.last_mut() {
                line.push_str(&text);
            }
        }
        let mut text = lines
            .iter()
            .map(|s| s.trim_end())
            .collect::<Vec<_>>()
            .join("\n");
        if self.linewise {
            text.push('\n');
        }
        Some(text)
    }

    fn input(&mut self, key: &str, width: usize, height: usize) -> CopyModeResult {
        if matches!(key, "ESC" | "Esc") && self.anchor.take().is_some() {
            self.count = None;
            self.pending_g = false;
            return CopyModeResult::Continue;
        }
        if matches!(key, "ESC" | "Esc" | "q" | "Ctrl c" | "ENTER" | "Enter") {
            return CopyModeResult::Exit;
        }
        if key == "y" {
            return self
                .selected_text(width)
                .map(CopyModeResult::Yank)
                .unwrap_or(CopyModeResult::Continue);
        }
        if key.len() == 1
            && key.as_bytes()[0].is_ascii_digit()
            && (key != "0" || self.count.is_some())
        {
            self.count = Some(
                self.count
                    .unwrap_or(0)
                    .saturating_mul(10)
                    .saturating_add((key.as_bytes()[0] - b'0') as usize)
                    .min(1_000_000),
            );
            return CopyModeResult::Continue;
        }
        if key == "g" && !self.pending_g {
            self.pending_g = true;
            return CopyModeResult::Continue;
        }
        let count = self.count.take();
        let repeat = count.unwrap_or(1);
        let was_g = std::mem::take(&mut self.pending_g);
        match key {
            "h" | "←" => {
                for _ in 0..repeat.min(width) {
                    self.cursor.column = self
                        .units(self.cursor.row)
                        .iter()
                        .rev()
                        .find(|(start, _)| *start < self.cursor.column)
                        .map(|(start, _)| *start)
                        .unwrap_or(0);
                }
            },
            "l" | "→" => {
                for _ in 0..repeat.min(width) {
                    self.cursor.column = self
                        .units(self.cursor.row)
                        .iter()
                        .find(|(start, _)| *start > self.cursor.column)
                        .map(|(start, _)| *start)
                        .unwrap_or(self.cursor.column);
                }
            },
            "j" | "↓" | "k" | "↑" | "Ctrl d" | "Ctrl u" | "Ctrl f" | "Ctrl b" | "PgDn" | "PgUp" =>
            {
                let page = match key {
                    "Ctrl d" | "Ctrl u" => (height / 2).max(1),
                    "Ctrl f" | "Ctrl b" | "PgDn" | "PgUp" => height.max(1),
                    _ => 1,
                };
                let amount = repeat.saturating_mul(page);
                let up = matches!(key, "k" | "↑" | "Ctrl u" | "Ctrl b" | "PgUp");
                self.cursor.row = if up {
                    self.cursor.row.saturating_sub(amount)
                } else {
                    self.cursor
                        .row
                        .saturating_add(amount)
                        .min(self.rows.len() - 1)
                };
                self.cursor.column = self.snap_column(self.cursor.row, self.goal_column);
                return CopyModeResult::Continue;
            },
            "0" | "HOME" => self.cursor.column = 0,
            "$" | "END" => self.cursor.column = self.snap_column(self.cursor.row, usize::MAX),
            "^" => {
                let mut point = Point {
                    row: self.cursor.row,
                    column: 0,
                };
                while self.kind(point, false) == 0 {
                    let next = self.step(point, true);
                    if next == point || next.row != point.row {
                        break;
                    }
                    point = next;
                }
                self.cursor = point;
            },
            "g" if was_g => {
                self.cursor.row = count
                    .unwrap_or(1)
                    .saturating_sub(1)
                    .min(self.rows.len() - 1);
                self.cursor.column = 0;
            },
            "G" => {
                self.cursor.row = count
                    .map(|c| c.saturating_sub(1))
                    .unwrap_or(self.rows.len() - 1)
                    .min(self.rows.len() - 1);
                self.cursor.column = 0;
            },
            "w" | "b" | "e" | "W" | "B" | "E" => {
                for _ in 0..repeat {
                    let previous = self.cursor;
                    self.word(key);
                    if self.cursor == previous {
                        break;
                    }
                }
            },
            "v" | "V" | "SPACE" => {
                let linewise = key == "V";
                if self.anchor.is_some() && self.linewise == linewise {
                    self.anchor = None;
                } else {
                    self.anchor.get_or_insert(self.cursor);
                    self.linewise = linewise;
                }
            },
            _ => {},
        }
        self.goal_column = self.cursor.column;
        CopyModeResult::Continue
    }
}

impl Grid {
    pub fn start_copy_mode(&mut self, client_id: ClientId) -> bool {
        if self.width == 0 || self.height == 0 || self.copy_mode.is_some() {
            return false;
        }
        let mut rows = vec![];
        for row in &self.lines_above {
            rows.extend(
                row.with_trailing_blanks()
                    .into_owned()
                    .split_to_rows_of_length(self.width),
            );
        }
        let top = rows.len();
        rows.extend(self.viewport.iter().cloned());
        while rows.len() < top + self.height {
            rows.push(Row::new().canonical());
        }
        rows.extend(self.lines_below.iter().cloned());
        let (column, row, _) = self.cursor_coordinates().unwrap_or((0, 0, true));
        let row = if self.is_scrolled {
            self.height - 1
        } else {
            row.min(self.height - 1)
        };
        let mut mode = CopyMode {
            owner: client_id,
            invalidated: false,
            rows,
            cursor: Point {
                row: top + row,
                column,
            },
            anchor: None,
            linewise: false,
            top,
            goal_column: column,
            count: None,
            pending_g: false,
        };
        mode.cursor.column = mode.snap_column(mode.cursor.row, column);
        mode.goal_column = mode.cursor.column;
        self.copy_mode = Some(mode);
        self.reset_selection();
        self.mark_for_rerender();
        true
    }

    pub fn copy_mode_cursor(&self, client_id: Option<ClientId>) -> Option<(usize, usize, bool)> {
        let mode = self.copy_mode.as_ref()?;
        if mode.invalidated || client_id.is_some_and(|id| id != mode.owner) {
            return None;
        }
        Some((
            mode.cursor.column,
            mode.cursor.row.saturating_sub(mode.top),
            true,
        ))
    }

    pub fn copy_mode_input(&mut self, key: &str, client_id: ClientId) -> CopyModeResult {
        let Some(mut mode) = self.copy_mode.take() else {
            return CopyModeResult::Exit;
        };
        if mode.invalidated || mode.owner != client_id {
            self.copy_mode = Some(mode);
            return CopyModeResult::Exit;
        }
        let result = mode.input(key, self.width, self.height);
        let desired_top = if mode.cursor.row < mode.top {
            mode.cursor.row
        } else if mode.cursor.row >= mode.top + self.height {
            mode.cursor.row + 1 - self.height
        } else {
            mode.top
        };
        if desired_top < mode.top {
            self.move_viewport_up(mode.top - desired_top);
        } else {
            self.move_viewport_down(desired_top - mode.top);
        }
        mode.top = desired_top;
        let mut selection = Selection::default();
        if let Some((start, end)) = mode.range(self.width) {
            let position = |p: Point| {
                Position::new((p.row as isize - mode.top as isize) as i32, p.column as u16)
            };
            selection.set_start_and_end_positions(position(start), position(end));
            selection.finalize();
        }
        self.selection = selection;
        self.copy_mode = Some(mode);
        self.output_buffer.update_all_lines();
        self.mark_for_rerender();
        result
    }

    pub fn end_copy_mode(&mut self, client_id: ClientId) -> bool {
        if self
            .copy_mode
            .as_ref()
            .is_some_and(|mode| mode.owner == client_id)
        {
            self.copy_mode = None;
            self.reset_selection();
            self.reset_viewport();
            self.mark_for_rerender();
            true
        } else {
            false
        }
    }
}
