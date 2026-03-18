use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::{App, Mode, Pane, TaskMotion},
    model::TaskStatus,
};

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    frame.render_widget(Block::default().style(Style::default().bg(Color::Rgb(12, 14, 18))), area);

    let groups_height = if area.height < 24 { 4 } else { 5 };
    let composer_height = if area.height < 24 { 9 } else { 11 };
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(groups_height),
            Constraint::Length(1),
            Constraint::Length(composer_height),
            Constraint::Length(1),
            Constraint::Min(6),
            Constraint::Length(2),
        ])
        .margin(1)
        .split(area);

    draw_groups(frame, layout[0], app);
    draw_composer(frame, layout[2], app);
    draw_tasks(frame, layout[4], app);
    draw_footer(frame, layout[5], app);

    match app.mode {
        Mode::CreatingGroup => draw_group_modal(frame, centered_rect(42, 7, area), app),
        Mode::RenamingGroup => draw_rename_group_modal(frame, centered_rect(42, 7, area), app),
        Mode::ConfirmClearClosed => {
            draw_clear_closed_modal(frame, centered_rect(48, 8, area), app)
        }
        Mode::ConfirmDeleteGroup => {
            draw_delete_group_modal(frame, centered_rect(48, 8, area), app)
        }
        Mode::Normal | Mode::EditingCapture | Mode::EditingTask => {}
    }
}

fn draw_groups(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let block = pane_block("Groups", app.focus == Pane::Groups, app);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let hint = Line::styled(
        "Left/Right select, Enter activate, n new, r rename, x delete, d clear closed",
        Style::default().fg(Color::Rgb(112, 125, 148)),
    );

    let row_area = Rect {
        x: inner.x,
        y: inner.y + 1,
        width: inner.width,
        height: 1,
    };
    let group_area = Rect {
        x: row_area.x + 2,
        y: row_area.y,
        width: row_area.width.saturating_sub(4),
        height: 1,
    };

    let available_width = group_area.width.saturating_sub(4) as usize;
    let (start, end, show_left, show_right) = visible_group_window(app, available_width);
    let mut spans = Vec::new();

    for (index, group) in app.project.groups.iter().enumerate().skip(start).take(end - start) {
        let selected = index == app.selected_group;
        let active = index == app.active_group;
        let mut style = Style::default()
            .fg(Color::Rgb(210, 216, 226))
            .bg(Color::Rgb(30, 35, 44));

        if active {
            style = style
                .fg(Color::Rgb(16, 18, 22))
                .bg(Color::Rgb(255, 214, 102))
                .add_modifier(Modifier::BOLD);
        }
        if selected && app.focus == Pane::Groups {
            style = style
                .bg(Color::Rgb(74, 92, 128))
                .fg(Color::Rgb(246, 248, 251))
                .add_modifier(Modifier::BOLD);
        }
        if active && app.animations.group_flash.as_ref().is_some_and(|f| f.is_active()) {
            style = style.bg(Color::Rgb(255, 232, 148));
        }

        spans.push(Span::styled(format!(" {} ", group.name), style));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        "+ n",
        Style::default()
            .fg(Color::Rgb(140, 152, 172))
            .bg(Color::Rgb(22, 26, 34)),
    ));

    frame.render_widget(
        Paragraph::new(Text::from(vec![Line::default(), Line::default(), hint]))
            .wrap(Wrap { trim: false }),
        inner,
    );
    frame.render_widget(
        Paragraph::new(Line::styled(
            if show_left { "←" } else { " " },
            Style::default().fg(Color::Rgb(112, 125, 148)),
        )),
        Rect {
            x: row_area.x,
            y: row_area.y,
            width: 1,
            height: 1,
        },
    );
    frame.render_widget(
        Paragraph::new(Line::from(spans)).wrap(Wrap { trim: false }),
        group_area,
    );
    frame.render_widget(
        Paragraph::new(Line::styled(
            if show_right { "→" } else { " " },
            Style::default().fg(Color::Rgb(112, 125, 148)),
        ))
        .alignment(Alignment::Right),
        Rect {
            x: row_area.x + row_area.width.saturating_sub(1),
            y: row_area.y,
            width: 1,
            height: 1,
        },
    );
}

