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
use std::collections::{HashSet, VecDeque};
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Returns `(dx, dy)` for each direction.
    fn vector(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

fn main() -> crossterm::Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::Clear(ClearType::All), cursor::Hide)?;

    // Display the splash screen
    draw_splash_screen(&mut stdout)?;

    // Clear the screen after splash
    execute!(stdout, terminal::Clear(ClearType::All))?;

    // Allow the user to select the boundary size and initial speed
    let (width, height, initial_speed) = select_game_settings(&mut stdout)?;

    // Clear the screen after selection
    execute!(stdout, terminal::Clear(ClearType::All))?;

    // Initialize snake in the center
    let start = Point {
        x: width / 2,
        y: height / 2,
    };
    let mut snake = VecDeque::new();
    snake.push_back(start);

    // Keep a set of positions for quick collision checks
    let mut snake_positions = HashSet::new();
    snake_positions.insert(start);

    // Random initial food
    let mut food = Point {
        x: 15.min(width - 2),
        y: 15.min(height - 2),
    };

    // Start moving to the Right by default
    let mut direction = Direction::Right;
    let mut next_direction = direction;
    let mut last_instant = Instant::now();
    let mut score = 0;    // Track the score
    let mut paused = false;
    let mut speed = initial_speed; // Start with the selected speed

    let game_over_message; // Will set this when the game ends

    // Draw initial walls and initial status
    draw_score(&mut stdout, score, speed)?;
    draw_walls(&mut stdout, width, height)?;

    loop {
        // Check for user input (non-blocking)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') => {
                        game_over_message = "You quit!";
                        break;
                    }
                    KeyCode::Char(' ') => {
                        paused = !paused;
                    }
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
                    // Direction changes. Disallow reversing directly into yourself.
                    KeyCode::Up if direction != Direction::Down => {
                        next_direction = Direction::Up;
                    }
                    KeyCode::Down if direction != Direction::Up => {
                        next_direction = Direction::Down;
                    }
                    KeyCode::Left if direction != Direction::Right => {
                        next_direction = Direction::Left;
                    }
                    KeyCode::Right if direction != Direction::Left => {
                        next_direction = Direction::Right;
                    }
                    _ => {}
                }
            }
        }

        if paused {
            continue;
        }

        // Check if it's time to move the snake
        if last_instant.elapsed() >= Duration::from_millis(speed) {
            last_instant = Instant::now();

            // Update direction from the queued next_direction
            direction = next_direction;

            // Calculate new head position
            let (dx, dy) = direction.vector();
            let head = snake.back().unwrap();
            let new_head = Point {
                x: head.x + dx,
                y: head.y + dy,
            };

            // Check collisions: walls
            if new_head.x < 1 || new_head.x >= width - 1 || new_head.y < 1 || new_head.y >= height - 1 {
                game_over_message = "Game Over! You hit the wall!";
                break;
            }
            // Check collisions: self
            if snake_positions.contains(&new_head) {
                game_over_message = "Game Over! You hit yourself!";
                break;
            }

            // Update snake
            snake.push_back(new_head);
            snake_positions.insert(new_head);

            if new_head == food {
                // Ate the food
                score += 1;
                draw_score(&mut stdout, score, speed)?;

                // Generate new food, avoiding the snake
                let mut rng = rand::thread_rng();
                loop {
                    let new_food = Point {
                        x: rng.gen_range(1..width - 1),
                        y: rng.gen_range(1..height - 1),
                    };
                    if !snake_positions.contains(&new_food) {
                        food = new_food;
                        break;
                    }
                }
            } else {
                // Normal movement: pop tail
                let tail = snake.pop_front().unwrap();
                snake_positions.remove(&tail);

                // Clear the old tail position from screen
                execute!(
                    stdout,
                    cursor::MoveTo(tail.x as u16, (tail.y + 1) as u16),
                    Print(" ")
                )?;
            }
        }

        // Render the snake and the food
        render_snake_and_food(&mut stdout, &snake, &food)?;
        stdout.flush()?;
    }

    // Game is over, display the final message
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

/// Show the menu to select boundary size and speed, returning `(width, height, speed_ms_per_tick)`.
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
                        speed = 300;
                        execute!(
                            stdout,
                            cursor::MoveToNextLine(1),
                            Print("You selected Easy (300ms per tick)\n")
                        )?;
                        break;
                    }
                    KeyCode::Char('2') => {
                        speed = 200;
                        execute!(
                            stdout,
                            cursor::MoveToNextLine(1),
                            Print("You selected Normal (200ms per tick)\n")
                        )?;
                        break;
                    }
                    KeyCode::Char('3') => {
                        speed = 100;
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

/// Draws the boundary walls using `#`.
fn draw_walls(stdout: &mut std::io::Stdout, width: i32, height: i32) -> crossterm::Result<()> {
    for y in 0..height {
        for x in 0..width {
            if y == 0 || y == height - 1 || x == 0 || x == width - 1 {
                execute!(
                    stdout,
                    cursor::MoveTo(x as u16, (y + 1) as u16),
                    Print("#")
                )?;
            }
        }
    }
    Ok(())
}

/// Renders the snake and the food in one pass.
fn render_snake_and_food(
    stdout: &mut std::io::Stdout,
    snake: &VecDeque<Point>,
    food: &Point,
) -> crossterm::Result<()> {
    // Draw the snake
    // The last element in `snake` is the head
    if let Some((last_idx, _)) = snake.iter().enumerate().last() {
        for (i, segment) in snake.iter().enumerate() {
            if i == last_idx {
                // Head
                execute!(
                    stdout,
                    cursor::MoveTo(segment.x as u16, (segment.y + 1) as u16),
                    SetForegroundColor(Color::Yellow),
                    Print("█")
                )?;
            } else {
                // Body
                execute!(
                    stdout,
                    cursor::MoveTo(segment.x as u16, (segment.y + 1) as u16),
                    SetForegroundColor(Color::Green),
                    Print("█")
                )?;
            }
        }
    }

    // Draw the food
    execute!(
        stdout,
        cursor::MoveTo(food.x as u16, (food.y + 1) as u16),
        SetForegroundColor(Color::Red),
        Print("■")
    )?;

    Ok(())
}

/// Draws the score (and speed) at the top of the screen.
fn draw_score(stdout: &mut std::io::Stdout, score: i32, speed: u64) -> crossterm::Result<()> {
    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        SetForegroundColor(Color::White),
        Print(format!("Score: {} | Speed: {}ms ", score, speed))
    )?;
    Ok(())
}

/// Wait until the user presses Enter.
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

/// Draws a simple splash screen before the game starts.
fn draw_splash_screen(stdout: &mut std::io::Stdout) -> crossterm::Result<()> {
    let snake_art = r#"
       /^\\/^\\
      / o   o \\
     (    ^    )
      \\_______/
       |     |
       |     |
    "#;

    execute!(stdout, terminal::Clear(ClearType::All))?;

    // Print the ASCII snake art
    for (i, line) in snake_art.lines().enumerate() {
        execute!(
            stdout,
            cursor::MoveTo(10, i as u16 + 5),
            Print(line)
        )?;
    }

    // Print a message below the ASCII snake
    execute!(
        stdout,
        cursor::MoveTo(10, 12),
        Print("Starting the game in 3 seconds...")
    )?;

    stdout.flush()?; 
    std::thread::sleep(std::time::Duration::from_secs(3));

    Ok(())
}
