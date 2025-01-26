// fn main() {
//     println!("Hello, world!");
// }

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, // Explicitly import execute! macro
    style::{Color, Print, SetForegroundColor},
    terminal::{self, ClearType},
};
use rand::Rng;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

fn main() -> crossterm::Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::Clear(ClearType::All), cursor::Hide)?;

    // Allow the user to select the boundary size
    let (width, height) = select_boundary_size(&mut stdout)?;

    // Clear the screen after boundary selection
    execute!(stdout, terminal::Clear(ClearType::All))?;

    let mut snake = vec![Point { x: width / 2, y: height / 2 }];
    let mut food = Point {
        x: 15.min(width - 2),
        y: 15.min(height - 2),
    };
    let mut direction = Point { x: 1, y: 0 }; // Moving right
    let mut next_direction = direction.clone(); // Buffer for the next direction
    let mut last_instant = Instant::now();
    let mut score = 0; // Track the score
    let mut paused = false; // Pause state

    let mut game_over_message = None; // Message to display on game over

    // Draw initial walls
    draw_score(&mut stdout, score)?; // Draw score above game area
    draw_walls(&mut stdout, width, height)?;

    loop {
        // Check for user input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') => {
                        game_over_message = Some("You quit!");
                        break; // Quit the game
                    }
                    KeyCode::Char(' ') => paused = !paused, // Pause/unpause
                    KeyCode::Up if direction.y == 0 => next_direction = Point { x: 0, y: -1 },
                    KeyCode::Down if direction.y == 0 => next_direction = Point { x: 0, y: 1 },
                    KeyCode::Left if direction.x == 0 => next_direction = Point { x: -1, y: 0 },
                    KeyCode::Right if direction.x == 0 => next_direction = Point { x: 1, y: 0 },
                    _ => {}
                }
            }
        }

        // If paused, skip game logic
        if paused {
            continue;
        }

        // Game logic: update snake position
        if last_instant.elapsed() >= Duration::from_millis(200) {
            last_instant = Instant::now();

            // Apply the queued direction if valid
            direction = next_direction.clone();

            // Move snake
            let mut new_head = snake.last().unwrap().clone();
            new_head.x += direction.x;
            new_head.y += direction.y;

            // Check collisions
            if new_head.x < 1 || new_head.x >= width - 1 || new_head.y < 1 || new_head.y >= height - 1 {
                game_over_message = Some("Game Over! You hit the wall!");
                break;
            }
            if snake.contains(&new_head) {
                game_over_message = Some("Game Over! You hit yourself!");
                break;
            }

            // Update game state: food or move
            if new_head == food {
                // Snake eats the food and grows
                snake.push(new_head);
                score += 1; // Increment the score
                draw_score(&mut stdout, score)?; // Update score display

                // Generate new food in a random position, avoiding the snake's body
                let mut rng = rand::thread_rng();
                loop {
                    let new_food = Point {
                        x: rng.gen_range(1..width - 1),
                        y: rng.gen_range(1..height - 1),
                    };
                    if !snake.contains(&new_food) {
                        food = new_food;
                        break;
                    }
                }
            } else {
                // Normal movement: add new head, remove tail
                snake.push(new_head);
                let tail = snake.remove(0);

                // Clear the old tail position
                execute!(
                    stdout,
                    cursor::MoveTo(tail.x as u16, (tail.y + 1) as u16),
                    Print(" ")
                )?;
            }
        }

        // Render snake and food
        render_snake_and_food(&mut stdout, &snake, &food)?;
        stdout.flush()?;
    }

    // Display the game-over message and wait for Enter
    if let Some(message) = game_over_message {
        execute!(
            stdout,
            cursor::MoveTo(0, height as u16 + 2),
            SetForegroundColor(Color::White),
            Print(format!("{}\nPress Enter to continue...", message))
        )?;
        wait_for_enter()?;
    }

    // Clear the screen and show the final score
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    execute!(
        stdout,
        Print(format!("Final Score: {}\n", score)),
        Print("Thank you for playing!\n")
    )?;

    terminal::disable_raw_mode()?;
    Ok(())
}

fn wait_for_enter() -> crossterm::Result<()> {
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if let KeyCode::Enter = key_event.code {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn select_boundary_size(stdout: &mut std::io::Stdout) -> crossterm::Result<(i32, i32)> {
    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        SetForegroundColor(Color::White),
        Print("Select Boundary Size:\n1. Small (20x10)\n2. Medium (40x20)\n3. Large (60x30)\nPress 1, 2, or 3 to choose:")
    )?;
    stdout.flush()?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('1') => return Ok((20, 10)),  // Small
                    KeyCode::Char('2') => return Ok((40, 20)),  // Medium
                    KeyCode::Char('3') => return Ok((60, 30)),  // Large
                    _ => {}
                }
            }
        }
    }
}

fn draw_walls(stdout: &mut std::io::Stdout, width: i32, height: i32) -> crossterm::Result<()> {
    for y in 0..height {
        for x in 0..width {
            if y == 0 || y == height - 1 || x == 0 || x == width - 1 {
                execute!(stdout, cursor::MoveTo(x as u16, (y + 1) as u16), Print("#"))?;
            }
        }
    }
    Ok(())
}

fn render_snake_and_food(
    stdout: &mut std::io::Stdout,
    snake: &[Point],
    food: &Point,
) -> crossterm::Result<()> {
    // Draw the snake
    for (i, segment) in snake.iter().enumerate() {
        if i == snake.len() - 1 {
            // Render head differently
            execute!(
                stdout,
                cursor::MoveTo(segment.x as u16, (segment.y + 1) as u16),
                SetForegroundColor(Color::Yellow), // Yellow snake head
                Print("█") // Head
            )?;
        } else {
            // Render body
            execute!(
                stdout,
                cursor::MoveTo(segment.x as u16, (segment.y + 1) as u16),
                SetForegroundColor(Color::Green), // Green snake body
                Print("█") // Body
            )?;
        }
    }

    // Draw the food
    execute!(
        stdout,
        cursor::MoveTo(food.x as u16, (food.y + 1) as u16),
        SetForegroundColor(Color::Red), // Red food
        Print("■")
    )?;

    Ok(())
}

fn draw_score(stdout: &mut std::io::Stdout, score: i32) -> crossterm::Result<()> {
    execute!(
        stdout,
        cursor::MoveTo(0, 0), // Display score above the playing area
        SetForegroundColor(Color::White),
        Print(format!("Score: {}", score))
    )?;
    Ok(())
}