fn draw_composer(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let available_width = area.width.saturating_sub(4);
    let card_width = if available_width < 34 {
        available_width
    } else {
        available_width.min(104)
    };
    let card_height = area.height.min(9).max(5);
    let composer_area = centered_rect(card_width, card_height, area);
    let shifted = shift_rect(
        composer_area,
        app.animations.shake.as_ref().map(|s| s.offset()).unwrap_or(0),
    );
    let header = format!(
        " {} | Active: {}{} ",
        match app.mode {
            Mode::EditingTask => "Edit Task",
            _ => "Capture",
        },
        app.project.groups[app.active_group].name,
        match app.mode {
            Mode::EditingCapture => " | typing",
            Mode::EditingTask => " | editing",
            _ => "",
        },
    );
    let block = pane_block(&header, app.focus == Pane::Composer, app);
    let inner = block.inner(shifted).inner(Margin {
        horizontal: 2,
        vertical: 1,
    });
    frame.render_widget(block, shifted);

    let hint = match app.mode {
        Mode::EditingCapture => Line::styled(
            "Enter saves the capture. Esc cancels it.",
            Style::default().fg(Color::Rgb(108, 122, 145)),
        ),
        Mode::EditingTask => Line::styled(
            "Enter saves the task edit. Esc cancels it.",
            Style::default().fg(Color::Rgb(108, 122, 145)),
        ),
        _ => Line::styled(
            "Press Enter to start a new capture.",
            Style::default().fg(Color::Rgb(108, 122, 145)),
        ),
    };

    let editor_height = inner.height.saturating_sub(2) as usize;
    let mut lines = editor_lines(
        &app.composer,
        app.composer_cursor,
        inner.width as usize,
        editor_height.max(1),
        matches!(app.mode, Mode::EditingCapture | Mode::EditingTask),
        app,
    );
    lines.push(Line::default());
    lines.push(hint);

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false }),
        inner,
    );
}

fn draw_tasks(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let block = pane_block("Tasks", app.focus == Pane::Tasks, app);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = app
        .project
        .groups
        .get(app.active_group)
        .map(|group| {
            group
                .tasks
                .iter()
                .enumerate()
                .map(|(index, task)| {
                    let is_selected = index == app.selected_task && app.focus == Pane::Tasks;
                    let mut style = match task.status {
                        TaskStatus::Open => Style::default().fg(Color::Rgb(230, 234, 240)),
                        TaskStatus::Closed => Style::default()
                            .fg(Color::Rgb(120, 131, 147))
                            .add_modifier(Modifier::CROSSED_OUT),
                    };

                    if is_selected {
                        style = style.bg(Color::Rgb(44, 52, 70));
                    }

                    let animated = app.recent_task_index == Some(index);
                    if animated {
                        if let Some(flash) = &app.animations.task_flash {
                            if flash.is_active() {
                                style = style.bg(match app.recent_task_motion {
                                    Some(TaskMotion::Added) => Color::Rgb(34, 67, 64),
                                    Some(TaskMotion::Closed) => Color::Rgb(76, 46, 46),
                                    Some(TaskMotion::Reopened) => Color::Rgb(48, 72, 52),
                                    None => Color::Rgb(44, 52, 70),
                                });
                            }
                        }
                    }

                    let marker = match (task.status, animated, app.recent_task_motion) {
                        (TaskStatus::Open, true, Some(TaskMotion::Added)) => "✦",
                        (TaskStatus::Open, true, Some(TaskMotion::Reopened)) => "↺",
                        (TaskStatus::Open, _, _) => "•",
                        (TaskStatus::Closed, true, Some(TaskMotion::Closed)) => "◆",
                        (TaskStatus::Closed, _, _) => "×",
                    };
                    Line::from(vec![
                        Span::styled(
                            format!("{marker} "),
                            style.fg(match task.status {
                                TaskStatus::Open => Color::Rgb(122, 211, 164),
                                TaskStatus::Closed => Color::Rgb(150, 115, 115),
                            }),
                        ),
                        Span::styled(task.text.as_str(), style),
                    ])
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let content = if rows.is_empty() {
        Text::from(Line::styled(
            "No tasks yet. Use the capture pane to create one.",
            Style::default().fg(Color::Rgb(108, 122, 145)),
        ))
    } else {
        let viewport_height = inner.height as usize;
        let top = visible_vertical_window(app.selected_task, rows.len(), viewport_height);
        let bottom = (top + viewport_height).min(rows.len());
        let mut all_lines = Vec::with_capacity(bottom - top);
        all_lines.extend(rows.iter().skip(top).take(bottom - top).cloned());
        Text::from(all_lines)
    };

    frame.render_widget(Paragraph::new(content).wrap(Wrap { trim: false }), inner);
    if !rows.is_empty() {
        let viewport_height = inner.height as usize;
        let top = visible_vertical_window(app.selected_task, rows.len(), viewport_height);
        let bottom = (top + viewport_height).min(rows.len());
        let show_up = top > 0;
        let show_down = bottom < rows.len();

        frame.render_widget(
            Paragraph::new(Line::styled(
                if show_up { "↑" } else { " " },
                Style::default().fg(Color::Rgb(112, 125, 148)),
            )),
            Rect {
                x: inner.x + inner.width.saturating_sub(2),
                y: inner.y,
                width: 1,
                height: 1,
            },
        );
        frame.render_widget(
            Paragraph::new(Line::styled(
                if show_down { "↓" } else { " " },
                Style::default().fg(Color::Rgb(112, 125, 148)),
            )),
            Rect {
                x: inner.x + inner.width.saturating_sub(2),
                y: inner.y + inner.height.saturating_sub(1),
                width: 1,
                height: 1,
            },
        );
    }
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let status_color = if app.animations.status_flash.as_ref().is_some_and(|f| f.is_active()) {
        Color::Rgb(255, 214, 102)
    } else {
        Color::Rgb(144, 154, 171)
    };

    let footer = Line::from(vec![
        Span::styled("Arrows", Style::default().fg(Color::Rgb(255, 214, 102))),
        Span::raw(" move/focus  "),
        Span::styled("Enter", Style::default().fg(Color::Rgb(255, 214, 102))),
        Span::raw(" start/save/toggle  "),
        Span::styled("1/2/3", Style::default().fg(Color::Rgb(255, 214, 102))),
        Span::raw(" jump panes  "),
        Span::styled("n/r/x/d", Style::default().fg(Color::Rgb(255, 214, 102))),
        Span::raw(" group actions  "),
        Span::styled("e/c/x/o", Style::default().fg(Color::Rgb(255, 214, 102))),
        Span::raw(" edit/copy/close/reopen  "),
        Span::styled("q", Style::default().fg(Color::Rgb(255, 214, 102))),
        Span::raw(" quit  "),
        Span::styled(
            format!("| {}", app.status),
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        ),
    ]);

    frame.render_widget(Paragraph::new(footer).alignment(Alignment::Center), area);
}

fn draw_group_modal(frame: &mut Frame<'_>, area: Rect, app: &App) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" New Group ")
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Rgb(255, 214, 102))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(18, 20, 26)));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(Text::from(vec![
            Line::styled(
                "Name the group and press Enter",
                Style::default().fg(Color::Rgb(210, 214, 224)),
            ),
            Line::default(),
            input_line(&app.group_input, app.group_input_cursor, inner.width as usize, true, app),
        ]))
        .alignment(Alignment::Left),
        inner,
    );
}

