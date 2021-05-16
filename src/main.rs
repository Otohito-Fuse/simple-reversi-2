use crossterm::cursor::{EnableBlinking, Hide, MoveTo, Show};
use crossterm::event::{read, Event, KeyCode};
use crossterm::style::{
    Colorize, Print, ResetColor, Styler,
};
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode};
use crossterm::Result;
use crossterm::{execute, queue};

use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;

use rand::seq::SliceRandom;
use rand::thread_rng;

pub mod boardstate;
use boardstate::BoardState;

/// 整数の入力が不正である旨のメッセージ
fn err_not_int() {
    println!("半角数字で整数を入力してください．");
}

/// 入力が不適切な旨のメッセージ
fn err_input() {
    println!("入力が不適切です．");
}

/// 入力が範囲外の旨のメッセージ
fn err_not_range() {
    println!("入力が範囲外です．");
}

/// カーソル位置は青の太字にするように盤面を表示させるキュー
fn preview_board(bs: &BoardState, cursor_x: usize, cursor_y: usize, row_now: u16) {
    let v = bs.show_board();
    let n = bs.get_size();

    for i in 0..n {
        queue!(
            stdout(),
            MoveTo(0, row_now + i as u16),
            Clear(ClearType::CurrentLine),
        );
        for j in 0..n {
            if i == cursor_x && j == cursor_y {
                queue!(
                    stdout(),
                    MoveTo(2 * j as u16, row_now + i as u16),
                    Print(" "),
                    Print(v[i][j].bold().blue()),
                );
            } else {
                queue!(
                    stdout(),
                    MoveTo(2 * j as u16, row_now + i as u16),
                    Print(" "),
                    Print(v[i][j])
                );
            }
        }
    }
}

/// 置けるマス目に+印をつけ，カーソル位置は青の太字にするように盤面を表示させるキュー
fn preview_board_with_help(bs: &BoardState, cursor_x: usize, cursor_y: usize, row_now: u16) {
    let v = bs.show_board();
    let cnt = bs.cnt_reversable();
    let n = bs.get_size();
    for i in 0..n {
        queue!(
            stdout(),
            MoveTo(0, row_now + i as u16),
            Clear(ClearType::CurrentLine),
        );
        for j in 0..n {
            if i == cursor_x && j == cursor_y {
                queue!(
                    stdout(),
                    MoveTo(2 * j as u16, row_now + i as u16),
                    Print(" "),
                    Print((if cnt[i][j] > 0 { '+' } else { v[i][j] }).bold().blue()),
                );
            } else {
                queue!(
                    stdout(),
                    MoveTo(2 * j as u16, row_now + i as u16),
                    Print(" "),
                    Print(if cnt[i][j] > 0 { '+' } else { v[i][j] })
                );
            }
        }
    }
}

/// どちらのターンかを表示する
fn preview_turn(bs: &BoardState) -> String {
    format!("{}のターン．", bs.which_turn())
}

/// 結果を表示する
fn show_result(bs: &BoardState) -> String {
    let ((c1, s1), (c2, s2)) = bs.count_pieces();
    if s1 > s2 {
        format!("{0}が{1}個，{2}が{3}個で{0}の勝ち！", c1, s1, c2, s2)
    } else if s1 < s2 {
        format!("{0}が{1}個，{2}が{3}個で{2}の勝ち！", c1, s1, c2, s2)
    } else {
        format!("{0}が{1}個，{2}が{3}個で引き分け！", c1, s1, c2, s2)
    }
}

