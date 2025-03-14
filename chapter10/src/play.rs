use std::sync::{Arc, Mutex};
use std::{thread, time};
use getch_rs::{Getch, Key};
use crate::game::*;
use crate::ai::eval;

// 通常プレイ
pub fn normal() -> ! {
    let game = Arc::new(Mutex::new(Game::new()));

    // 画面クリア
    println!("\x1b[2J\x1b[H\x1b[?25l");
    // フィールドを描画
    draw(&game.lock().unwrap());

    // 自然落下処理
    {
        let game = Arc::clone(&game);
        let _ = thread::spawn(move || {
            loop {
                // nミリ秒間スリーブする
                let sleep_msec = match 1000u64.saturating_sub((game.lock().unwrap().line as u64 / 10) * 100) {
                    0 => 100,
                    msec => msec,
                };
                thread::sleep(time::Duration::from_millis(sleep_msec));
                // 自然落下
                let mut game = game.lock().unwrap();
                let new_pos = Position {
                    x: game.pos.x,
                    y: game.pos.y + 1,
                };
                if !is_collision(&game.field, &new_pos, &game.block) {
                    // posの座標を更新
                    game.pos = new_pos;
                } else {
                    // ブロック落下後の処理
                    if landing(&mut game).is_err() {
                        // ブロックを生成できないならゲームオーバー
                        gameover(&game);
                    }
                }
                // フィールドを描画
                draw(&game);
            }
        });
    }

    // キー入力処理
    let g = Getch::new();
    loop {
        // キー入力待ち
        match g.getch() {
            Ok(Key::Left) => {
                let mut game = game.lock().unwrap();
                let new_pos = Position {
                    x: game.pos.x.checked_sub(1).unwrap_or(game.pos.x),
                    y: game.pos.y,
                };
                move_block(&mut game, new_pos);
                draw(&game);
            }
            Ok(Key::Down) => {
                let mut game = game.lock().unwrap();
                let new_pos = Position {
                    x: game.pos.x,
                    y: game.pos.y + 1,
                };
                move_block(&mut game, new_pos);
                draw(&game);
            }
            Ok(Key::Right) => {
                let mut game = game.lock().unwrap();
                let new_pos = Position {
                    x: game.pos.x + 1,
                    y: game.pos.y,
                };
                move_block(&mut game, new_pos);
                draw(&game);
            }
            Ok(Key::Up) => {
                // ハードドロップ
                let mut game = game.lock().unwrap();
                hard_drop(&mut game);
                if landing(&mut game).is_err() {
                    // ブロックを生成できないならゲームオーバー
                    gameover(&game);
                }
                draw(&game);
            }
            Ok(Key::Char('z')) => {
                // 左回転
                let mut game = game.lock().unwrap();
                rotate_left(&mut game);
                draw(&game);
            }
            Ok(Key::Char('x')) => {
                // 右回転
                let mut game = game.lock().unwrap();
                rotate_right(&mut game);
                draw(&game);
            }
            Ok(Key::Char(' ')) => {
                // ホールド
                let mut game = game.lock().unwrap();
                hold(&mut game);
                draw(&game);
            }
            Ok(Key::Char('q')) => {
                quit();
            }
            _ => (),  // 何もしない
        }
    }
}

// オートプレイ
pub fn auto() -> ! {
    // 自動化処理
    let _ = thread::spawn(|| {
        let mut game = Game::new();
        // 画面クリア
        println!("\x1b[2J\x1b[H\x1b[?25l");
        // フィールドを描画
        draw(&game);

        loop {
            // 指定した遺伝子で評価後のエリート個体を取得
            let elite = eval(&game, &[100, 1, 10, 100]);
            game = elite;
            // エリート個体のブロックを落下
            if landing(&mut game).is_err() {
                // ブロックを生成できないならゲームオーバー
                gameover(&game);
            }
            draw(&game);
        }
    });

    // キー入力処理
    let g = Getch::new();
    loop {
        // `q`キーで終了
        if let Ok(Key::Char('q')) = g.getch() {
            quit();
        }
    }
}