fn draw_rename_group_modal(frame: &mut Frame<'_>, area: Rect, app: &App) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" Rename Group ")
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Rgb(255, 214, 102))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(18, 20, 26)));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(Text::from(vec![
            Line::styled(
                "Edit the group name and press Enter",
                Style::default().fg(Color::Rgb(210, 214, 224)),
            ),
            Line::default(),
            input_line(&app.group_input, app.group_input_cursor, inner.width as usize, true, app),
        ]))
        .alignment(Alignment::Left),
        inner,
    );
}

fn draw_clear_closed_modal(frame: &mut Frame<'_>, area: Rect, app: &App) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" Clear Closed Tasks ")
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Rgb(255, 176, 94))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(18, 20, 26)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let count = app
        .project
        .groups
        .get(app.active_group)
        .map(|group| {
            group
                .tasks
                .iter()
                .filter(|task| task.status == TaskStatus::Closed)
                .count()
        })
        .unwrap_or(0);

    frame.render_widget(
        Paragraph::new(Text::from(vec![
            Line::styled(
                format!("Remove {count} closed task(s) from this group?"),
                Style::default()
                    .fg(Color::Rgb(236, 239, 245))
                    .add_modifier(Modifier::BOLD),
            ),
            Line::default(),
            Line::styled(
                "This cannot be undone from inside the app.",
                Style::default().fg(Color::Rgb(255, 176, 94)),
            ),
            Line::default(),
            Line::styled(
                "Press y to confirm or Esc/n to cancel.",
                Style::default().fg(Color::Rgb(140, 152, 172)),
            ),
        ]))
        .alignment(Alignment::Left),
        inner,
    );
}

