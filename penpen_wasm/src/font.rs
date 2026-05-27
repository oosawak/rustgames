/// GenInterfaceJP フォント埋め込みモジュール
///
/// `embed-font` feature を有効にしてビルドすると、
/// TTF フォントバイト列が WASM バイナリに同梱されます。
///
/// ビルド例:
///   wasm-pack build --features embed-font
///
/// このとき JS 側で `engine_font_bytes_maze3d()` を呼ぶと
/// Uint8Array が返るので FontFace API で登録できます。
/// feature が無効の場合は空の Uint8Array が返り、
/// JS 側は外部ファイル（docs/fonts/）へフォールバックします。

#[cfg(feature = "embed-font")]
const FONT_REGULAR_BYTES: &[u8] =
    include_bytes!("../../docs/fonts/GenInterfaceJP-Regular.ttf");

#[cfg(feature = "embed-font")]
const FONT_BOLD_BYTES: &[u8] =
    include_bytes!("../../docs/fonts/GenInterfaceJP-Bold.ttf");

/// Regular ウェイトのフォントバイト列を返す。
/// `embed-font` feature が無効なら空スライスを返す。
pub fn regular_bytes() -> &'static [u8] {
    #[cfg(feature = "embed-font")]
    { FONT_REGULAR_BYTES }
    #[cfg(not(feature = "embed-font"))]
    { &[] }
}

/// Bold ウェイトのフォントバイト列を返す。
/// `embed-font` feature が無効なら空スライスを返す。
pub fn bold_bytes() -> &'static [u8] {
    #[cfg(feature = "embed-font")]
    { FONT_BOLD_BYTES }
    #[cfg(not(feature = "embed-font"))]
    { &[] }
}

/// フォントが埋め込まれているかどうかを返す。
pub fn is_embedded() -> bool {
    cfg!(feature = "embed-font")
}
