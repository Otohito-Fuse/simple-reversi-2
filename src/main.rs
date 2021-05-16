use crossterm::cursor::{EnableBlinking, Hide, MoveTo, Show};
use crossterm::event::{read, Event, KeyCode};
use crossterm::style::{Colorize, Print, ResetColor, Styler};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
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
                    Print(v[i][j].bold().blue().on_yellow()),
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
                    Print(
                        (if cnt[i][j] > 0 { '+' } else { v[i][j] })
                            .bold()
                            .blue()
                            .on_yellow()
                    ),
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
        println!("盤面のサイズを4以上の偶数で半角数字で入力してください．Enterキーで確定します．");
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

    // ここからRAWモードに入る
    enable_raw_mode()?;

    // 盤面サイズを常時表示
    execute!(
        stdout(),
        Clear(ClearType::All),
        MoveTo(0, 0),
        Print(" ===== Simple Reversi ===== ".to_string().red().bold()),
        MoveTo(0, 1),
        Print(format!("盤面：{0} x {0}", size).to_string()),
    )?;

    // CPUとやるかどうかの入力・決定
    let mut cpu_flag = false;
    let mut cpu_only_flag = false;

    let mut item_num: usize = 0;
    let mut enter = false;
    loop {
        // 常時表示
        execute!(
            stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0),
            Print(" ===== Simple Reversi ===== ".to_string().red().bold()),
            MoveTo(0, 1),
            Print(format!("盤面：{0} x {0}", size).to_string()),
            MoveTo(0, 2),
            Print("モードを選択してください．↑↓キーで選択，Enterキーで決定．"),
        )?;
        // 選択肢を表示
        if item_num == 0 {
            execute!(stdout(), MoveTo(0, 3), Print("CPU対戦モード".blue().bold()),)?;
        } else {
            execute!(stdout(), MoveTo(0, 3), Print("CPU対戦モード"),)?;
        }
        if item_num == 1 {
            execute!(stdout(), MoveTo(0, 4), Print("観戦モード".blue().bold()),)?;
        } else {
            execute!(stdout(), MoveTo(0, 4), Print("観戦モード"),)?;
        }
        if item_num == 2 {
            execute!(stdout(), MoveTo(0, 5), Print("1人2役モード".blue().bold()),)?;
        } else {
            execute!(stdout(), MoveTo(0, 5), Print("1人2役モード"),)?;
        }
        // キー入力読み込み
        loop {
            let event = read()?;

            if event == Event::Key(KeyCode::Up.into()) {
                item_num = if item_num > 0 { item_num - 1 } else { item_num };
                break;
            }

            if event == Event::Key(KeyCode::Down.into()) {
                item_num = if item_num < 2 { item_num + 1 } else { item_num };
                break;
            }

            if event == Event::Key(KeyCode::Enter.into()) {
                enter = true;
                break;
            }

            // windowのサイズが変わったときは再描画→点滅の元なのでやはり削除
            // if let Event::Resize(_,_) = event {
            //    break;
            // }
        }
        if enter {
            break;
        }
    }

    if item_num == 0 {
        cpu_flag = true;
    } else if item_num == 1 {
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
    )?;

    let mut i_am_white = false;

    if cpu_flag {
        // どちらの番から始めるかの入力・決定
        let mut enter = false;
        loop {
            // 常時表示
            execute!(
                stdout(),
                Clear(ClearType::All),
                MoveTo(0, 0),
                Print(" ===== Simple Reversi ===== ".to_string().red().bold()),
                MoveTo(0, 1),
                Print(format!("盤面：{0} x {0}", size).to_string()),
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
                MoveTo(0, 3),
                Print(format!(
                    "{0}と{1}，どちらから始めますか？ {0}が先攻です．←→キーで選択，Enterキーで決定．",
                    BoardState::black_piece(),
                    BoardState::white_piece()
                )),
            )?;
            if !i_am_white {
                execute!(
                    stdout(),
                    MoveTo(3, 4),
                    Print(format!("{}", BoardState::black_piece()).blue().bold()),
                )?;
            } else {
                execute!(
                    stdout(),
                    MoveTo(3, 4),
                    Print(format!("{}", BoardState::black_piece())),
                )?;
            }
            if i_am_white {
                execute!(
                    stdout(),
                    MoveTo(6, 4),
                    Print(format!("{}", BoardState::white_piece()).blue().bold()),
                )?;
            } else {
                execute!(
                    stdout(),
                    MoveTo(6, 4),
                    Print(format!("{}", BoardState::white_piece())),
                )?;
            }
            // キー入力読み込み
            loop {
                let event = read()?;

                if event == Event::Key(KeyCode::Left.into()) {
                    if i_am_white {
                        i_am_white = false;
                    }
                    break;
                }

                if event == Event::Key(KeyCode::Right.into()) {
                    if !i_am_white {
                        i_am_white = true;
                    }
                    break;
                }

                if event == Event::Key(KeyCode::Enter.into()) {
                    enter = true;
                    break;
                }

                // windowのサイズが変わったときは再描画→点滅の元なのでやはり削除
                // if let Event::Resize(_,_) = event {
                //     break;
                // }
            }
            if enter {
                break;
            }
        }
    }

    // 盤面作成
    let mut bs = BoardState::new(size / 2, false);

    // ヘルプ（+印）を表示するかどうか
    let mut with_help_or_not = false;

    // 「そこには置けません．を表示するかどうか」
    let mut not_puttable_message = false;

    // カーソル位置
    // 観戦モードのときはカーソルを出さないようにする工夫
    let mut cursor_x: usize = 0;
    let mut cursor_y: usize = if cpu_only_flag { size } else { 0 };

    // ゲーム実行
    loop {
        // 一旦画面をクリアし、タイトルその他諸々を表示
        execute!(
            stdout(),
            EnterAlternateScreen,
            Clear(ClearType::All),
            MoveTo(0, 0),
            Print(" ===== Simple Reversi ===== ".to_string().red().bold()),
            MoveTo(0, 1),
            Print(format!("盤面：{0} x {0}", size).to_string()),
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
            MoveTo(0, 3),
            Print(
                preview_turn(&bs)
                    + if (cpu_flag
                        && (i_am_white || bs.is_it_white_turn())
                        && !(i_am_white && bs.is_it_white_turn()))
                        || cpu_only_flag
                    {
                        ""
                    } else {
                        "↑↓←→キーで選択，Enterキーで決定．"
                    }
            ),
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
            preview_board_with_help(
                &bs,
                cursor_x,
                if (cpu_flag
                    && (i_am_white || bs.is_it_white_turn())
                    && !(i_am_white && bs.is_it_white_turn()))
                    || cpu_only_flag
                {
                    size
                } else {
                    cursor_y
                },
                4,
            );
        } else {
            preview_board(
                &bs,
                cursor_x,
                if (cpu_flag
                    && (i_am_white || bs.is_it_white_turn())
                    && !(i_am_white && bs.is_it_white_turn()))
                    || cpu_only_flag
                {
                    size
                } else {
                    cursor_y
                },
                4,
            );
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
        if cursor_x == size + 1 {
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
                cursor_x = if cursor_x > 0 { cursor_x - 1 } else { cursor_x };
                move_cursor = true;
                break;
            }

            if event == Event::Key(KeyCode::Down.into()) {
                cursor_x = if cursor_x <= size {
                    cursor_x + 1
                } else {
                    cursor_x
                };
                move_cursor = true;
                break;
            }

            if event == Event::Key(KeyCode::Left.into()) {
                cursor_y = if cursor_y > 0 { cursor_y - 1 } else { cursor_y };
                move_cursor = true;
                break;
            }

            if event == Event::Key(KeyCode::Right.into()) {
                cursor_y = if cursor_y < size - 1 {
                    cursor_y + 1
                } else {
                    cursor_y
                };
                move_cursor = true;
                break;
            }

            // windowのサイズが変わったときは再描画→点滅の元なのでやはり削除
            // if let Event::Resize(_,_) = event {
            //     move_cursor = true;
            //     break;
            // }

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
            let mut yes = true;
            let mut enter = false;
            loop {
                // 常時表示
                execute!(
                    stdout(),
                    EnterAlternateScreen,
                    Clear(ClearType::All),
                    MoveTo(0, 0),
                    Print(" ===== Simple Reversi ===== ".to_string().red().bold()),
                    MoveTo(0, 1),
                    Print(format!("盤面：{0} x {0}", size).to_string()),
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
                    MoveTo(0, 5),
                    Print("本当に終了しますか？".bold()),
                )?;
                if yes {
                    execute!(
                        stdout(),
                        MoveTo(2, 7),
                        Clear(ClearType::CurrentLine),
                        Print("はい".blue().bold()),
                    )?;
                    execute!(stdout(), MoveTo(10, 7), Print("いいえ"),)?;
                } else {
                    execute!(
                        stdout(),
                        MoveTo(2, 7),
                        Clear(ClearType::CurrentLine),
                        Print("はい"),
                    )?;
                    execute!(stdout(), MoveTo(10, 7), Print("いいえ".blue().bold()),)?;
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

                    // windowのサイズが変わったときは再描画→点滅の元なのでやはり削除
                    // if let Event::Resize(_,_) = event {
                    //     break;
                    // }
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

    loop {
        // 常時表示
        execute!(
            stdout(),
            EnterAlternateScreen,
            Clear(ClearType::All),
            MoveTo(0, 0),
            Print(" ===== Simple Reversi ===== ".to_string().red().bold()),
            MoveTo(0, 1),
            Print(format!("盤面：{0} x {0}", size).to_string()),
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
        )?;
        // 盤面表示
        preview_board(&bs, size, size, 4);
        stdout().flush()?;

        // 結果表示
        execute!(
            stdout(),
            MoveTo(0, 5 + size as u16),
            Print(show_result(&bs)),
            MoveTo(0, 7 + size as u16),
            Print("終了するにはEnterを押してください．")
        )?;

        let event = read()?;

        if event == Event::Key(KeyCode::Enter.into()) {
            break;
        }

        // windowのサイズが変わったときは再描画→点滅の元なのでやはり削除
        // if let Event::Resize(_,_) = event {
        //     continue;
        // }
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
