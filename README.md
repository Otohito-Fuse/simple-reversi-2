# simple-reversi
簡易的なリバーシ（いわゆるオセロ）。Rustの練習。

## 操作について
基本的には指示が出ます。
盤面サイズを半角数字で入力するところ以外は
矢印キーとEnterキーしか使いません。

ターミナルのウィンドウのサイズは極力変更しないでください（レイアウトが乱れることがある）。

## ゲームの始め方
### 手元でビルドして実行する方法（要Rust）
```git clone (ここのURL)```などでダウンロードした後、
```simple-reversi-2```内で
```
cargo run
```
を打つ（Rustのインストールをしていない場合はまずそれをする）。

あとは指示が出ます。

盤面のサイズは各辺偶数マスの正方形から自由に選べます。

CPUと戦うか、自分で全部やるかも選べます。

### 実行ファイルを直接ダウンロードする方法
Releasesにあるzipファイル（v.0.1.0.3が現状最新です）のうち、自分のPCのOSに合ったものをダウンロードして解凍し、
```release```フォルダ内の```simple-reversi-2(.exe)```を実行する
（開発元が不明なためセキュリティがブロックしましたというような表示が出ると思われるが構わず実行する
（責任は取りません））。