fn draw_delete_group_modal(frame: &mut Frame<'_>, area: Rect, app: &App) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" Delete Group ")
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Rgb(255, 120, 120))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(18, 20, 26)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let group_name = app
        .project
        .groups
        .get(app.selected_group)
        .map(|group| group.name.as_str())
        .unwrap_or("this group");

    frame.render_widget(
        Paragraph::new(Text::from(vec![
            Line::styled(
                format!("Delete group \"{group_name}\" and all its tasks?"),
                Style::default()
                    .fg(Color::Rgb(236, 239, 245))
                    .add_modifier(Modifier::BOLD),
            ),
            Line::default(),
            Line::styled(
                "This cannot be undone from inside the app.",
                Style::default().fg(Color::Rgb(255, 120, 120)),
            ),
            Line::default(),
            Line::styled(
                "Press y to confirm or Esc/n to cancel.",
                Style::default().fg(Color::Rgb(140, 152, 172)),
            ),
        ]))
        .alignment(Alignment::Left),
        inner,
    );
}

fn pane_block(title: &str, focused: bool, app: &App) -> Block<'static> {
    let mut color = Color::Rgb(72, 86, 108);
    if focused {
        color = Color::Rgb(255, 214, 102);
    }
    if focused && app.animations.focus_flash.as_ref().is_some_and(|f| f.is_active()) {
        color = Color::Rgb(255, 236, 161);
    }

    Block::default()
        .title(title.to_string())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .style(Style::default().bg(Color::Rgb(18, 20, 26)))
}

fn input_line(
    text: &str,
    cursor: usize,
    width: usize,
    show_cursor: bool,
    app: &App,
) -> Line<'static> {
    let safe_cursor = cursor.min(text.len());
    let cursor_char_index = byte_to_char_index(text, safe_cursor);
    let available_text_width = width.saturating_sub(6).max(1);
    let total_chars = text.chars().count();
    let scroll_chars = cursor_char_index.saturating_sub(available_text_width.saturating_sub(1));
    let end_chars = (scroll_chars + available_text_width).min(total_chars);
    let visible_text = slice_chars(text, scroll_chars, end_chars);
    let visible_chars: Vec<char> = visible_text.chars().collect();
    let cursor_in_view = cursor_char_index.saturating_sub(scroll_chars).min(visible_chars.len());
    let left: String = visible_chars.iter().take(cursor_in_view).collect();
    let show_left = scroll_chars > 0;
    let show_right = end_chars < total_chars;
    let cursor_on_char = cursor_in_view < visible_chars.len();
    let current_char = visible_chars.get(cursor_in_view).copied().unwrap_or(' ');

    let mut spans = vec![
        Span::styled(
            if show_left { "← " } else { "  " },
            Style::default().fg(Color::Rgb(112, 125, 148)),
        ),
        Span::styled("> ", Style::default().fg(Color::Rgb(255, 214, 102))),
        Span::styled(left, Style::default().fg(Color::Rgb(245, 247, 250))),
    ];

    if cursor_on_char {
        spans.push(Span::styled(
            current_char.to_string(),
            if show_cursor && app.animations.cursor_visible() {
                Style::default()
                    .fg(Color::Rgb(18, 20, 26))
                    .bg(Color::Rgb(255, 214, 102))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(245, 247, 250))
            },
        ));
        let trailing: String = visible_chars.iter().skip(cursor_in_view + 1).collect();
        spans.push(Span::styled(
            trailing,
            Style::default().fg(Color::Rgb(245, 247, 250)),
        ));
    } else {
        spans.push(Span::styled(
            if show_cursor && app.animations.cursor_visible() {
                "█"
            } else {
                " "
            },
            Style::default()
                .fg(Color::Rgb(255, 214, 102))
                .add_modifier(Modifier::BOLD),
        ));
    }

    spans.push(Span::styled(
        if show_right { " →" } else { "  " },
        Style::default().fg(Color::Rgb(112, 125, 148)),
    ));

    if text.is_empty() {
        spans = vec![
            Span::raw("  "),
            Span::styled("> ", Style::default().fg(Color::Rgb(255, 214, 102))),
            Span::styled(
                if show_cursor && app.animations.cursor_visible() {
                    "█"
                } else {
                    " "
                },
                Style::default()
                    .fg(Color::Rgb(255, 214, 102))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
        ];
    }

    Line::from(spans)
}

