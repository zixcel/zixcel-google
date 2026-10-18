# zixcel-google

provider固有のaccount/resource mappingと必要権限・副作用を持つ汎用連携libraryです。認可・限定HTTP実行は正式配布のCrowsiを利用し、業務policyや実account登録はownerアプリに残します。

[Node package](packages/provider-google/README.md) · [読み書きplaygroundと公開配布consumer](playground/provider-operations/README.ja.md) · [English](README.md)

現在のpackageは `@zixcel/provider-google` です。実account受入、完全IAM discovery、invoice正規化・監視の完成とは区別します。

Dependency updates: [repository policy](docs/dependency-updates.md).
Registry releases: [version tags and GitHub OIDC](docs/registry-release.md).

## Native管理extension

[Google IAM観測module](docs/google-admin.md) は独立したRust moduleとして `zix google iam keys inspect` を提供します。既存planning crate・npm libraryのAPIとは区別し、ローカルのGoogle SDKを必須にしません。
