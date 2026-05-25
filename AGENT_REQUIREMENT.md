# TARGET STORY: US-043.1 Fix TUI Scrolling (Alternate Screen)

## 1. ARCHITECTURE BOUNDARIES
- Add `crossterm` crate for advanced terminal control.
- Enter "Alternate Screen" mode to draw UI without affecting terminal scrollback.
- Hide cursor during operation. Show on exit.

## 2. EXECUTION PLAN
1. Update `Cargo.toml`: Add `crossterm = "0.27"`.
2. Update `src/dashboard.rs`: Remove the old `print!("{esc}[2J...")` ANSI clear hack. 
3. Update `src/main.rs`: 
   - Before loop: `execute!(stdout(), EnterAlternateScreen, Hide)`.
   - Loop: Use `tokio::select!` to race `sleep(2)` and `ctrl_c()`. 
   - Sleep tick: `MoveTo(0,0)`, `Clear(ClearType::All)`, call dashboard, flush stdout.
   - Ctrl+C tick: break loop.
   - Exit: `execute!(stdout(), Show, LeaveAlternateScreen)`.