fn editor_lines(
    text: &str,
    cursor: usize,
    width: usize,
    height: usize,
    show_cursor: bool,
    app: &App,
) -> Vec<Line<'static>> {
    let safe_cursor = cursor.min(text.len());
    let before_cursor = &text[..safe_cursor];
    let cursor_line = before_cursor.chars().filter(|ch| *ch == '\n').count();
    let cursor_col = before_cursor
        .rsplit('\n')
        .next()
        .unwrap_or("")
        .chars()
        .count();
    let lines = split_preserving_empty_lines(text);
    let top = visible_vertical_window(cursor_line, lines.len(), height.max(1));
    let bottom = (top + height.max(1)).min(lines.len());
    let mut rendered = Vec::with_capacity(bottom.saturating_sub(top));

    for (line_index, line_text) in lines.iter().enumerate().skip(top).take(bottom - top) {
        let current = line_index == cursor_line;
        let safe_width = width.saturating_sub(4).max(1);
        let line_chars = line_text.chars().count();
        let scroll_left = if current {
            cursor_col.saturating_sub(safe_width.saturating_sub(1))
        } else {
            0
        };
        let end = (scroll_left + safe_width).min(line_chars);
        let visible = slice_chars(line_text, scroll_left, end);
        let visible_chars: Vec<char> = visible.chars().collect();
        let cursor_in_view = cursor_col.saturating_sub(scroll_left).min(visible_chars.len());
        let left: String = visible_chars.iter().take(cursor_in_view).collect();
        let cursor_on_char = cursor_in_view < visible_chars.len();
        let current_char = visible_chars.get(cursor_in_view).copied().unwrap_or(' ');

        let mut spans = vec![
            Span::styled(
                if current { ">" } else { " " },
                Style::default().fg(Color::Rgb(255, 214, 102)),
            ),
            Span::raw(" "),
            Span::styled(left, Style::default().fg(Color::Rgb(245, 247, 250))),
        ];

        if current {
            if cursor_on_char {
                spans.push(Span::styled(
                    current_char.to_string(),
                    if show_cursor && app.animations.cursor_visible() {
                        Style::default()
                            .fg(Color::Rgb(18, 20, 26))
                            .bg(Color::Rgb(255, 214, 102))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Rgb(245, 247, 250))
                    },
                ));
                let trailing: String = visible_chars.iter().skip(cursor_in_view + 1).collect();
                spans.push(Span::styled(
                    trailing,
                    Style::default().fg(Color::Rgb(245, 247, 250)),
                ));
            } else {
                spans.push(Span::styled(
                    if show_cursor && app.animations.cursor_visible() {
                        "█"
                    } else {
                        " "
                    },
                    Style::default()
                        .fg(Color::Rgb(255, 214, 102))
                        .add_modifier(Modifier::BOLD),
                ));
            }
        } else {
            spans.push(Span::styled(
                visible_chars.iter().skip(cursor_in_view).collect::<String>(),
                Style::default().fg(Color::Rgb(245, 247, 250)),
            ));
        }

        rendered.push(Line::from(spans));
    }

    if rendered.is_empty() {
        rendered.push(Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Rgb(255, 214, 102))),
            Span::styled(
                if show_cursor && app.animations.cursor_visible() {
                    "█"
                } else {
                    " "
                },
                Style::default()
                    .fg(Color::Rgb(255, 214, 102))
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    rendered
}

fn visible_group_window(app: &App, width: usize) -> (usize, usize, bool, bool) {
    if app.project.groups.is_empty() {
        return (0, 0, false, false);
    }

    let group_widths = app
        .project
        .groups
        .iter()
        .map(|group| group.name.chars().count() + 3)
        .collect::<Vec<_>>();

    let mut start = app.selected_group.min(group_widths.len().saturating_sub(1));
    let mut end = start + 1;
    let mut used = group_widths[start];

    while start > 0 {
        let needed = group_widths[start - 1] + 1;
        if used + needed > width {
            break;
        }
        start -= 1;
        used += needed;
    }
    while end < group_widths.len() {
        let needed = group_widths[end] + 1;
        if used + needed > width {
            break;
        }
        used += needed;
        end += 1;
    }

    (start, end, start > 0, end < group_widths.len())
}

fn visible_vertical_window(selected: usize, total: usize, viewport: usize) -> usize {
    if viewport == 0 || total <= viewport {
        return 0;
    }
    let padding = viewport / 3;
    let mut top = selected.saturating_sub(padding);
    if top + viewport > total {
        top = total.saturating_sub(viewport);
    }
    top
}

fn byte_to_char_index(text: &str, byte_index: usize) -> usize {
    text[..byte_index].chars().count()
}

fn slice_chars(text: &str, start: usize, end: usize) -> String {
    text.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn split_preserving_empty_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        vec![""]
    } else {
        text.split('\n').collect()
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(popup[1]);
    horizontal[1].inner(Margin {
        horizontal: 0,
        vertical: 0,
    })
}

fn shift_rect(area: Rect, offset_x: i16) -> Rect {
    if offset_x == 0 {
        return area;
    }

    let x = if offset_x.is_negative() {
        area.x.saturating_sub(offset_x.unsigned_abs())
    } else {
        area.x.saturating_add(offset_x as u16)
    };
    Rect { x, ..area }
}
