# Google操作の合成playground

Rustの実バイナリで操作要求を組み立て、Crowsiの承認・一回限り実行へ渡します。
通信先は合成HTTPであり、実Googleアカウントには接続しません。

```sh
npm --prefix packages/provider-google ci --ignore-scripts
npm --prefix packages/provider-google test
(cd packages/provider-google && node ../../.github/scripts/provider-consumer.mts)
```

独立consumerはnpm候補をpack・展開し、コピーしたネイティブバイナリと公開APIを使います。
ラベル更新の合成etag、読み戻し、grant再利用時の拒否まで確認します。
この例のビルドhashを、本番配布のprovenance検証済みhashとは扱いません。
実利用時は[packageガイド](../../packages/provider-google/README.md)に従い、
検証済み配布物と明示的な対象・認証情報・承認を利用者が渡してください。
