use tui::{
    backend::{Backend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Tabs, Cell, Row, Table},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    Frame
};
use crate::ui_data_state::{TimeData, TableData, SelectedTab};

pub fn ui<B: Backend>(f: &mut Frame<B>, months: &TimeData, years: &TimeData, table: &mut TableData, 
    balance: &mut Vec<Vec<String>>, cu_tab: &SelectedTab, width_data: &mut Vec<Constraint>) {

    let size = f.size();
    let selected_style_blue = Style::default().fg(Color::Blue).add_modifier(Modifier::REVERSED);
    let selected_style_red = Style::default().fg(Color::Red).add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::LightBlue);

    let header_cells = ["Date", "Details", "Source", "Amount", "Type"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));

    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);
    
    // iter through table data and turn them into rows and columns
    let rows = table.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().map(|c| Cell::from(c.to_string()));
        Row::new(cells).height(height as u16).bottom_margin(0)
    });

    //decides how many chunks of spaces in the terminal will be 
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([Constraint::Length(5), Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    let block = Block::default().style(Style::default().bg(Color::White).fg(Color::Green));
    f.render_widget(block, size);

    //color the first three letters of the month to blue
    let month_titles = months.titles.iter().map(|t| {
        let (first, rest) = t.split_at(3);
        Spans::from(vec![
            Span::styled(first, Style::default().fg(Color::Blue)),
            Span::styled(rest, Style::default().fg(Color::Green))
        ])
    })
    .collect();

    //color the first letter of the year to blue
    let year_titles = years.titles.iter().map(|t| {
        let (first, rest) = t.split_at(1);
        Spans::from(vec![
            Span::styled(first, Style::default().fg(Color::Blue)),
            Span::styled(rest, Style::default().fg(Color::Green))
        ])
    })
    .collect();

    let mut month_tab = Tabs::new(month_titles)
        .block(Block::default().borders(Borders::ALL).title("Months"))
        .select(months.index)
        .style(Style::default().fg(Color::Green))
        .highlight_style(Style::default()
        .add_modifier(Modifier::BOLD)
        .bg(Color::Black));
        
    
    let mut year_tab = Tabs::new(year_titles)
        .block(Block::default().borders(Borders::ALL).title("Years"))
        .select(years.index)
        .style(Style::default().fg(Color::Green))
        .highlight_style(Style::default()
        .add_modifier(Modifier::BOLD)
        .bg(Color::Black));

    // set up the table columns and their size
    let mut table_area = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Transactions"))
        .widths(&[
            Constraint::Percentage(10),
            Constraint::Percentage(40),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15)
        ]);

    let bal_data = balance.iter().map(|item| {
        let height = 1;
        let cells = item.iter().map(|c| {
            if c.contains("↑") {
                Cell::from(c.to_string()).style(Style::default().fg(Color::Blue))
            }
            else if c.contains("↓"){
                Cell::from(c.to_string()).style(Style::default().fg(Color::Red))
            }
            else {
                Cell::from(c.to_string())
            }
            
        });
        Row::new(cells).height(height as u16).bottom_margin(0)
    });

    let balance_area = Table::new(bal_data).block(Block::default().borders(Borders::ALL).title("Balance"))
            .widths(&width_data);

    match cu_tab {
    // previously added a black block to year and month widget on the selected value.
    // based on which widget is selected, turns the black block to green.
        SelectedTab::Months => {
            month_tab = month_tab.highlight_style(Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::LightGreen));
        }

        SelectedTab::Years => {
            year_tab = year_tab.highlight_style(Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::LightGreen));
        }
        // changes the color of row based on Expense or Income tx type.
        SelectedTab::Table => {
            if let Some(a) = table.state.selected() {
                if table.items[a][4] == "Expense" {
                    table_area = table_area.highlight_style(selected_style_red)
                        .highlight_symbol(">> ")
                }
                else if table.items[a][4] == "Income" {
                    table_area = table_area.highlight_style(selected_style_blue)
                .highlight_symbol(">> ")
                }
            }
            
        }
    }

    f.render_widget(balance_area, chunks[0]);
    f.render_widget(month_tab, chunks[2]);
    f.render_widget(year_tab, chunks[1]);
    f.render_stateful_widget(table_area, chunks[3], &mut table.state)
}