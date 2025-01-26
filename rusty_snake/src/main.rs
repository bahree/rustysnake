// fn main() {
//     println!("Hello, world!");
// }

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
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

    // Allow the user to select the boundary size and initial speed
    let (width, height, initial_speed) = select_game_settings(&mut stdout)?;

    // Clear the screen after selection
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
    let mut speed = initial_speed; // Start with the selected speed

    let game_over_message; // Message to display on game over

    // Draw initial walls
    draw_score(&mut stdout, score, speed)?; // Draw score and speed above game area
    draw_walls(&mut stdout, width, height)?;

    loop {
        // Check for user input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') => {
                        game_over_message = "You quit!";
                        break; // Quit the game
                    }
                    KeyCode::Char(' ') => paused = !paused, // Pause/unpause
                    KeyCode::Char('+') => {
                        if speed > 50 {
                            speed -= 50; // Speed up
                            draw_score(&mut stdout, score, speed)?;
                        }
                    }
                    KeyCode::Char('-') => {
                        if speed < 500 {
                            speed += 50; // Slow down
                            draw_score(&mut stdout, score, speed)?;
                        }
                    }
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
        if last_instant.elapsed() >= Duration::from_millis(speed) {
            last_instant = Instant::now();

            // Apply the queued direction if valid
            direction = next_direction.clone();

            // Move snake
            let mut new_head = snake.last().unwrap().clone();
            new_head.x += direction.x;
            new_head.y += direction.y;

            // Check collisions
            if new_head.x < 1 || new_head.x >= width - 1 || new_head.y < 1 || new_head.y >= height - 1 {
                game_over_message = "Game Over! You hit the wall!";
                break;
            }
            if snake.contains(&new_head) {
                game_over_message = "Game Over! You hit yourself!";
                break;
            }

            // Update game state: food or move
            if new_head == food {
                // Snake eats the food and grows
                snake.push(new_head);
                score += 1; // Increment the score
                draw_score(&mut stdout, score, speed)?; // Update score display

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
    execute!(
        stdout,
        cursor::MoveTo(0, height as u16 + 2),
        SetForegroundColor(Color::White),
        Print(format!("{}\nPress Enter to continue...", game_over_message))
    )?;
    wait_for_enter()?;

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

fn select_game_settings(stdout: &mut std::io::Stdout) -> crossterm::Result<(i32, i32, u64)> {
    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        SetForegroundColor(Color::White),
        Print("Select Boundary Size:\n"),
        Print("1. Small (20x10)\n2. Medium (40x20)\n3. Large (60x30)\n"),
        Print("Press 1, 2, or 3 to choose: ")
    )?;
    stdout.flush()?;

    let width;
    let height;

    // Get boundary size selection
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('1') => {
                        width = 20;
                        height = 10;
                        execute!(
                            stdout,
                            cursor::MoveToNextLine(1),
                            Print("You selected Small (20x10)\n")
                        )?;
                        break;
                    }
                    KeyCode::Char('2') => {
                        width = 40;
                        height = 20;
                        execute!(
                            stdout,
                            cursor::MoveToNextLine(1),
                            Print("You selected Medium (40x20)\n")
                        )?;
                        break;
                    }
                    KeyCode::Char('3') => {
                        width = 60;
                        height = 30;
                        execute!(
                            stdout,
                            cursor::MoveToNextLine(1),
                            Print("You selected Large (60x30)\n")
                        )?;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    let speed;

    // Prompt for difficulty selection
    execute!(
        stdout,
        cursor::MoveToNextLine(2),
        Print("Select Difficulty:\n"),
        Print("1. Easy (300ms per tick)\n2. Normal (200ms per tick)\n3. Hard (100ms per tick)\n"),
        Print("Press 1, 2, or 3 to choose: ")
    )?;
    stdout.flush()?;

    // Get difficulty selection
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('1') => {
                        speed = 300; // Easy
                        execute!(
                            stdout,
                            cursor::MoveToNextLine(1),
                            Print("You selected Easy (300ms per tick)\n")
                        )?;
                        break;
                    }
                    KeyCode::Char('2') => {
                        speed = 200; // Normal
                        execute!(
                            stdout,
                            cursor::MoveToNextLine(1),
                            Print("You selected Normal (200ms per tick)\n")
                        )?;
                        break;
                    }
                    KeyCode::Char('3') => {
                        speed = 100; // Hard
                        execute!(
                            stdout,
                            cursor::MoveToNextLine(1),
                            Print("You selected Hard (100ms per tick)\n")
                        )?;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // Wait a moment before clearing the screen
    std::thread::sleep(Duration::from_millis(1000));
    execute!(stdout, terminal::Clear(ClearType::All))?;

    Ok((width, height, speed))
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

fn draw_score(stdout: &mut std::io::Stdout, score: i32, speed: u64) -> crossterm::Result<()> {
    execute!(
        stdout,
        cursor::MoveTo(0, 0), // Display score above the playing area
        SetForegroundColor(Color::White),
        Print(format!("Score: {} | Speed: {}ms", score, speed))
    )?;
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
