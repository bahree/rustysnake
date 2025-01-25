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

    const WIDTH: i32 = 20;
    const HEIGHT: i32 = 10;

    let mut snake = vec![Point { x: WIDTH / 2, y: HEIGHT / 2 }];
    let mut food = Point {
        x: 15.min(WIDTH - 2),
        y: 15.min(HEIGHT - 2),
    };
    let mut direction = Point { x: 1, y: 0 }; // Moving right
    let mut last_instant = Instant::now();

    // Draw initial walls
    draw_walls(&mut stdout, WIDTH, HEIGHT)?;

    loop {
        // Check for user input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc => break, // Exit game
                    KeyCode::Up if direction.y == 0 => direction = Point { x: 0, y: -1 },
                    KeyCode::Down if direction.y == 0 => direction = Point { x: 0, y: 1 },
                    KeyCode::Left if direction.x == 0 => direction = Point { x: -1, y: 0 },
                    KeyCode::Right if direction.x == 0 => direction = Point { x: 1, y: 0 },
                    _ => {}
                }
            }
        }

        // Game logic: update snake position
        if last_instant.elapsed() >= Duration::from_millis(200) {
            last_instant = Instant::now();

            // Move snake
            let mut new_head = snake.last().unwrap().clone();
            new_head.x += direction.x;
            new_head.y += direction.y;

            // Check collisions
            if new_head.x < 1 || new_head.x >= WIDTH - 1 || new_head.y < 1 || new_head.y >= HEIGHT - 1 {
                execute!(stdout, cursor::MoveTo(0, HEIGHT as u16 + 2), Print("Game Over! You hit the wall!\n"))?;
                break;
            }
            if snake.contains(&new_head) {
                execute!(stdout, cursor::MoveTo(0, HEIGHT as u16 + 2), Print("Game Over! You hit yourself!\n"))?;
                break;
            }

            // Update game state: food or move
            if new_head == food {
                // Snake eats the food and grows
                snake.push(new_head);

                // Generate new food in a random position, avoiding the snake's body
                let mut rng = rand::thread_rng();
                loop {
                    let new_food = Point {
                        x: rng.gen_range(1..WIDTH - 1),
                        y: rng.gen_range(1..HEIGHT - 1),
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
                    cursor::MoveTo(tail.x as u16, tail.y as u16),
                    Print(" ")
                )?;
            }
        }

        // Render snake and food
        render_snake_and_food(&mut stdout, &snake, &food)?;
        stdout.flush()?;
    }

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn draw_walls(stdout: &mut std::io::Stdout, width: i32, height: i32) -> crossterm::Result<()> {
    for y in 0..height {
        for x in 0..width {
            if y == 0 || y == height - 1 || x == 0 || x == width - 1 {
                execute!(stdout, cursor::MoveTo(x as u16, y as u16), Print("#"))?;
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
    for segment in snake {
        execute!(
            stdout,
            cursor::MoveTo(segment.x as u16, segment.y as u16),
            SetForegroundColor(Color::Green), // Green snake
            Print("█")
        )?;
    }

    // Draw the food
    execute!(
        stdout,
        cursor::MoveTo(food.x as u16, food.y as u16),
        SetForegroundColor(Color::Red), // Red food
        Print("■")
    )?;

    Ok(())
}