fn main() -> Result<()> {
    // Alternate Screen に入り、画面をクリアし、カーソルを非表示にし、Simple Reversi と表示
    execute!(
        stdout(),
        EnterAlternateScreen,
        Clear(ClearType::All),
        MoveTo(0, 0),
        Hide,
        Print(" ===== Simple Reversi ===== ".to_string().red().bold()),
        MoveTo(0, 1)
    )?;

    // ゲーム開始までは標準入力から行いたいのでRAWモードにはまだ入らない

    // 盤面サイズの入力・決定
    let size: usize;
    loop {
        println!("盤面のサイズを4以上の偶数で入力してください．Returnキーで確定します．");
        let mut size_string = String::new();
        std::io::stdin().read_line(&mut size_string).ok();
        if let Ok(n) = size_string.trim().parse::<usize>() {
            if n >= 4 && n % 2 == 0 {
                size = n;
                break;
            } else {
                err_input();
            }
        } else {
            err_not_int();
        }
    }

    // 2行目以降を消す
    for i in (1..=crossterm::cursor::position().unwrap().1).rev() {
        execute!(stdout(), MoveTo(0, i), Clear(ClearType::CurrentLine),)?;
    }

    // 盤面サイズを常時表示
    execute!(
        stdout(),
        MoveTo(0, 1),
        Print(format!("盤面：{0} x {0}", size).to_string()),
        MoveTo(0, 2)
    )?;

    // CPUとやるかどうかの入力・決定
    let mut cpu_flag: bool = false;
    let mut cpu_only_flag: bool = false;
    println!("CPUと戦う場合は1，CPUだけが操作しているのを見る場合は2，自分で両方を操作する場合はそれ以外を入力してください．");
    let mut y_or_no = String::new();
    std::io::stdin().read_line(&mut y_or_no).ok();
    if y_or_no.trim() == "1" {
        cpu_flag = true;
    } else if y_or_no.trim() == "2" {
        cpu_only_flag = true;
    }

    // 2行目以降を消す
    for i in (2..=crossterm::cursor::position().unwrap().1).rev() {
        execute!(stdout(), MoveTo(0, i), Clear(ClearType::CurrentLine),)?;
    }

    // モードを常時表示
    execute!(
        stdout(),
        MoveTo(0, 2),
        Print(
            if cpu_flag {
                "CPU対戦モード"
            } else if cpu_only_flag {
                "観戦モード"
            } else {
                "1人2役モード"
            }
            .to_string()
        ),
        MoveTo(0, 3)
    )?;

    let mut i_am_white: bool = false;

    if cpu_flag {
        // どちらの番から始めるかの入力・決定
        loop {
            println!(
                "{0}として始める場合は1を，{1}として始める場合は2を入力してください．{0}が先攻です．",
                BoardState::black_piece(),
                BoardState::white_piece()
            );
            let mut size_string = String::new();
            std::io::stdin().read_line(&mut size_string).ok();
            if let Ok(n) = size_string.trim().parse::<usize>() {
                match n {
                    1 => {
                        break;
                    }
                    2 => {
                        i_am_white = true;
                        break;
                    }
                    _ => {
                        err_not_range();
                    }
                }
            } else {
                err_not_int();
            }
        }
    }

    // 4行目以降を消す
    for i in (3..=crossterm::cursor::position().unwrap().1).rev() {
        execute!(stdout(), MoveTo(0, i), Clear(ClearType::CurrentLine),)?;
    }

    // 盤面作成
    let mut bs = BoardState::new(size / 2, false);

    // ヘルプ（+印）を表示するかどうか
    let mut with_help_or_not: bool = false;

    // 「そこには置けません．を表示するかどうか」
    let mut not_puttable_message: bool = false;

    // カーソル位置
    // 観戦モードのときはカーソルを出さないようにする工夫
    let mut cursor_x: usize = 0;
    let mut cursor_y: usize = if cpu_only_flag {size} else {0};

    // ここからRAWモードに入る
    enable_raw_mode()?;

    // ゲーム実行
    loop {
        // 一旦画面をクリア
        for i in (3..=crossterm::cursor::position().unwrap().1+1).rev() {
            execute!(stdout(), MoveTo(0, i), Clear(ClearType::CurrentLine),)?;
        }

        // どちらのターンかの表示
        execute!(
            stdout(),
            MoveTo(0, 3),
            Clear(ClearType::CurrentLine),
            Print(preview_turn(&bs)),
        )?;

        // 「そこには置けません」メッセージの表示
        if not_puttable_message {
            execute!(
                stdout(),
                MoveTo(0, 6 + size as u16),
                Clear(ClearType::CurrentLine),
                Print("そこには置けません".red().bold()),
            )?;
        }
        not_puttable_message = false;

        // 盤面の表示
        if with_help_or_not {
            preview_board_with_help(&bs, cursor_x, cursor_y, 4);
        } else {
            preview_board(&bs, cursor_x, cursor_y, 4);
        }

        // 盤面表示キューの内容を実行
        stdout().flush()?;

        // CPUの番の場合
        if (cpu_flag
            && (i_am_white || bs.is_it_white_turn())
            && !(i_am_white && bs.is_it_white_turn()))
            || cpu_only_flag
        {
            // 乱数発生用
            let mut rng = thread_rng();

            // 時間を空けつつメッセージを表示
            if cpu_flag {
                sleep(Duration::from_millis(250));
            }
            execute!(
                stdout(),
                MoveTo(0, 5 + size as u16),
                Print("CPU操作中...".bold()),
            )?;
            sleep(Duration::from_millis(if cpu_only_flag { 500 } else { 750 }));

            // 置けるマス目を重み付けしつつVecで管理
            let mut options: Vec<(usize, usize)> = Vec::new();
            let mut options_corners: Vec<(usize, usize)> = Vec::new();
            let vec = &bs.cnt_reversable();
            let n = bs.get_size();
            for i in 0..n {
                for j in 0..n {
                    if vec[i][j] > 0 {
                        for _ in 0..vec[i][j] {
                            options.push((i, j));
                        }
                        if (i == 0 || i == n - 1) && (j == 0 || j == n - 1) {
                            options_corners.push((i, j));
                        }
                    }
                }
            }

            // ランダムに選ぶ
            let &(i, j) = if options_corners.is_empty() {
                options
            } else {
                options_corners
            }
            .choose(&mut rng)
            .unwrap();

            // マス目更新
            let can_continue = bs.put(i, j);

            // 続行できないときはループを抜けてゲームを終了
            if !can_continue {
                break;
            }
            continue;
        }

        // 以下、自分の番の場合

        // 操作方法の表示
        if cursor_x == size {
            execute!(
                stdout(),
                MoveTo(0, 4 + size as u16),
                Clear(ClearType::CurrentLine),
                Print("駒が置ける場所のヒントを見る".blue().bold()),
            )?;
        } else {
            execute!(
                stdout(),
                MoveTo(0, 4 + size as u16),
                Clear(ClearType::CurrentLine),
                Print("駒が置ける場所のヒントを見る"),
            )?;
        }
        if cursor_x == size+1 {
            execute!(
                stdout(),
                MoveTo(0, 5 + size as u16),
                Clear(ClearType::CurrentLine),
                Print("ゲームを終わって結果を見る".blue().bold()),
            )?;
        } else {
            execute!(
                stdout(),
                MoveTo(0, 5 + size as u16),
                Clear(ClearType::CurrentLine),
                Print("ゲームを終わって結果を見る"),
            )?;
        }
        
        // カーソル移動操作ならtrueを返しloopを再び回す
        let mut move_cursor: bool = false;

        // キー入力読み込み
        loop {
            let event = read()?;
    
            if event == Event::Key(KeyCode::Up.into()) {
                cursor_x = if cursor_x > 0 {cursor_x - 1} else {cursor_x};
                move_cursor = true;
                break;
            }

            if event == Event::Key(KeyCode::Down.into()) {
                cursor_x = if cursor_x <= size {cursor_x + 1} else {cursor_x};
                move_cursor = true;
                break;
            }

            if event == Event::Key(KeyCode::Left.into()) {
                cursor_y = if cursor_y > 0 {cursor_y - 1} else {cursor_y};
                move_cursor = true;
                break;
            }

            if event == Event::Key(KeyCode::Right.into()) {
                cursor_y = if cursor_y < size-1 {cursor_y + 1} else {cursor_y};
                move_cursor = true;
                break;
            }
    
            if event == Event::Key(KeyCode::Enter.into()) {
                break;
            }
        }

        // カーソルを動かしただけのときはloopを回して再描画
        if move_cursor {
            continue;
        }

        // 終了処理
        if cursor_x == size + 1 {
            // 一旦画面をクリア
            for i in (3..=crossterm::cursor::position().unwrap().1).rev() {
                execute!(stdout(), MoveTo(0, i), Clear(ClearType::CurrentLine),)?;
            }
            let mut yes: bool = true;
            let mut enter: bool = false;
            execute!(
                stdout(),
                MoveTo(0, 5),
                Clear(ClearType::CurrentLine),
                Print("本当に終了しますか？".bold()),
            )?;
            loop {
                if yes {
                    execute!(
                        stdout(),
                        MoveTo(2, 7),
                        Clear(ClearType::CurrentLine),
                        Print("はい".blue().bold()),
                    )?;
                    execute!(
                        stdout(),
                        MoveTo(10, 7),
                        Print("いいえ"),
                    )?;
                } else {
                    execute!(
                        stdout(),
                        MoveTo(2, 7),
                        Clear(ClearType::CurrentLine),
                        Print("はい"),
                    )?;
                    execute!(
                        stdout(),
                        MoveTo(10, 7),
                        Print("いいえ".blue().bold()),
                    )?;
                }
                // キー入力読み込み
                loop {
                    let event = read()?;

                    if event == Event::Key(KeyCode::Left.into()) {
                        if !yes {
                            yes = true;
                        }
                        break;
                    }

                    if event == Event::Key(KeyCode::Right.into()) {
                        if yes {
                            yes = false;
                        }
                        break;
                    }
            
                    if event == Event::Key(KeyCode::Enter.into()) {
                        enter = true;
                        break;
                    }
                }
                if enter {
                    break;
                }
            }
            if yes {
                break;
            } else {
                continue;
            }
        }

        // ヘルプ表示処理
        if cursor_x == size {
            with_help_or_not = true;
            continue;
        }
        with_help_or_not = false;


        // 置けるマス目かどうか判定
        let v = bs.cnt_reversable();
        if v[cursor_x][cursor_y] == 0 {
            not_puttable_message = true;
            continue;
        }

        // マス目更新
        let can_continue = bs.put(cursor_x, cursor_y);

        // 続行できないときはループを抜けてゲームを終了
        if !can_continue {
            break;
        }
    }

    // 一旦画面をクリア
    for i in (3..=crossterm::cursor::position().unwrap().1).rev() {
        execute!(stdout(), MoveTo(0, i), Clear(ClearType::CurrentLine),)?;
    }

    // 盤面表示
    preview_board(&bs, size, size, 4);
    stdout().flush()?;

    // 結果表示
    execute!(
        stdout(),
        MoveTo(0,5+size as u16),
        Print(show_result(&bs)),
        MoveTo(0,7+size as u16),
        Print("終了するにはEnterを押してください．")
    )?;

    loop {
        let event = read()?;

        if event == Event::Key(KeyCode::Enter.into()) {
            break;
        }
    }

    // 画面を全消しする
    execute!(stdout(), Clear(ClearType::All),)?;

    // RAWモードを抜ける
    disable_raw_mode()?;

    // カーソルを表示に戻す．書式をリセットする．
    execute!(stdout(), Show, EnableBlinking, ResetColor)?;

    // Alternate Screen を抜ける
    execute!(stdout(), LeaveAlternateScreen)
}
