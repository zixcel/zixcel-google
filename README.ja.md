# zixcel-google

ローカルの管理ツールやアプリから、Google Cloudの資源・権限・課金情報を確認し、
対象を限定した操作を計画・実行するためのOSSです。[English](README.md)

| 配置 | 責務 |
| --- | --- |
| `crates/google-core` | Google固有の型、取得・検証、操作要求の組み立て |
| `crates/google-cli` | `zix google` 用のネイティブ入口とTypeScript用の計画プロトコル |
| `packages/provider-google` | Rustの計画をCrowsiの承認・HTTP実行へ渡すTypeScript接続 |

Google固有の要求定義をTypeScriptへ重複実装しません。共通入口は
[zixcel/cli](https://github.com/zixcel/cli)に置き、このrepoはGoogle固有の処理を持ちます。

[導入](docs/getting-started.md) · [調査コマンドと取得範囲](docs/administration.md) ·
[TypeScript操作](packages/provider-google/README.md) ·
[合成例](playground/provider-operations/README.ja.md) · [構成](docs/architecture.md)

ネイティブの21操作は読み取りです。TypeScriptからは、Rustで生成した操作計画を
既存Crowsiの承認・一回限り実行へ渡せます。ラベル更新には取得済みetagが必要です。
計画のみの成功を、外部APIの操作成功として扱いません。

認証情報は利用者が明示的に渡します。Google SDKや暗黙のログインは必須にしません。
読み取りと無料は別であり、取得範囲・欠測・料金未評価を明示します。
FOCUS対応、実請求額の照合、OAuthログインは今回の再整理では追加していません。

開発試験はRustのworkspace試験と、`npm --prefix packages/provider-google test`で実行します。
後者は実Rustバイナリと合成HTTPを使用し、実Cloudには接続しません。
ソース反映、配布、本番・実アカウント受入は別工程です。

[License](LICENSE) · [NOTICE](NOTICE) · [Security](SECURITY.md) ·
[依存更新](docs/dependency-updates.md) · [配布](docs/registry-release.md)
