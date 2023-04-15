#![allow(unused)]

use std::{
    io::{stdout, Write},
    thread,
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp, Show},
    event::{self, poll, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Print, PrintStyledContent, Stylize},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use rand::Rng;

const SIZE: u16 = 25;
const FRAME_DUR: u64 = 100;
const WARPING: bool = true;

type Pos = (u16, u16);

enum Direction {
    Up,
    Right,
    Left,
    Down,
}

struct Game {
    score: usize,
    snake: Vec<Pos>,
    food: Pos,
    direction: Direction,
}

impl Game {
    fn new() -> Self {
        Self {
            score: 0,
            snake: vec![(0, 0)],
            direction: Direction::Right,
            food: (2, 2),
        }
    }

    fn handle_keypress(&mut self) -> crossterm::Result<bool> {
        // Returns false if program has to quit
        thread::sleep(Duration::from_millis(FRAME_DUR - 1));
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(e) = event::read()? {
                match e.code {
                    KeyCode::Char('q') => return Ok(false),
                    KeyCode::Up => match self.direction {
                        Direction::Down => {}
                        _ => self.direction = Direction::Up,
                    },
                    KeyCode::Left => match self.direction {
                        Direction::Right => {}
                        _ => self.direction = Direction::Left,
                    },
                    KeyCode::Down => match self.direction {
                        Direction::Up => {}
                        _ => self.direction = Direction::Down,
                    },
                    KeyCode::Right => match self.direction {
                        Direction::Left => {}
                        _ => self.direction = Direction::Right,
                    },
                    _ => return Ok(true),
                };
            }
        }

        Ok(true)
    }

    fn generate_food(&mut self) {
        let mut rng = rand::thread_rng();
        let (mut food_col, mut food_row) = (0, 0);

        loop {
            (food_col, food_row) = (rng.gen_range(0..SIZE), rng.gen_range(0..SIZE));
            if !self.snake.contains(&(food_col, food_row)) {
                break;
            }
        }

        self.food = (food_col, food_row)
    }

    fn move_snake(&mut self) {
        let (mut new_col, mut new_row) = self.snake[0];

        match self.direction {
            Direction::Up => {
                if self.snake[0].1 > 0 {
                    new_row = self.snake[0].1 - 1
                } else if WARPING {
                    new_row = SIZE - 1
                }
            }
            Direction::Left => {
                if self.snake[0].0 > 0 {
                    new_col = self.snake[0].0 - 1
                } else if WARPING {
                    new_col = SIZE - 1
                }
            }
            Direction::Down => {
                if self.snake[0].1 < SIZE - 1 {
                    new_row = self.snake[0].1 + 1
                } else if WARPING {
                    new_row = 0
                }
            }
            Direction::Right => {
                if self.snake[0].0 < SIZE - 1 {
                    new_col = self.snake[0].0 + 1
                } else if WARPING {
                    new_col = 0
                }
            }
        }

        self.snake.insert(0, (new_col, new_row));

        // Pop back
        while self.snake.len() != self.score + 1 {
            self.snake.pop();
        }

        self.check_collision();
    }

    fn check_collision(&mut self) {
        // Update score if food in snake
        if self.snake.contains(&self.food) {
            self.score += 1;
            self.generate_food();
        }
    }

    fn get_left_corner(&self) -> (u16, u16) {
        let (cols, rows) = terminal::size().unwrap();
        (cols / 3 - SIZE + 1, (rows - SIZE) / 2)
    }

    fn draw_game(&self) {
        let left_corner = self.get_left_corner();

        // Draw Score
        queue!(
            stdout(),
            MoveTo(left_corner.0, left_corner.1 - 1),
            Print("Score : "),
            PrintStyledContent(self.score.to_string().bold().green())
        );

        // Draw Box
        queue!(stdout(), MoveTo(left_corner.0, left_corner.1));
        print!("┏{}┓", "━".repeat(SIZE as usize * 2));

        for _ in 0..SIZE {
            queue!(
                stdout(),
                MoveDown(1),
                MoveLeft(SIZE * 2 + 2),
                Print("┃"),
                MoveRight(SIZE * 2),
                Print("┃")
            );
        }

        queue!(stdout(), MoveDown(1), MoveLeft(SIZE * 2 + 2));
        print!("┗{}┛", "━".repeat(SIZE as usize * 2));

        let block = "██";

        // Draw snake head
        let head = self.as_real_coords(&self.snake[0]);
        queue!(
            stdout(),
            MoveTo(head.0, head.1),
            PrintStyledContent(block.blue())
        );

        // Draw snake body
        if self.score > 0 {
            for part in self.snake[1..].iter() {
                let part_coord = self.as_real_coords(part);
                queue!(
                    stdout(),
                    MoveTo(part_coord.0, part_coord.1),
                    PrintStyledContent(block.dim()),
                );
            }
        }

        // Draw food
        let food_coord = self.as_real_coords(&self.food);
        queue!(
            stdout(),
            MoveTo(food_coord.0, food_coord.1),
            PrintStyledContent(block.red())
        );

        // Draw instructions
        queue!(
            stdout(),
            MoveTo(left_corner.0 + SIZE * 2 + 10, left_corner.1 + 5),
            Print("- Use arrow keys to move"),
            MoveTo(left_corner.0 + SIZE * 2 + 10, left_corner.1 + 7),
            Print("- Hit Q to quit")
        );

        stdout().flush();
    }

    fn as_real_coords(&self, coord: &(u16, u16)) -> (u16, u16) {
        let left_corner = self.get_left_corner();
        let inside_corner = (left_corner.0 + 1, left_corner.1 + 1);

        (inside_corner.0 + coord.0 * 2, inside_corner.1 + coord.1)
    }

    fn clear(&self) {
        let left_corner = self.get_left_corner();

        // Clear Score
        queue!(
            stdout(),
            MoveTo(left_corner.0, left_corner.1 - 3),
            Clear(ClearType::CurrentLine),
            MoveTo(left_corner.0, left_corner.1),
        );

        // Clear inside box
        for _ in 0..SIZE + 1 {
            queue!(stdout(), MoveDown(1), Clear(ClearType::CurrentLine));
        }
    }

    fn run(&mut self) -> bool {
        self.clear();
        self.move_snake();
        self.draw_game();
        self.handle_keypress().unwrap()
    }
}

struct RawModeManager;

impl RawModeManager {
    fn new() -> Self {
        enable_raw_mode();
        execute!(stdout(), EnterAlternateScreen, Hide);
        Self
    }
}

impl Drop for RawModeManager {
    fn drop(&mut self) {
        disable_raw_mode();
        execute!(stdout(), LeaveAlternateScreen, Show);
    }
}

fn main() {
    let _raw_mode_manager = RawModeManager::new();
    let mut game = Game::new();

    while game.run() {}
}
