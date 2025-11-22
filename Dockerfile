# マルチステージビルド
# ビルドステージ
FROM rust:1.76-slim AS builder

WORKDIR /app

# 依存関係をコピーしてビルド（キャッシュを活用）
# pkg-configとOpenSSLを追加（reqwestなどの依存関係で必要になる場合がある）
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# ソースコードをコピーしてビルド
COPY src ./src
# ダミーのmain.rsでビルドしたアーティファクトのタイムスタンプを更新して再ビルドをトリガー
RUN touch src/main.rs
RUN cargo build --release

# 実行ステージ
FROM debian:bookworm-slim

# 実行時に必要なライブラリをインストール（OpenSSLなど）
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# セキュリティ: 非rootユーザーを作成
RUN groupadd -r appuser && useradd -r -g appuser appuser

WORKDIR /app

# ビルド済みバイナリをコピー
COPY --from=builder /app/target/release/google-gemini-image-creator /app/google-gemini-image-creator

# 非rootユーザーに切り替え
USER appuser

# 環境変数
ENV RUST_LOG=info

# エントリーポイント
ENTRYPOINT ["/app/google-gemini-image-creator"]

