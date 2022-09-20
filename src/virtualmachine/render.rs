use crate::interpreter::Value;

use super::{ExecutionFrame, UpdateData};

const CLEAR: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";

fn move_cursor(f: &mut std::fmt::Formatter<'_>, x: usize, y: usize) -> std::fmt::Result {
    write!(f, "\x1b[{};{}H", y, x)
}

fn render_heap(f: &mut std::fmt::Formatter<'_>, left: usize, top: usize, data: &[String], color: &[&str]) -> std::fmt::Result {
    let max_width = data.iter().map(|s| s.len()).max().unwrap_or(1).max(3);
    let texts = data.iter().map(|s| format!("{:^width$}", s, width=max_width)).collect::<Vec<_>>();
    let tree_height = (((1 + texts.len()) as f64).log2().ceil() as u32).max(1) as usize;

    for row in 0..tree_height {
        let layer_start_indexing = 2usize.pow(row as u32) - 1;
        let count = 2usize.pow(row as u32);
        let height = tree_height - row;
        let upper_bound = (layer_start_indexing + count).min(data.len());
        let spacing = (1 + max_width) * 2usize.pow(height as u32 - 1);

        let offset = ((2.0f64.powf(height as f64 - 1.0) - 1.0) * (max_width as f64 + 1.0) / 2.0).ceil() as usize;

        move_cursor(f, left + offset, top + 2 * row)?;
        for (i, s) in texts[layer_start_indexing..upper_bound].iter().enumerate() {
            write!(f, "{}{}{}", color[layer_start_indexing + i], s, CLEAR)?;
            for _ in 0..(spacing - max_width) {
                write!(f, " ")?;
            }
        }

        let left_s = &format!("{:^width$}", "", width=(max_width)/2);
        let right_s= &format!("{:─^width$}", "", width=(max_width - 1)/2);

        if row != 0 {
            move_cursor(f, left + offset, top + 2 * row - 1)?;
            for (i, _) in texts[layer_start_indexing..upper_bound].iter().enumerate() {
                write!(f, "{}{}{}", if i % 2 == 0 {left_s} else {right_s}, if i % 2 == 0 {"┌"} else {"┐"}, if i % 2 == 0 {right_s} else {left_s})?;
                for j in 0..(spacing - max_width) {
                    if i % 2 == 0 {
                        if j == (spacing - max_width - 1) / 2 {
                            write!(f, "┴")?;
                        }
                        else {
                            write!(f, "─")?;
                        }
                    }
                    else {
                        write!(f, " ")?;
                    }
                    
                }
            }
        }
    }

    /*
    for (i, text) in texts.iter().enumerate() {
        let layer = ((i + 1) as f64).log2() as usize;
        let height = tree_height - layer;
        let spacing = (1 + max_width) * 2usize.pow(height as u32) - 1;
        let count = 2usize.pow(layer as u32);
        let total_width = count * max_width + (count - 1) * spacing;
        let layer_start_indexing = 2usize.pow(layer as u32) - 1;

        // writeln!(f, "{}, {}", i, layer_start_indexing)?;
        let this_left = root_x - total_width / 2 + (i - layer_start_indexing) * (max_width + spacing);
        move_cursor(f, this_left, root_y + layer * 2)?;
        write!(f, "{}", text)?;
    } */
    Ok(())
}

impl<'file> std::fmt::Display for ExecutionFrame<'file> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\x1b[2J{}", CLEAR)?;

        let mut left_most = 1;

        // Prepare to display the code listing if it exists
        if let Some(code) = self.function.raw_file {
            let highlight_line = self.last_line;
            let secondary_lines = &self.last_lines;
            let longest_line = code.lines().map(|s| s.len()).max().unwrap_or(10);
            move_cursor(f, left_most, 1)?;
            write!(f, "{}:", self.function.name.location.filename)?;
            
            for (i, line) in code.lines().enumerate() {
                
                move_cursor(f, left_most, i + 2)?;
                write!(f, "{:<4}| ", i + 1)?;
                if Some(i) == highlight_line {
                    write!(f, "{}", RED)?;
                }
                else if secondary_lines.contains(&i) {
                    write!(f, "{}", CYAN)?;
                }
                write!(f, "{}", line)?;
                move_cursor(f, left_most + 7 + longest_line, i + 2)?;
                write!(f, "{}|", CLEAR)?;
            }

            left_most += longest_line + 9;
        }

        // List all of the currently present variables
        let mut keys = self.variables.keys().filter(|s| !s.contains('$')).collect::<Vec<_>>();
        keys.sort();

        let count = keys.len().max(10);

        for (i, variable_name) in keys.iter().enumerate() {
            move_cursor(f, left_most, i + 2)?;
            write!(f, "{}: ", variable_name)?;

            /*
            if self.last_updated.contains(variable_name) {
                write!(f, "{}", YELLOW)?;
            }
            else if self.last_read.contains(variable_name) {
                write!(f, "{}", CYAN)?;
            }

            write!(f, "{}{}", self.variables.get(*variable_name).unwrap(), CLEAR)?; */

            let v = self.variables.get(*variable_name).unwrap();

            let color =  if self.last_updated.contains(&UpdateData::variable(variable_name.to_string())) {
                YELLOW
            }
            else if self.last_read.contains(&UpdateData::variable(variable_name.to_string())) {
                CYAN
            }
            else {
                CLEAR
            };

            match v {
                crate::interpreter::Value::Number(number) => write!(f, "{}{}{}", color, number, CLEAR),
                crate::interpreter::Value::Array(v) => {
                    let mut colors = Vec::new();
                    write!(f, "{}[", color)?;
                    for (i, v) in v.borrow().0.iter().enumerate() {
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        let this_color = if self.last_updated.contains(&UpdateData::indexed(variable_name.to_string(), i + 1)) {
                            YELLOW
                        }
                        else if self.last_read.contains(&UpdateData::indexed(variable_name.to_string(), i + 1)) {
                            CYAN
                        }
                        else {
                            color
                        };
                        colors.push(this_color);
                        write!(f, "{}{}{}", this_color, v, color)?;
                    }
                    write!(f, "]{}", CLEAR)?;

                    let heap = v.borrow().0.iter().map(|v| v.to_string()).collect::<Vec<_>>();

                    if let Value::Number(n) = v.borrow().1.clone() {
                        render_heap(f, left_most, count + 5, &heap[..n as usize], &colors[..n as usize])?;
                    }
                    
                    Ok(())
                },
                crate::interpreter::Value::Boolean(b) => write!(f, "{}{}{}", color, b, CLEAR),
                crate::interpreter::Value::None => write!(f, "{}None{}", color, CLEAR),
            }?;
        }

        move_cursor(f, 0, 10000)?;
        write!(f, ">")?;

        Ok(())
    }
}