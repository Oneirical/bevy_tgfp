cargo build --profile wasm-release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web \
  --out-dir ./js/ \
  --out-name "redesign_tgfp" \
  ./target/wasm32-unknown-unknown/wasm-release/redesign_tgfp.wasm
../../../Downloads/binaryen-version_121/bin/wasm-opt -Os --output output.wasm js/redesign_tgfp_bg.wasm
rm js/redesign_tgfp_bg.wasm
mv output.wasm js/redesign_tgfp_bg.wasm
zip -vr tgfp.zip assets/ js/ index.html -x "*.DS_Store"
