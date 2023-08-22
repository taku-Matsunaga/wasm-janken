#![allow(non_snake_case)]
use dioxus::prelude::*;
use reqwest::{self, Error};
use wasm_bindgen::prelude::*;
use rand::prelude::*;
use serde::Deserialize;

fn main() {
  dioxus_web::launch(App);
}

// console.logをjsから読み込み
#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}

enum Player {
  You,
  Cpu
}

enum BgColor {
  Red,
  Blue
} 

struct JankenResultCard {
  color: BgColor,
  player: Player,
  hand: usize,
}

impl JankenResultCard {
  fn show_player(&self) -> String {
    match self.player {
      Player::Cpu => String::from("CPU"),
      Player::You => String::from("YOU"),
    }
  }

  fn show_color(&self) -> String {
    match self.color {
      BgColor::Blue => String::from("blue"),
      BgColor::Red  => String::from("red"),
    }
  }
}

// APIからのresponseの型を定義。json形式からのdeserializeや表示に使うdebugは組み込みのtraitを使う
#[derive(Deserialize, Debug, Clone)]
struct ApiResponse {
  message: String,
  status: String,
}

fn App(cx: Scope) -> Element {
  // 状態管理はuse_stateでできる。use_effect, use_futureとかもある
  let my_hand  = use_state(cx, || 0);
  let cpu_hand = use_state(cx, || 0);
  let result = use_state(cx, || "");
  let image_url = use_state(cx, || ApiResponse{
    message: String::new(),
    status: String::new(), 
  });

  let janken_result_cards = [
    JankenResultCard{ color: BgColor::Blue, player: Player::You, hand: my_hand.get().clone() },
    JankenResultCard{ color: BgColor::Red,  player: Player::Cpu, hand: cpu_hand.get().clone() },
  ];
  // 任意のタイミングで非同期処理を発火させたい場合はcx.spawnを使う
  let get_image_url = move |_| {
    cx.spawn({
      let image_url = image_url.to_owned();

      async move {
        let url = "https://dog.ceo/api/breeds/image/random";
        let res: Result<ApiResponse, Error> = reqwest::get(url)
          .await
          .unwrap()
          .json()
          .await;
        
        match res {
          Ok(_data) => {
            log(&format!("status: {}", _data.status));
            image_url.set(_data);
          }
          Err(_err) => {
            log(&format!("Error get image url : {:?}", _err));
          }
        }
      }
    })
  };

  // じゃんけんの手の画像 0: グー, 1: チョキ, 2: パー
  let hands = [
    "https://jskm.sakura.ne.jp/js01/kadai/img02/g.png",
    "https://jskm.sakura.ne.jp/js01/kadai/img02/c.png",
    "https://jskm.sakura.ne.jp/js01/kadai/img02/p.png"
  ];

  // ここからは見た目。style直接当ててるのは見逃してください。。。
  cx.render(rsx! {
    div {
      h3 {
        style: "text-align: center;",
        "じゃんけん"
      }
      p {
        style: "text-align: center;",
        "あなたの手を選んでください"
      }
      div {
        style: "display: flex; gap: 12px; align-items: center; justify-content: center;",
        hands.iter().enumerate().map(|(i, &hand)| rsx!{
          button {
            style: "padding: 4px 8px; background-color: #000; border-radius: 12px;",
            onclick: move |_| { 
              my_hand.set(i + 1);
              let tmp_cpu_hand = dicide_cpu_hand();
              cpu_hand.set(tmp_cpu_hand);
              result.set(janken(i + 1, tmp_cpu_hand));
            },
            img {
              width: "64px",
              src: hand,
            },
          }
        })
      }
      div  {
        style: "display: flex; gap: 24px; align-items: center; justify-content: center; margin-top: 64px",

        janken_result_cards.iter().map(|card| {
          let style = format!("padding: 4px 8px; border-radius: 12px; width: 64px;height: 64px; background-color: {}", card.show_color());
          rsx! {
            div {
              style: "display: flex; flex-direction: column; align-items: center; justify-content: center;",
              div {
                style: "{style}",
                img {
                  width: "64px",
                  src: if my_hand.get().clone() != 0 { hands[card.hand - 1 as usize] } else {""},
                }
              }
              p {
                style: "height: 12px; margin: 0;",
                card.show_player()
              }
            }    
          }
        })
      }

      div {
        style: "margin-top:64px;",
        h3 {
          style: "text-align: center;",
          "{result.get()}"
        }
      }
      if result.get().clone() == "負け・・・" {
        rsx! {
          div {
            style: "margin: 0 auto; width: 144px; display:flex; flex-direction: column; align-items:center;",
            button {
              onclick: get_image_url,
              "傷ついた心を癒してもらうわん"
            }
            img {
              style: "margin-top: 24px; max-height: 480px",
              src: "{image_url.get().message}"
            }
          }
        }
      }
    }
  })
}


fn dicide_cpu_hand() -> usize {
  let mut rng = rand::thread_rng();
  rng.gen_range(1..4)
}

fn janken(my_hand: usize, cpu_hand: usize) -> &'static str {
  let n = (my_hand + 3 - cpu_hand) % 3;
  let result = match n {
    0 => "あいこ",
    1 => "負け・・・",
    2 => "勝ち！",
    _ => "エラー",
  };
  log(&format!("{} {} {}", my_hand, cpu_hand, result));
  result
}
